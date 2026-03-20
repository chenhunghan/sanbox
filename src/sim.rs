use crate::font;

pub type Color = (u8, u8, u8);
pub const BACKGROUND: Color = (96, 72, 50);

const TEXT_BASE: Color = (236, 239, 243);
const TEXT_ACCENT: Color = (255, 232, 188);
const TEXT_SHADOW: Color = (54, 42, 29);
const SAND_PALETTE: [Color; 7] = [
    (252, 235, 196),
    (243, 216, 170),
    (228, 193, 145),
    (208, 170, 125),
    (184, 146, 104),
    (154, 118, 82),
    (118, 88, 60),
];
const SAND_GLOW: Color = (255, 241, 208);
const SAND_WARMTH: Color = (205, 145, 94);
const SAND_SHADE: Color = (88, 66, 47);

const BASE_TEXT_SPEED_X: f32 = 0.58;
const BASE_TEXT_SPEED_Y: f32 = 0.36;
const MAX_AIRBORNE: usize = 6_000;
const AMBIENT_DIFFUSE_DIVISOR: usize = 180;
const AMBIENT_DIFFUSE_MIN: usize = 96;
const AMBIENT_DIFFUSE_MAX: usize = 3_072;
const DIFFUSE_BOOST_FRAMES: u16 = 18;
const SCRATCH_FILL_DIVISOR: usize = 520;
const SCRATCH_FILL_MIN: usize = 48;
const SCRATCH_FILL_MAX: usize = 1024;
const SCRATCH_EXPOSED_RATE: f32 = 0.010;
const SCRATCH_COVERED_RATE: f32 = 0.0035;
const SCRATCH_TRAIL_RATE: f32 = 0.0055;
const SCRATCH_SIDE_RATE: f32 = 0.0025;
const WIND_SWAY_SPEED: f32 = 0.010;
const WIND_GUST_SPEED: f32 = 0.004;
const WIND_EDDY_SPEED: f32 = 0.006;
const WIND_AIRBORNE_PUSH: f32 = 0.028;
const WIND_AIRBORNE_LIFT: f32 = 0.006;
const WIND_SURFACE_BIAS: f32 = 0.24;
const BED_LAYER_1: Color = (168, 130, 92);
const BED_LAYER_2: Color = (154, 118, 82);
const BED_LAYER_3: Color = (138, 104, 72);

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    color: Color,
}

pub struct Sim {
    pub width: usize,
    pub pixel_h: usize,

    bed: Vec<Color>,
    scratch: Vec<f32>,
    grid: Vec<Option<Color>>,
    airborne: Vec<Particle>,

    text: String,
    text_bmp: Vec<Vec<bool>>,
    pub text_w: usize,
    pub text_h: usize,

    tx: f32,
    ty: f32,
    tvx: f32,
    tvy: f32,
    motion_factor: f32,
    pub text_color: Color,
    text_flash: f32,
    diffuse_boost_frames: u16,

    frame: u64,
    rng: fastrand::Rng,

    pub fb: Vec<Option<Color>>,
}

fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mix = |lhs: u8, rhs: u8| lhs as f32 + (rhs as f32 - lhs as f32) * t;
    (
        mix(a.0, b.0).round() as u8,
        mix(a.1, b.1).round() as u8,
        mix(a.2, b.2).round() as u8,
    )
}

fn sand_ramp(t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let scaled = t * (SAND_PALETTE.len() - 1) as f32;
    let idx = scaled.floor() as usize;
    let next = (idx + 1).min(SAND_PALETTE.len() - 1);
    lerp_color(SAND_PALETTE[idx], SAND_PALETTE[next], scaled - idx as f32)
}

fn sand_color(rng: &mut fastrand::Rng, xf: f32, yf: f32, depth: f32) -> Color {
    let grain = rng.f32() * 0.20 - 0.10;
    let ridge = ((xf * 2.6) + (yf * 1.3)).sin() * 0.12 + ((xf * 6.4) - (yf * 2.8)).cos() * 0.08;
    let basin = ((xf * 1.8) - (yf * 4.0)).cos() * 0.10;
    let highlight = (0.32 + (1.0 - yf) * 0.26 + ridge * 0.85).clamp(0.0, 1.0);
    let warmth = (0.18 + xf * 0.18 + basin * 0.65 + rng.f32() * 0.10).clamp(0.0, 1.0);
    let cool_shadow = (0.12 + yf * 0.24 - ridge * 0.45 + rng.f32() * 0.04).clamp(0.0, 1.0);

    let base = sand_ramp((depth + grain + ridge * 0.55).clamp(0.0, 1.0));
    let warmed = lerp_color(base, SAND_WARMTH, warmth * 0.30);
    let shaded = lerp_color(warmed, SAND_SHADE, cool_shadow * 0.28);
    lerp_color(shaded, SAND_GLOW, highlight * 0.24)
}

