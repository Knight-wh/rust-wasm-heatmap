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
    flip_y: bool,
    heat_values: Vec<f64>,
    color_values: Vec<RGBA>,
    points: Vec<HeatPoint>,
}

impl HeatMap {
    fn get_index(&self, row: u32, colume: u32) -> usize {
        (row * self.size_x + colume) as usize
    }

    fn get_row(&self, y: f64) -> i32 {
        if self.flip_y {
            self.size_y as i32 - ((y - self.min_y) / self.per_pixel).floor() as i32 - 1
        } else {
            ((y - self.min_y) / self.per_pixel).floor() as i32
        }
    }

    fn get_col(&self, x: f64) -> i32 {
        ((x - self.min_x) / self.per_pixel).floor() as i32
    }

    fn update_heat_values(&mut self, point: &HeatPoint) {
        let radius = self.radius;
        let row = self.get_row(point.y);
        let col = self.get_col(point.x);
        for dx in -(radius as i32)..=radius as i32 {
            for dy in -(radius as i32)..=radius as i32 {
                let nx = col + dx;
                let ny = row + dy;

                if nx >= 0 && nx < self.size_x as i32 && ny >= 0 && ny < self.size_y as i32 {
                    let distance = ((dx * dx + dy * dy) as f64).sqrt();
                    let weight = 1.0 - distance / (radius as f64);
                    if weight < 0.0 {
                        continue;
                    }
                    let idx = self.get_index(ny as u32, nx as u32);
                    self.heat_values[idx] += point.value * weight;
                }
            }
        }
    }

