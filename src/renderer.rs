use std::io::{self, Write};

use crate::sim;

const TEXT_WIDTH_SCALE: usize = 1;
const TEXT_HEIGHT_SCALE: usize = 2;
const TEXT_FONT_SCALE: usize = 3;

pub struct SimSize {
    pub width: usize,
    pub pixel_h: usize,
    pub font_scale: usize,
    pub motion_scale: f32,
}

pub(crate) struct TextRenderer;

impl TextRenderer {
    pub fn new() -> Self {
        Self
    }

    pub fn sim_size(&self, cols: u16, rows: u16) -> SimSize {
        SimSize {
            width: cols as usize * TEXT_WIDTH_SCALE,
            pixel_h: rows as usize * TEXT_HEIGHT_SCALE,
            font_scale: TEXT_FONT_SCALE,
            motion_scale: 1.0,
        }
    }

    pub fn render<W: Write>(
        &mut self,
        sim: &sim::Sim,
        out: &mut W,
        buf: &mut Vec<u8>,
    ) -> io::Result<()> {
        buf.clear();
        self.render_into(sim, buf);
        out.write_all(buf)
    }

    fn render_into(&mut self, sim: &sim::Sim, buf: &mut Vec<u8>) {
        let w = sim.width;
        let ph = sim.pixel_h;
        let term_h = ph / 2;
        let bg = sim::BACKGROUND;
        let fb = sim.framebuffer();

        buf.extend_from_slice(b"\x1b[H");

        let mut prev_fg: Option<(u8, u8, u8)> = None;
        let mut prev_bg: Option<(u8, u8, u8)> = None;

        for row in 0..term_h {
            let py_top = row * 2;
            let py_bot = py_top + 1;

            for col in 0..w {
                let top = fb[py_top * w + col];
                let bot = if py_bot < ph {
                    fb[py_bot * w + col]
                } else {
                    None
                };

                match (top, bot) {
                    (Some(tc), Some(bc)) if tc == bc => {
                        set_bg(buf, tc, &mut prev_bg);
                        buf.push(b' ');
                    }
                    (Some(tc), Some(bc)) => {
                        set_fg(buf, tc, &mut prev_fg);
                        set_bg(buf, bc, &mut prev_bg);
                        buf.extend_from_slice("▀".as_bytes());
                    }
                    (Some(tc), None) => {
                        set_fg(buf, tc, &mut prev_fg);
                        set_bg(buf, bg, &mut prev_bg);
                        buf.extend_from_slice("▀".as_bytes());
                    }
                    (None, Some(bc)) => {
                        set_fg(buf, bc, &mut prev_fg);
                        set_bg(buf, bg, &mut prev_bg);
                        buf.extend_from_slice("▄".as_bytes());
                    }
                    (None, None) => {
                        set_bg(buf, bg, &mut prev_bg);
                        buf.push(b' ');
                    }
                }
            }

            if row + 1 < term_h {
                buf.extend_from_slice(b"\r\n");
            }
        }

        buf.extend_from_slice(b"\x1b[0m");
    }
}

#[inline]
fn set_fg(buf: &mut Vec<u8>, c: (u8, u8, u8), prev: &mut Option<(u8, u8, u8)>) {
    if *prev != Some(c) {
        write_ansi_color(buf, 38, c);
        *prev = Some(c);
    }
}

#[inline]
fn set_bg(buf: &mut Vec<u8>, c: (u8, u8, u8), prev: &mut Option<(u8, u8, u8)>) {
    if *prev != Some(c) {
        write_ansi_color(buf, 48, c);
        *prev = Some(c);
    }
}

#[inline]
fn write_ansi_color(buf: &mut Vec<u8>, kind: u8, (r, g, b): (u8, u8, u8)) {
    buf.extend_from_slice(b"\x1b[");
    itoa_push(buf, kind);
    buf.extend_from_slice(b";2;");
    itoa_push(buf, r);
    buf.push(b';');
    itoa_push(buf, g);
    buf.push(b';');
    itoa_push(buf, b);
    buf.push(b'm');
}

#[inline]
fn itoa_push(buf: &mut Vec<u8>, n: u8) {
    if n >= 100 {
        buf.push(b'0' + n / 100);
        buf.push(b'0' + (n / 10) % 10);
        buf.push(b'0' + n % 10);
    } else if n >= 10 {
        buf.push(b'0' + n / 10);
        buf.push(b'0' + n % 10);
    } else {
        buf.push(b'0' + n);
    }
}