fn compact_sand_color(rng: &mut fastrand::Rng, xf: f32, yf: f32, depth: f32) -> Color {
    let dune = ((xf * 2.1) + (yf * 1.0)).sin() * 0.10 + ((xf * 4.4) - (yf * 2.0)).cos() * 0.07;
    let basin = ((xf * 1.2) - (yf * 2.6)).cos() * 0.08;
    let grain = rng.f32() * 0.08 - 0.04;
    let depth = (depth + dune * 0.35 + basin * 0.18 + grain).clamp(0.0, 1.0);

    let base = sand_ramp(depth);
    let warmed = lerp_color(
        base,
        SAND_WARMTH,
        (0.10 + xf * 0.10 + basin * 0.25).clamp(0.0, 0.22),
    );
    let shaded = lerp_color(
        warmed,
        SAND_SHADE,
        (0.24 + yf * 0.20 - dune * 0.25 + rng.f32() * 0.03).clamp(0.12, 0.42),
    );
    lerp_color(
        shaded,
        SAND_GLOW,
        (0.06 + (1.0 - yf) * 0.10 + dune * 0.15).clamp(0.0, 0.14),
    )
}

fn coord_noise(x: usize, y: usize, seed: u32) -> f32 {
    let mut n = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263) ^ seed;
    n = (n ^ (n >> 13)).wrapping_mul(1_274_126_177);
    (((n >> 8) & 0xffff) as f32 / 65_535.0) * 2.0 - 1.0
}

fn layered_bed_color(
    base: Color,
    depth: f32,
    x: usize,
    y: usize,
    width: usize,
    pixel_h: usize,
) -> Color {
    let depth = depth.clamp(0.0, 1.0);
    let xf = x as f32 / width.max(1) as f32;
    let yf = y as f32 / pixel_h.max(1) as f32;
    let ripple = ((xf * 6.2) + (yf * 2.4)).sin() * 0.08 + ((xf * 13.8) - (yf * 4.7)).cos() * 0.05;
    let eddy = ((xf * 3.4) - (yf * 6.8)).cos() * 0.05;
    let coarse = coord_noise(x / 3, y / 3, 0x9E37) * 0.08;
    let medium = coord_noise(x, y, 0x7F4A) * 0.06;
    let fine = coord_noise(x * 3 + 11, y * 3 + 7, 0x51ED) * 0.035;
    let sparkle = coord_noise(x * 5 + 17, y * 5 + 3, 0xC13F);

    let sandy_depth = (0.30
        + depth * 0.34
        + ripple * 0.28
        + eddy * 0.16
        + coarse * 0.85
        + medium * 0.55
        + fine * 0.35)
        .clamp(0.0, 1.0);
    let sandy = sand_ramp(sandy_depth);

    let packed = lerp_color(base, sandy, (0.38 + depth * 0.16).clamp(0.20, 0.56));
    let stage1 = lerp_color(
        packed,
        BED_LAYER_1,
        (depth * 0.34 + coarse * 0.18 + medium * 0.10).clamp(0.0, 0.22),
    );
    let stage2 = lerp_color(
        stage1,
        BED_LAYER_2,
        ((depth - 0.30) * 0.42 + eddy * 0.12 + fine * 0.10).clamp(0.0, 0.14),
    );
    let stage3 = lerp_color(
        stage2,
        BED_LAYER_3,
        ((depth - 0.68) * 0.44 + ripple * 0.06).clamp(0.0, 0.08),
    );

    let shadow = (0.05 + depth * 0.10 + coarse.abs() * 0.06 - medium * 0.04).clamp(0.03, 0.16);
    let warmed = lerp_color(
        stage3,
        SAND_WARMTH,
        (0.07 + xf * 0.05 + eddy * 0.16).clamp(0.0, 0.14),
    );
    let mut lit = lerp_color(
        warmed,
        SAND_GLOW,
        (0.03 + (1.0 - yf) * 0.04 + ripple * 0.12).clamp(0.0, 0.09),
    );
    lit = lerp_color(lit, SAND_SHADE, shadow);

    if sparkle > 0.74 {
        lit = lerp_color(
            lit,
            SAND_GLOW,
            ((sparkle - 0.74) * 0.42 + 0.06).clamp(0.0, 0.14),
        );
    } else if sparkle < -0.76 {
        lit = lerp_color(
            lit,
            SAND_SHADE,
            ((-0.76 - sparkle) * 0.34 + 0.04).clamp(0.0, 0.10),
        );
    }

    lit
}