    fn update_heat_values_v2(&mut self) {
        for point in &self.points {
            let radius = self.radius;
            let row = self.get_row(point.y);
            let col = self.get_col(point.x);
            for dx in -(radius as i32)..=radius as i32 {
                let nx = col + dx;
                if nx >= 0 && nx < self.size_x as i32 {
                    for dy in -(radius as i32)..=radius as i32 {
                        let ny = row + dy;
                        if ny >= 0 && ny < self.size_y as i32 {
                            let distance = ((dx * dx + dy * dy) as f64).sqrt();
                            let weight = 1.0 - distance / (radius as f64);
                            if weight < 0.0 {
                                continue;
                            }
                            let idx = self.get_index(ny as u32, nx as u32);
                            self.heat_values[idx] += point.value * weight;
                        }
                    }
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

    fn update_color_values_v2(&mut self) {
        for (i, &value) in self.heat_values.iter().enumerate() {
            self.color_values[i] = self.cal_pixel_color(value);
        }
    }
}

#[wasm_bindgen]
impl HeatMap {
    pub fn new(
        width: usize,
        radius: usize,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        max_heat: f64,
    ) -> HeatMap {
        utils::set_panic_hook();

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

        let flip_y = true;

        HeatMap {
            size_x: size_x as u32,
            size_y: size_y as u32,
            radius: radius as u32,
            max_heat,
            gradient,
            min_x,
            min_y,
            per_pixel,
            flip_y,
            heat_values,
            color_values,
            points: Vec::new(),
        }
    }

    pub fn add_one_point(&mut self, x: f64, y: f64, value: f64) {
        self.update_heat_values(&HeatPoint::new(x, y, value));
        self.update_color_values();
    }

    // https://rustwasm.github.io/wasm-bindgen/reference/types.html
    pub fn add_points(&mut self, points: Vec<HeatPoint>) {
        for point in points {
            self.update_heat_values(&point);
        }
        self.update_color_values();
    }

    // * Optimize for receiving data across the WebAssembly ABI boundary
    pub fn add_points_v2(&mut self, points: Vec<f64>) {
        assert!(
            points.len() % 3 == 0,
            "Points length must be a multiple of 3"
        );

        for chunk in points.chunks(3) {
            let x = chunk[0];
            let y = chunk[1];
            let value = chunk[2];
            self.update_heat_values(&HeatPoint::new(x, y, value));
        }

        self.update_color_values();
    }

    pub fn add_points_v3(&mut self, points: Vec<f64>) {
        assert!(
            points.len() % 3 == 0,
            "Points length must be a multiple of 3"
        );

        for chunk in points.chunks(3) {
            let x = chunk[0];
            let y = chunk[1];
            let value = chunk[2];
            self.points.push(HeatPoint::new(x, y, value));
        }
    }

    pub fn calc_heatmap(&mut self) {
        self.update_heat_values_v2();
        self.update_color_values_v2();
    }

    pub fn tag1(&mut self) {
        self.update_heat_values_v2();
    }

    pub fn tag2(&mut self) {
        self.update_color_values_v2();
    }

    pub fn set_gradients(&mut self, gradient: Vec<RGBA>) {
        self.gradient = gradient;
    }

    pub fn set_max_heat(&mut self, max_heat: f64) {
        self.max_heat = max_heat;
    }

    pub fn set_radius(&mut self, radius: usize) {
        self.radius = radius as u32;
    }

    pub fn set_flip_y(&mut self, flip_y: bool) {
        self.flip_y = flip_y;
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

    pub fn reset(&mut self) {
        self.heat_values.fill(0.0);
        // self.color_values.fill(RGBA {
        //     r: 0,
        //     g: 0,
        //     b: 0,
        //     a: 0,
        // });
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

#[wasm_bindgen]
pub struct HeatMapV2 {
    size_x: u32,
    size_y: u32,
    min_x: f64,
    min_y: f64,
    per_pixel: f64,
    radius: u32,
    max_heat: f64,
    gradient: Vec<RGBA>,
    flip_y: bool,
    points: Vec<HeatPoint>,
    heat_values: Vec<f64>,
    color_values: Vec<u8>,
}

impl HeatMapV2 {
    fn get_index(&self, row: u32, colume: u32) -> usize {
        (row * self.size_x + colume) as usize
    }

    fn get_row(&self, y: f64) -> i32 {
        if self.flip_y {
            self.size_y as i32 - ((y - self.min_y) / self.per_pixel).floor() as i32 - 1
        } else {
            ((y - self.min_y) / self.per_pixel).floor() as i32
        }
    }

    fn get_col(&self, x: f64) -> i32 {
        ((x - self.min_x) / self.per_pixel).floor() as i32
    }

    fn update_heat_values(&mut self) {
        for point in &self.points {
            let radius = self.radius;
            let row = self.get_row(point.y);
            let col = self.get_col(point.x);
            for dx in -(radius as i32)..=radius as i32 {
                let nx = col + dx;
                if nx >= 0 && nx < self.size_x as i32 {
                    for dy in -(radius as i32)..=radius as i32 {
                        let ny = row + dy;
                        if ny >= 0 && ny < self.size_y as i32 {
                            let distance = ((dx * dx + dy * dy) as f64).sqrt();
                            let weight = 1.0 - distance / (radius as f64);
                            if weight < 0.0 {
                                continue;
                            }
                            let idx = self.get_index(ny as u32, nx as u32);
                            self.heat_values[idx] += point.value * weight;
                        }
                    }
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
        for (i, &value) in self.heat_values.iter().enumerate() {
            let rgba = self.cal_pixel_color(value);
            let idx = i * 4;
            self.color_values[idx] = rgba.r;
            self.color_values[idx + 1] = rgba.g;
            self.color_values[idx + 2] = rgba.b;
            self.color_values[idx + 3] = rgba.a;
        }
    }
}

#[wasm_bindgen]
impl HeatMapV2 {
    pub fn new() -> HeatMapV2 {
        HeatMapV2 {
            size_x: 0,
            size_y: 0,
            min_x: 0.0,
            min_y: 0.0,
            per_pixel: 0.0,
            radius: 0,
            max_heat: 0.0,
            gradient: default_gradients(),
            flip_y: true,
            points: Vec::new(),
            heat_values: Vec::new(),
            color_values: Vec::new(),
        }
    }

    pub fn add_points(&mut self, points: Vec<f64>) {
        assert!(
            points.len() % 3 == 0,
            "Points length must be a multiple of 3"
        );

        for chunk in points.chunks(3) {
            let x = chunk[0];
            let y = chunk[1];
            let value = chunk[2];
            self.points.push(HeatPoint::new(x, y, value));
        }
    }

    pub fn calc_heatmap(&mut self) {
        self.update_heat_values();
        self.update_color_values();
    }

    pub fn tag1(&mut self) {
        self.update_heat_values();
    }

    pub fn tag2(&mut self) {
        self.update_color_values();
    }

    pub fn width(&self) -> u32 {
        self.size_x
    }

    pub fn height(&self) -> u32 {
        self.size_y
    }

    pub fn set_gradients(&mut self, gradient: Vec<RGBA>) {
        self.gradient = gradient;
    }

    pub fn set_max_heat(&mut self, max_heat: f64) {
        self.max_heat = max_heat;
    }

    pub fn set_radius(&mut self, radius: usize) {
        self.radius = radius as u32;
    }

    pub fn set_flip_y(&mut self, flip_y: bool) {
        self.flip_y = flip_y;
    }

    pub fn set_size(&mut self, size_x: usize, min_x: f64, min_y: f64, max_x: f64, max_y: f64) {
        let delta_x = max_x - min_x;
        let delta_y = max_y - min_y;
        let per_pixel = delta_x / size_x as f64;
        let size_y = ((delta_y / per_pixel) as f64).ceil() as usize;
        self.size_x = size_x as u32;
        self.size_y = size_y as u32;
        self.min_x = min_x;
        self.min_y = min_y;
        self.per_pixel = per_pixel;
        self.heat_values = vec![0.0; size_x * size_y];
        self.color_values = vec![0; size_x * size_y * 4];
    }

    pub fn reset(&mut self) {
        self.heat_values.fill(0.0);
    }

    pub fn color_values(&self) -> *const u8 {
        self.color_values.as_ptr()
    }
}
