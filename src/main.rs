mod font;
mod renderer;
mod sim;

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use renderer::TextRenderer;
use std::io::{self, Write};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "san", about = "Sand + text bouncing terminal animation")]
struct Cli {
    /// Text to bounce around the terminal
    text: String,

    /// Optional fun word (e.g. "box") — purely cosmetic
    #[arg(default_value = None)]
    _extra: Option<String>,
}

const TARGET_FPS: u64 = 20;
const FRAME_DUR: Duration = Duration::from_micros(1_000_000 / TARGET_FPS);

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Setup panic hook to restore terminal.
    let orig_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let mut out = io::stdout();
        let _ = execute!(out, LeaveAlternateScreen, cursor::Show);
        let _ = terminal::disable_raw_mode();
        orig_hook(info);
    }));

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(ClearType::All)
    )?;

    let mut renderer = TextRenderer::new();
    let (term_w, term_h) = terminal::size()?;
    let sim_size = renderer.sim_size(term_w, term_h);
    let mut sim = sim::Sim::new(
        sim_size.width,
        sim_size.pixel_h,
        &cli.text,
        sim_size.font_scale,
        sim_size.motion_scale,
    );

    let mut buf: Vec<u8> = Vec::with_capacity(sim_size.width * sim_size.pixel_h * 8);

    loop {
        let t0 = Instant::now();

        // Handle events.
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    restore_terminal(&mut stdout)?;
                    return Ok(());
                }
                Event::Resize(nw, nh) => {
                    let sim_size = renderer.sim_size(nw, nh);
                    sim.resize(
                        sim_size.width,
                        sim_size.pixel_h,
                        sim_size.font_scale,
                        sim_size.motion_scale,
                    );
                }
                _ => {}
            }
        }

        // Step simulation.
        sim.step();

        renderer.render(&sim, &mut stdout, &mut buf)?;
        stdout.flush()?;

        // Sleep to maintain target FPS.
        let elapsed = t0.elapsed();
        if elapsed < FRAME_DUR {
            std::thread::sleep(FRAME_DUR - elapsed);
        }
    }
}

fn restore_terminal(stdout: &mut io::Stdout) -> io::Result<()> {
    execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()
}