fn impact_scale(motion_scale: f32) -> f32 {
    (motion_scale / 3.0).clamp(1.0, 3.0)
}

fn text_velocity(rng: &mut fastrand::Rng, motion_scale: f32) -> (f32, f32) {
    let vx = BASE_TEXT_SPEED_X * motion_scale;
    let vy = BASE_TEXT_SPEED_Y * motion_scale;
    (
        if rng.bool() { vx } else { -vx },
        if rng.bool() { vy } else { -vy },
    )
}

fn seed_sand_grid(width: usize, pixel_h: usize, rng: &mut fastrand::Rng) -> Vec<Option<Color>> {
    let mut grid = vec![None; width * pixel_h];
    if width == 0 || pixel_h == 0 {
        return grid;
    }

    let h_span = pixel_h.saturating_sub(1).max(1) as f32;

    for y in 0..pixel_h {
        let yf = y as f32 / h_span;
        for x in 0..width {
            let xf = x as f32 / width.max(1) as f32;
            let broad =
                ((xf * 4.8) + (yf * 3.2)).sin() * 0.09 + ((xf * 8.6) - (yf * 6.0)).cos() * 0.06;
            let pockets = ((xf * 15.0) + (yf * 12.5)).sin() * 0.03;
            let density = (0.44 + yf * 0.24 + broad + pockets).clamp(0.24, 0.82);

            if rng.f32() < density {
                let depth = (0.22 + yf * 0.52 + broad * 0.45 + pockets * 0.30).clamp(0.0, 1.0);
                grid[y * width + x] = Some(sand_color(rng, xf, yf, depth));
            }
        }
    }

    grid
}

fn seed_sand_bed(width: usize, pixel_h: usize, rng: &mut fastrand::Rng) -> Vec<Color> {
    let mut bed = vec![BACKGROUND; width * pixel_h];
    if width == 0 || pixel_h == 0 {
        return bed;
    }

    let h_span = pixel_h.saturating_sub(1).max(1) as f32;

    for y in 0..pixel_h {
        let yf = y as f32 / h_span;
        for x in 0..width {
            let xf = x as f32 / width.max(1) as f32;
            let broad =
                ((xf * 2.8) + (yf * 1.6)).sin() * 0.11 + ((xf * 5.2) - (yf * 2.4)).cos() * 0.07;
            let pockets = ((xf * 8.5) + (yf * 6.0)).sin() * 0.04;
            let depth = (0.34 + yf * 0.32 + broad * 0.40 + pockets * 0.16).clamp(0.0, 1.0);
            bed[y * width + x] = compact_sand_color(rng, xf, yf, depth);
        }
    }

    bed
}

fn resize_sand_grid(
    old_grid: &[Option<Color>],
    old_width: usize,
    old_height: usize,
    width: usize,
    pixel_h: usize,
    rng: &mut fastrand::Rng,
) -> Vec<Option<Color>> {
    if width == 0 || pixel_h == 0 {
        return Vec::new();
    }

    if old_width == 0 || old_height == 0 || old_grid.is_empty() {
        return seed_sand_grid(width, pixel_h, rng);
    }

    let mut grid = vec![None; width * pixel_h];
    for y in 0..pixel_h {
        let src_y = y * old_height / pixel_h;
        for x in 0..width {
            let src_x = x * old_width / width;
            grid[y * width + x] = old_grid[src_y * old_width + src_x];
        }
    }

    let seeded = seed_sand_grid(width, pixel_h, rng);
    for (cell, seed) in grid.iter_mut().zip(seeded.into_iter()) {
        if cell.is_none() && seed.is_some() && rng.f32() < 0.42 {
            *cell = seed;
        }
    }

    grid
}

