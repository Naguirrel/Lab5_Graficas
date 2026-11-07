use crate::math::{pack, rgb};
use image::{ImageBuffer, Rgb};
use std::path::Path;

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    depth: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            depth: vec![f32::NEG_INFINITY; width * height],
        }
    }

    pub fn clear_space(&mut self) {
        self.depth.fill(f32::NEG_INFINITY);
        for y in 0..self.height {
            let t = y as f32 / self.height as f32;
            let color = pack(rgb(5, 8, 18) * (1.0 - t) + rgb(12, 15, 29) * t);
            self.pixels[y * self.width..(y + 1) * self.width].fill(color);
        }
        // Estrellas del fondo como puntos calculados; tampoco usan texturas.
        let mut seed = 713_u32;
        for _ in 0..self.width * self.height / 1300 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let x = seed as usize % self.width;
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let y = seed as usize % self.height;
            let brightness = 55 + (seed >> 24) as u8 / 2;
            self.pixels[y * self.width + x] =
                pack(rgb(brightness, brightness, brightness.saturating_add(20)));
        }
    }

    pub fn visible(&self, index: usize, depth: f32) -> bool {
        depth > self.depth[index]
    }

    pub fn point(&mut self, index: usize, depth: f32, color: u32) {
        if self.visible(index, depth) {
            self.pixels[index] = color;
            self.depth[index] = depth;
        }
    }

    pub fn save_png(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let image = ImageBuffer::from_fn(self.width as u32, self.height as u32, |x, y| {
            let c = self.pixels[y as usize * self.width + x as usize];
            Rgb([(c >> 16) as u8, (c >> 8) as u8, c as u8])
        });
        image.save(path)?;
        Ok(())
    }
}
