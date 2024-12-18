mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct RGBA {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[wasm_bindgen]
impl RGBA {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> RGBA {
        RGBA { r, g, b, a }
    }
}

#[wasm_bindgen]
pub struct HeatPoint {
    x: f64,
    y: f64,
    value: f64,
}

#[wasm_bindgen]
impl HeatPoint {
    pub fn new(x: f64, y: f64, value: f64) -> HeatPoint {
        HeatPoint { x, y, value }
    }
}

#[wasm_bindgen]
pub struct HeatMap {
    size_x: u32,
    size_y: u32,
    radius: u32,
    max_heat: f64,
    gradient: Vec<RGBA>,
    min_x: f64,
    min_y: f64,
    per_pixel: f64,
    heat_values: Vec<f64>,
    color_values: Vec<RGBA>,
}

impl HeatMap {
    fn get_index(&self, row: u32, colume: u32) -> usize {
        (row * self.size_x + colume) as usize
    }

    fn get_row(&self, x: f64) -> u32 {
        ((x - self.min_x) / self.per_pixel).floor() as u32
    }

    fn get_col(&self, y: f64) -> u32 {
        ((y - self.min_y) / self.per_pixel).floor() as u32
    }

    fn update_heat_values(&mut self, point: &HeatPoint) {
        let radius = self.radius;
        let row = self.get_row(point.x);
        let col = self.get_col(point.y);
        for dx in -(radius as i32)..=radius as i32 {
            for dy in -(radius as i32)..=radius as i32 {
                let nx = row as i32 + dx;
                let ny = col as i32 + dy;

                if nx >= 0 && nx < self.size_x as i32 && ny >= 0 && ny < self.size_y as i32 {
                    let distance = ((dx * dx + dy * dy) as f64).sqrt();
                    let weight = 1.0 - distance / (radius as f64);
                    if weight < 0.0 {
                        continue;
                    }
                    let idx = self.get_index(nx as u32, ny as u32);
                    self.heat_values[idx] += point.value * weight;
                }
            }
        }
    }

    fn cal_pixel_color(&self, heat: f64) -> RGBA {
        let mut rgba = RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        };
        let gradient_steps = self.gradient.len();
        let step_lens = self.max_heat / gradient_steps as f64;

        let heat_stepf = (heat / step_lens).floor() as usize;
        let step_pos = heat / step_lens - heat_stepf as f64;

        if heat_stepf >= gradient_steps {
            rgba = self.gradient[gradient_steps - 1].clone();
        } else {
            if heat_stepf == 0 {
                rgba = self.gradient[0].clone();
                rgba.a = ((255.0 * step_pos).round()).clamp(0.0, 255.0) as u8;
            } else {
                let grad_pos_inv = 1.0 - step_pos;
                rgba.r = (self.gradient[heat_stepf - 1].r as f64 * grad_pos_inv
                    + self.gradient[heat_stepf].r as f64 * step_pos)
                    .round() as u8;
                rgba.g = (self.gradient[heat_stepf - 1].g as f64 * grad_pos_inv
                    + self.gradient[heat_stepf].g as f64 * step_pos)
                    .round() as u8;
                rgba.b = (self.gradient[heat_stepf - 1].b as f64 * grad_pos_inv
                    + self.gradient[heat_stepf].b as f64 * step_pos)
                    .round() as u8;
                rgba.a = (self.gradient[heat_stepf - 1].a as f64 * grad_pos_inv
                    + self.gradient[heat_stepf].a as f64 * step_pos)
                    .round() as u8;
            }
        }
        rgba
    }

    fn update_color_values(&mut self) {
        self.color_values = self
            .heat_values
            .iter()
            .map(|&value| self.cal_pixel_color(value))
            .collect();
    }
}

#[wasm_bindgen]
impl HeatMap {
    pub fn new(
        width: usize,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        max_heat: f64,
    ) -> HeatMap {
        utils::set_panic_hook();

        let radius = 10;
        let size_x = width;

        let delta_x = max_x - min_x;
        let delta_y = max_y - min_y;

        let per_pixel = delta_x / size_x as f64;
        let size_y = ((delta_y / per_pixel) as f64).ceil() as usize;

        let heat_values = vec![0.0; size_x * size_y];
        let color_values = vec![
            RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0
            };
            size_x * size_y
        ];

        let gradient = default_gradients();

        HeatMap {
            size_x: size_x as u32,
            size_y: size_y as u32,
            radius,
            max_heat,
            gradient,
            min_x,
            min_y,
            per_pixel,
            heat_values,
            color_values,
        }
    }

    pub fn add_one_point(&mut self, x: f64, y: f64, value: f64) {
        self.update_heat_values(&HeatPoint::new(x, y, value));
        self.update_color_values();
    }

    // TODO: optimize for wasm-bindgen
    // https://rustwasm.github.io/wasm-bindgen/reference/types.html
    pub fn add_points(&mut self, points: Vec<HeatPoint>) {
        for point in points {
            self.update_heat_values(&point);
        }
        self.update_color_values();
    }

    // pub fn set_width(&mut self, size_x: u32) {
    //     self.size_x = size_x;
    //     // TODO: recalculate
    // }

    pub fn set_gradients(&mut self, gradient: Vec<RGBA>) {
        self.gradient = gradient;
        self.update_color_values();
    }

    pub fn set_max_heat(&mut self, max_heat: f64) {
        self.max_heat = max_heat;
        self.update_color_values();
    }

    pub fn width(&self) -> u32 {
        self.size_x
    }

    pub fn height(&self) -> u32 {
        self.size_y
    }

    pub fn heat_value(&self, row: usize, col: usize) -> f64 {
        let idx = self.get_index(row as u32, col as u32);
        self.heat_values[idx]
    }

    pub fn color_value(&self, row: usize, col: usize) -> RGBA {
        let idx = self.get_index(row as u32, col as u32);
        self.color_values[idx]
    }

    pub fn color_values(&self) -> *const RGBA {
        self.color_values.as_ptr()
    }
}

fn default_gradients() -> Vec<RGBA> {
    vec![
        RGBA {
            r: 255,
            g: 255,
            b: 255,
            a: 255,
        },
        RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
    ]
}