fn resize_sand_bed(
    old_bed: &[Color],
    old_width: usize,
    old_height: usize,
    width: usize,
    pixel_h: usize,
    rng: &mut fastrand::Rng,
) -> Vec<Color> {
    if width == 0 || pixel_h == 0 {
        return Vec::new();
    }

    if old_width == 0 || old_height == 0 || old_bed.is_empty() {
        return seed_sand_bed(width, pixel_h, rng);
    }

    let seeded = seed_sand_bed(width, pixel_h, rng);
    let mut bed = vec![BACKGROUND; width * pixel_h];
    for y in 0..pixel_h {
        let src_y = y * old_height / pixel_h;
        for x in 0..width {
            let src_x = x * old_width / width;
            let prev = old_bed[src_y * old_width + src_x];
            let fresh = seeded[y * width + x];
            bed[y * width + x] = lerp_color(prev, fresh, 0.18);
        }
    }

    bed
}

fn resize_scalar_field(
    old_field: &[f32],
    old_width: usize,
    old_height: usize,
    width: usize,
    pixel_h: usize,
) -> Vec<f32> {
    if width == 0 || pixel_h == 0 {
        return Vec::new();
    }

    if old_width == 0 || old_height == 0 || old_field.is_empty() {
        return vec![0.0; width * pixel_h];
    }

    let mut field = vec![0.0; width * pixel_h];
    for y in 0..pixel_h {
        let src_y = y * old_height / pixel_h;
        for x in 0..width {
            let src_x = x * old_width / width;
            field[y * width + x] = old_field[src_y * old_width + src_x];
        }
    }

    field
}

impl Sim {
    pub fn new(width: usize, pixel_h: usize, text: &str, scale: usize, motion_scale: f32) -> Self {
        let (text_bmp, text_w, text_h) = font::render_text(text, scale);

        let mut rng = fastrand::Rng::new();
        let (tvx, tvy) = text_velocity(&mut rng, motion_scale);
        let tx = rng.f32() * (width as f32 - text_w as f32).max(1.0);
        let ty = rng.f32() * (pixel_h as f32 - text_h as f32).max(1.0);

        let n = width * pixel_h;
        let bed = seed_sand_bed(width, pixel_h, &mut rng);
        let scratch = vec![0.0; n];
        let grid = seed_sand_grid(width, pixel_h, &mut rng);

        let mut sim = Self {
            width,
            pixel_h,
            bed,
            scratch,
            grid,
            airborne: Vec::with_capacity(8_192),
            text: text.to_owned(),
            text_bmp,
            text_w,
            text_h,
            tx,
            ty,
            tvx,
            tvy,
            motion_factor: motion_scale,
            text_color: TEXT_BASE,
            text_flash: 0.0,
            diffuse_boost_frames: DIFFUSE_BOOST_FRAMES,
            frame: 0,
            rng,
            fb: vec![None; n],
        };

        sim.build_fb();
        sim
    }

    pub fn resize(&mut self, width: usize, pixel_h: usize, scale: usize, motion_scale: f32) {
        let n = width * pixel_h;
        let old_width = self.width;
        let old_height = self.pixel_h;
        let old_text_w = self.text_w as f32;
        let old_text_h = self.text_h as f32;
        let old_motion_factor = self.motion_factor;
        let cx_ratio = if old_width > 0 {
            (self.tx + old_text_w * 0.5) / old_width as f32
        } else {
            0.5
        };
        let cy_ratio = if old_height > 0 {
            (self.ty + old_text_h * 0.5) / old_height as f32
        } else {
            0.5
        };
        let old_bed = std::mem::take(&mut self.bed);
        let old_scratch = std::mem::take(&mut self.scratch);
        let old_grid = std::mem::take(&mut self.grid);

        self.width = width;
        self.pixel_h = pixel_h;
        self.bed = resize_sand_bed(
            &old_bed,
            old_width,
            old_height,
            width,
            pixel_h,
            &mut self.rng,
        );
        self.scratch = resize_scalar_field(&old_scratch, old_width, old_height, width, pixel_h);
        self.grid = resize_sand_grid(
            &old_grid,
            old_width,
            old_height,
            width,
            pixel_h,
            &mut self.rng,
        );
        self.fb = vec![None; n];

        let (text_bmp, text_w, text_h) = font::render_text(&self.text, scale);
        self.text_bmp = text_bmp;
        self.text_w = text_w;
        self.text_h = text_h;

        let motion_ratio = motion_scale / old_motion_factor.max(f32::EPSILON);
        self.motion_factor = motion_scale;
        self.tvx *= motion_ratio;
        self.tvy *= motion_ratio;

        if old_width > 0 && old_height > 0 {
            let sx = width as f32 / old_width as f32;
            let sy = pixel_h as f32 / old_height as f32;
            for particle in &mut self.airborne {
                particle.x *= sx;
                particle.y *= sy;
                particle.vx *= motion_ratio;
                particle.vy *= motion_ratio;
            }
        } else {
            self.airborne.clear();
        }

        let max_x = (width as f32 - self.text_w as f32).max(0.0);
        let max_y = (pixel_h as f32 - self.text_h as f32).max(0.0);
        self.tx = (cx_ratio * width as f32 - self.text_w as f32 * 0.5).clamp(0.0, max_x);
        self.ty = (cy_ratio * pixel_h as f32 - self.text_h as f32 * 0.5).clamp(0.0, max_y);
        self.diffuse_boost_frames = DIFFUSE_BOOST_FRAMES;

        self.build_fb();
    }

