use wasm_bindgen::prelude::*;

use crate::{
    font,
    sim::{self, Sim},
};

const TARGET_INTERNAL_AREA: f32 = 34_000.0;
const MIN_INTERNAL_WIDTH: usize = 140;
const MIN_INTERNAL_HEIGHT: usize = 96;
const MAX_INTERNAL_WIDTH: usize = 320;
const MAX_INTERNAL_HEIGHT: usize = 220;

fn web_sim_size(pixel_width: usize, pixel_height: usize) -> (usize, usize, usize, f32) {
    let safe_w = pixel_width.max(MIN_INTERNAL_WIDTH);
    let safe_h = pixel_height.max(MIN_INTERNAL_HEIGHT);
    let area = (safe_w * safe_h) as f32;
    let scale = (area / TARGET_INTERNAL_AREA).sqrt().ceil().clamp(1.0, 4.0) as usize;

    let width = (safe_w / scale).clamp(MIN_INTERNAL_WIDTH, MAX_INTERNAL_WIDTH);
    let pixel_h = (safe_h / scale).clamp(MIN_INTERNAL_HEIGHT, MAX_INTERNAL_HEIGHT);
    let font_scale = ((width.min(pixel_h) as f32 / 42.0).round() as usize).clamp(3, 5);
    let motion_scale = 1.0;

    (width, pixel_h, font_scale, motion_scale)
}

fn fitted_font_scale(text: &str, width: usize, pixel_h: usize, fallback: usize) -> usize {
    let chars = text.chars().count().max(1);
    let max_text_w = (width as f32 * 0.82).round() as usize;
    let max_text_h = (pixel_h as f32 * 0.34).round() as usize;

    for scale in (3..=6).rev() {
        let total_w = chars * (font::CHAR_W + font::CHAR_GAP) - font::CHAR_GAP;
        let text_w = total_w * scale;
        let text_h = font::CHAR_H * scale;
        if text_w <= max_text_w.max(1) && text_h <= max_text_h.max(1) {
            return scale;
        }
    }

    fallback
}

#[wasm_bindgen]
pub struct WebSim {
    sim: Sim,
    rgba: Vec<u8>,
    text: String,
}

#[wasm_bindgen]
impl WebSim {
    #[wasm_bindgen(constructor)]
    pub fn new(pixel_width: u32, pixel_height: u32, text: String) -> Self {
        console_error_panic_hook::set_once();

        let (width, pixel_h, fallback_scale, motion_scale) =
            web_sim_size(pixel_width as usize, pixel_height as usize);
        let font_scale = fitted_font_scale(&text, width, pixel_h, fallback_scale);
        let sim = Sim::new(width, pixel_h, &text, font_scale, motion_scale);
        let rgba = vec![0; width * pixel_h * 4];

        Self { sim, rgba, text }
    }

    pub fn resize(&mut self, pixel_width: u32, pixel_height: u32) {
        let (width, pixel_h, fallback_scale, motion_scale) =
            web_sim_size(pixel_width as usize, pixel_height as usize);
        let font_scale = fitted_font_scale(&self.text, width, pixel_h, fallback_scale);
        self.sim.resize(width, pixel_h, font_scale, motion_scale);
        self.rgba.resize(width * pixel_h * 4, 0);
    }

    pub fn set_text(&mut self, text: String) {
        let width = self.sim.width;
        let pixel_h = self.sim.pixel_h;
        let fallback_scale = ((width.min(pixel_h) as f32 / 42.0).round() as usize).clamp(3, 5);
        let font_scale = fitted_font_scale(&text, width, pixel_h, fallback_scale);
        let motion_scale = 1.0;
        self.text = text.clone();
        self.sim = Sim::new(width, pixel_h, &text, font_scale, motion_scale);
        self.rgba.resize(width * pixel_h * 4, 0);
    }

    pub fn step(&mut self) {
        self.sim.step();
    }

    pub fn width(&self) -> u32 {
        self.sim.width as u32
    }

    pub fn height(&self) -> u32 {
        self.sim.pixel_h as u32
    }

    pub fn background_css(&self) -> String {
        let (r, g, b) = sim::BACKGROUND;
        format!("rgb({r}, {g}, {b})")
    }

    pub fn text(&self) -> String {
        self.text.clone()
    }

    pub fn render_frame(&mut self) {
        let fb = self.sim.framebuffer();
        let (br, bg, bb) = sim::BACKGROUND;

        if self.rgba.len() != fb.len() * 4 {
            self.rgba.resize(fb.len() * 4, 0);
        }

        for (pixel, rgba) in fb.iter().zip(self.rgba.chunks_exact_mut(4)) {
            let (r, g, b) = pixel.unwrap_or((br, bg, bb));
            rgba[0] = r;
            rgba[1] = g;
            rgba[2] = b;
            rgba[3] = 255;
        }
    }

    pub fn frame_ptr(&self) -> usize {
        self.rgba.as_ptr() as usize
    }

    pub fn frame_len(&self) -> usize {
        self.rgba.len()
    }

    pub fn frame_rgba(&self) -> Vec<u8> {
        self.rgba.clone()
    }
}