    pub fn step(&mut self) {
        self.frame += 1;
        self.text_flash *= 0.90;

        self.move_text();
        self.scatter_sand();
        self.scratch_bed();
        self.update_airborne();
        self.diffuse_sand();
        self.settle_scratches();
        self.diffuse_boost_frames = self.diffuse_boost_frames.saturating_sub(1);
        self.build_fb();
    }

    fn move_text(&mut self) {
        self.tx += self.tvx;
        self.ty += self.tvy;

        let max_x = (self.width as f32 - self.text_w as f32).max(0.0);
        let max_y = (self.pixel_h as f32 - self.text_h as f32).max(0.0);

        let mut bounced = false;
        if self.tx <= 0.0 {
            self.tx = 0.0;
            self.tvx = self.tvx.abs();
            bounced = true;
        } else if self.tx >= max_x {
            self.tx = max_x;
            self.tvx = -self.tvx.abs();
            bounced = true;
        }

        if self.ty <= 0.0 {
            self.ty = 0.0;
            self.tvy = self.tvy.abs();
            bounced = true;
        } else if self.ty >= max_y {
            self.ty = max_y;
            self.tvy = -self.tvy.abs();
            bounced = true;
        }

        if bounced {
            self.text_flash = 1.0;
        }
    }

    fn wind_at(&self, x: usize, y: usize) -> (f32, f32) {
        let xf = x as f32 / self.width.max(1) as f32;
        let yf = y as f32 / self.pixel_h.max(1) as f32;
        let t = self.frame as f32;

        let sweep = (t * WIND_SWAY_SPEED + yf * 4.2 + xf * 1.1).sin();
        let gust = (t * WIND_GUST_SPEED + xf * 5.6 - yf * 1.7).cos();
        let eddy = (t * WIND_EDDY_SPEED + xf * 3.1 + yf * 4.6).sin();
        let pulse = 0.45 + (((t * 0.0026) + xf * 1.3).sin() * 0.5 + 0.5) * 0.55;

        let horizontal = (sweep * 0.58 + gust * 0.42) * pulse;
        let vertical = (eddy * 0.12 - sweep * 0.06 + gust * 0.03) * pulse;
        (horizontal, vertical)
    }

    fn wind_step(&mut self, wind_x: f32, wind_y: f32) -> (isize, isize) {
        let dx_roll = self.rng.f32() + wind_x * WIND_SURFACE_BIAS;
        let dy_roll = self.rng.f32() + wind_y * (WIND_SURFACE_BIAS * 0.65);

        let mut dx = if dx_roll > 0.76 {
            1
        } else if dx_roll < 0.24 {
            -1
        } else {
            0
        };

        let mut dy = if dy_roll > 0.88 {
            1
        } else if dy_roll < 0.12 {
            -1
        } else {
            0
        };

        if dx == 0 && dy == 0 {
            dx = self.rng.i32(-1..=1) as isize;
            if dx == 0 {
                dx = if wind_x >= 0.0 { 1 } else { -1 };
            }

            if self.rng.f32() < 0.22 {
                dy = self.rng.i32(-1..=1) as isize;
            }
        }

        (dx, dy)
    }

    fn deepen_scratch(&mut self, x: usize, y: usize, amount: f32) {
        let idx = y * self.width + x;
        let depth = self.scratch[idx];
        let resistance = 1.0 - depth * 0.72;
        self.scratch[idx] = (depth + amount * resistance.max(0.18)).min(1.0);
    }

    fn scratch_bed(&mut self) {
        let tx_i = self.tx as i32;
        let ty_i = self.ty as i32;
        let speed = (self.tvx * self.tvx + self.tvy * self.tvy)
            .sqrt()
            .clamp(0.45, 2.2);
        let center_rate = SCRATCH_EXPOSED_RATE * speed;
        let covered_rate = SCRATCH_COVERED_RATE * speed;
        let trail_rate = SCRATCH_TRAIL_RATE * speed;
        let side_rate = SCRATCH_SIDE_RATE * speed;

        let trail_dx = if self.tvx > 0.08 {
            -1
        } else if self.tvx < -0.08 {
            1
        } else {
            0
        };
        let trail_dy = if self.tvy > 0.08 {
            -1
        } else if self.tvy < -0.08 {
            1
        } else {
            0
        };

        for ly in 0..self.text_h as i32 {
            for lx in 0..self.text_w as i32 {
                if !self.text_bmp[ly as usize][lx as usize] {
                    continue;
                }

                let gx = tx_i + lx;
                let gy = ty_i + ly;
                if gx < 0 || gy < 0 || gx >= self.width as i32 || gy >= self.pixel_h as i32 {
                    continue;
                }

                let ux = gx as usize;
                let uy = gy as usize;
                let idx = uy * self.width + ux;
                let amount = if self.grid[idx].is_some() {
                    covered_rate
                } else {
                    center_rate
                };
                self.deepen_scratch(ux, uy, amount);

                let tx = gx + trail_dx;
                let ty = gy + trail_dy;
                if tx >= 0 && ty >= 0 && tx < self.width as i32 && ty < self.pixel_h as i32 {
                    self.deepen_scratch(tx as usize, ty as usize, trail_rate);
                }

                if trail_dx != 0 {
                    let sy1 = gy - 1;
                    let sy2 = gy + 1;
                    if sy1 >= 0 {
                        self.deepen_scratch(ux, sy1 as usize, side_rate);
                    }
                    if sy2 < self.pixel_h as i32 {
                        self.deepen_scratch(ux, sy2 as usize, side_rate);
                    }
                } else if trail_dy != 0 {
                    let sx1 = gx - 1;
                    let sx2 = gx + 1;
                    if sx1 >= 0 {
                        self.deepen_scratch(sx1 as usize, uy, side_rate);
                    }
                    if sx2 < self.width as i32 {
                        self.deepen_scratch(sx2 as usize, uy, side_rate);
                    }
                }
            }
        }
    }

    fn settle_scratches(&mut self) {
        if self.width == 0 || self.pixel_h == 0 {
            return;
        }

        let total = self.width * self.pixel_h;
        let samples = (total / SCRATCH_FILL_DIVISOR).clamp(SCRATCH_FILL_MIN, SCRATCH_FILL_MAX);

        for _ in 0..samples {
            let x = self.rng.usize(0..self.width);
            let y = self.rng.usize(0..self.pixel_h);
            if self.is_text_pixel(x, y) {
                continue;
            }

            let idx = y * self.width + x;
            let depth = self.scratch[idx];
            if depth <= 0.0 {
                continue;
            }

            let fill = if self.grid[idx].is_some() {
                0.013
            } else {
                0.0028
            };
            self.scratch[idx] = (depth - fill).max(0.0);
        }
    }

    #[inline]
    fn is_text_pixel(&self, x: usize, y: usize) -> bool {
        let rx = x as i32 - self.tx as i32;
        let ry = y as i32 - self.ty as i32;
        if rx < 0 || ry < 0 || rx >= self.text_w as i32 || ry >= self.text_h as i32 {
            return false;
        }
        self.text_bmp[ry as usize][rx as usize]
    }

    #[inline]
    fn in_bounds(&self, x: isize, y: isize) -> bool {
        x >= 0 && y >= 0 && x < self.width as isize && y < self.pixel_h as isize
    }

    #[inline]
    fn cell_blocked(&self, x: usize, y: usize) -> bool {
        self.grid[y * self.width + x].is_some() || self.is_text_pixel(x, y)
    }

    fn try_deposit(&mut self, x: usize, y: usize, color: Color) -> bool {
        let dirs = [
            (0isize, 0isize),
            (0, 1),
            (-1, 1),
            (1, 1),
            (-1, 0),
            (1, 0),
            (0, -1),
            (-1, -1),
            (1, -1),
        ];
        let start = self.rng.usize(0..dirs.len());

        for offset in 0..dirs.len() {
            let (dx, dy) = dirs[(start + offset) % dirs.len()];
            let nx = x as isize + dx;
            let ny = y as isize + dy;
            if !self.in_bounds(nx, ny) {
                continue;
            }

            let nx = nx as usize;
            let ny = ny as usize;
            let idx = ny * self.width + nx;
            if self.grid[idx].is_none() && !self.is_text_pixel(nx, ny) {
                self.grid[idx] = Some(color);
                return true;
            }
        }

        false
    }

    fn scatter_sand(&mut self) {
        if self.airborne.len() > MAX_AIRBORNE / 2 {
            return;
        }

        let tx_i = self.tx as i32;
        let ty_i = self.ty as i32;

        for ly in 0..self.text_h as i32 {
            for lx in 0..self.text_w as i32 {
                let gx = tx_i + lx;
                let gy = ty_i + ly;
                if gx < 0 || gy < 0 || gx >= self.width as i32 || gy >= self.pixel_h as i32 {
                    continue;
                }

                if !self.text_bmp[ly as usize][lx as usize] {
                    continue;
                }

                let ux = gx as usize;
                let uy = gy as usize;
                let idx = uy * self.width + ux;
                if let Some(color) = self.grid[idx].take() {
                    if self.rng.f32() < 0.03 {
                        self.grid[idx] = Some(color);
                        continue;
                    }

                    let edge_y = ly as f32 / self.text_h.max(1) as f32 - 0.5;
                    let edge_x = lx as f32 / self.text_w.max(1) as f32 - 0.5;
                    let spread = impact_scale(self.motion_factor);
                    let push = 1.25 + self.rng.f32() * 0.55;
                    let vx = self.tvx * push
                        + edge_x * 0.75 * spread
                        + (self.rng.f32() - 0.5) * 0.75 * spread;
                    let vy = self.tvy * push
                        + edge_y * 0.75 * spread
                        + (self.rng.f32() - 0.5) * 0.75 * spread;
                    self.airborne.push(Particle {
                        x: gx as f32,
                        y: gy as f32,
                        vx,
                        vy,
                        color,
                    });
                }
            }
        }
    }

    fn update_airborne(&mut self) {
        let mut next = Vec::with_capacity(self.airborne.len());
        let airborne = std::mem::take(&mut self.airborne);

        for mut p in airborne {
            let jitter = 0.06 * impact_scale(self.motion_factor);
            p.vx += (self.rng.f32() - 0.5) * jitter;
            p.vy += (self.rng.f32() - 0.5) * jitter;
            p.vx *= 0.94;
            p.vy *= 0.94;

            p.x += p.vx;
            p.y += p.vy;

            if p.x < 0.0 {
                p.x = 0.0;
                p.vx = p.vx.abs() * 0.20;
            } else if p.x >= self.width as f32 - 1.0 {
                p.x = self.width as f32 - 1.0;
                p.vx = -p.vx.abs() * 0.20;
            }

            if p.y < 0.0 {
                p.y = 0.0;
                p.vy = p.vy.abs() * 0.20;
            } else if p.y >= self.pixel_h as f32 - 1.0 {
                p.y = self.pixel_h as f32 - 1.0;
                p.vy = -p.vy.abs() * 0.12;
            }

            let gx = p.x.clamp(0.0, self.width.saturating_sub(1) as f32) as usize;
            let gy = p.y.clamp(0.0, self.pixel_h.saturating_sub(1) as f32) as usize;
            let (wind_x, wind_y) = self.wind_at(gx, gy);
            p.vx += wind_x * WIND_AIRBORNE_PUSH;
            p.vy += wind_y * (WIND_AIRBORNE_PUSH * 0.65) - wind_x.abs() * WIND_AIRBORNE_LIFT;
            let speed_sq = p.vx * p.vx + p.vy * p.vy;

            if self.is_text_pixel(gx, gy) {
                let bounce_kick = 0.85 * impact_scale(self.motion_factor);
                p.vx = self.tvx * 1.15 + (self.rng.f32() - 0.5) * bounce_kick;
                p.vy = self.tvy * 1.15 + (self.rng.f32() - 0.5) * bounce_kick;
                next.push(p);
                continue;
            }

            if speed_sq < 0.05 && self.try_deposit(gx, gy, p.color) {
                continue;
            }

            if self.cell_blocked(gx, gy) && self.try_deposit(gx, gy, p.color) {
                continue;
            }

            next.push(p);
        }

        if next.len() > MAX_AIRBORNE {
            let keep_from = next.len() - MAX_AIRBORNE;
            self.airborne = next.split_off(keep_from);
        } else {
            self.airborne = next;
        }
    }

    fn diffuse_sand(&mut self) {
        if self.width == 0 || self.pixel_h == 0 {
            return;
        }

        let total = self.width * self.pixel_h;
        let boost = if self.diffuse_boost_frames > 0 { 3 } else { 1 };
        let samples = ((total / AMBIENT_DIFFUSE_DIVISOR) * boost)
            .clamp(AMBIENT_DIFFUSE_MIN, AMBIENT_DIFFUSE_MAX * boost);

        for _ in 0..samples {
            let x = self.rng.usize(0..self.width);
            let y = self.rng.usize(0..self.pixel_h);
            let idx = y * self.width + x;
            let color = match self.grid[idx] {
                Some(color) => color,
                None => continue,
            };

            if self.is_text_pixel(x, y) {
                continue;
            }

            for _ in 0..3 {
                let (wind_x, wind_y) = self.wind_at(x, y);
                let (dx, dy) = self.wind_step(wind_x, wind_y);
                if dx == 0 && dy == 0 {
                    continue;
                }

                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if !self.in_bounds(nx, ny) {
                    continue;
                }

                let nx = nx as usize;
                let ny = ny as usize;
                let nidx = ny * self.width + nx;
                if self.grid[nidx].is_some() || self.is_text_pixel(nx, ny) {
                    continue;
                }

                self.grid[nidx] = Some(color);
                self.grid[idx] = None;
                break;
            }
        }
    }

    fn build_fb(&mut self) {
        for (idx, ((dst, &bed_color), &depth)) in self
            .fb
            .iter_mut()
            .zip(&self.bed)
            .zip(&self.scratch)
            .enumerate()
        {
            let x = idx % self.width.max(1);
            let y = idx / self.width.max(1);
            *dst = Some(layered_bed_color(
                bed_color,
                depth,
                x,
                y,
                self.width,
                self.pixel_h,
            ));
        }

        for (dst, grain) in self.fb.iter_mut().zip(&self.grid) {
            if let Some(color) = grain {
                *dst = Some(*color);
            }
        }

        for p in &self.airborne {
            let gx = p.x as usize;
            let gy = p.y as usize;
            if gx < self.width && gy < self.pixel_h {
                self.fb[gy * self.width + gx] = Some(p.color);
            }
        }

        let tx_i = self.tx as i32;
        let ty_i = self.ty as i32;
        let text_color = lerp_color(self.text_color, TEXT_ACCENT, self.text_flash * 0.55);
        let shadow_x = tx_i + 1;
        let shadow_y = ty_i + 1;

        for ly in 0..self.text_h {
            for lx in 0..self.text_w {
                if !self.text_bmp[ly][lx] {
                    continue;
                }

                let sx = shadow_x + lx as i32;
                let sy = shadow_y + ly as i32;
                if sx >= 0 && sy >= 0 && (sx as usize) < self.width && (sy as usize) < self.pixel_h
                {
                    let sidx = sy as usize * self.width + sx as usize;
                    let under = self.fb[sidx].unwrap_or(BACKGROUND);
                    self.fb[sidx] = Some(lerp_color(under, TEXT_SHADOW, 0.58));
                }

                let gx = tx_i + lx as i32;
                let gy = ty_i + ly as i32;
                if gx >= 0 && gy >= 0 && (gx as usize) < self.width && (gy as usize) < self.pixel_h
                {
                    self.fb[gy as usize * self.width + gx as usize] = Some(text_color);
                }
            }
        }
    }

    #[inline]
    pub fn framebuffer(&self) -> &[Option<Color>] {
        &self.fb
    }
}
