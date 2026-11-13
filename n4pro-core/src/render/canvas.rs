// RGBA 画布：黑底、图像合成、180° 旋转、PNG 编码

use image::{ImageEncoder, RgbaImage};

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl Canvas {
    pub fn new_black(width: u32, height: u32) -> Self {
        let mut data = vec![0u8; (width * height * 4) as usize];
        // 纯黑不透明
        for px in data.chunks_exact_mut(4) {
            px[3] = 255;
        }
        Canvas { width, height, data }
    }

    #[inline]
    fn blend(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let a = color[3] as u32;
        if a == 0 {
            return;
        }
        let idx = ((y as u32 * self.width + x as u32) * 4) as usize;
        let inv = 255 - a;
        for c in 0..3 {
            self.data[idx + c] =
                ((color[c] as u32 * a + self.data[idx + c] as u32 * inv) / 255) as u8;
        }
        self.data[idx + 3] = 255;
    }

    /// 供文字层调用的像素合成
    pub fn blend_text_pixel(&mut self, x: i32, y: i32, color: [u8; 4]) {
        self.blend(x, y, color);
    }

    /// 等比缩放并填满目标区域（裁剪居中）
    pub fn draw_cover(&mut self, img: &RgbaImage, dx: i32, dy: i32, dw: u32, dh: u32) {
        if dw == 0 || dh == 0 || img.width() == 0 || img.height() == 0 {
            return;
        }
        let scale = f64::max(dw as f64 / img.width() as f64, dh as f64 / img.height() as f64);
        let nw = (img.width() as f64 * scale).round().max(1.0) as u32;
        let nh = (img.height() as f64 * scale).round().max(1.0) as u32;
        let resized = image::imageops::resize(img, nw, nh, image::imageops::FilterType::Triangle);
        let ox = nw.saturating_sub(dw) / 2;
        let oy = nh.saturating_sub(dh) / 2;
        for y in 0..dh {
            for x in 0..dw {
                let p = resized.get_pixel(ox + x, oy + y);
                self.blend(dx + x as i32, dy + y as i32, [p[0], p[1], p[2], p[3]]);
            }
        }
    }

    /// 等比缩放并完整放入目标区域（居中，留黑边）
    pub fn draw_contain(&mut self, img: &RgbaImage, dx: i32, dy: i32, dw: u32, dh: u32) {
        if dw == 0 || dh == 0 || img.width() == 0 || img.height() == 0 {
            return;
        }
        let scale = f64::min(dw as f64 / img.width() as f64, dh as f64 / img.height() as f64);
        let nw = (img.width() as f64 * scale).round().max(1.0) as u32;
        let nh = (img.height() as f64 * scale).round().max(1.0) as u32;
        let resized = image::imageops::resize(img, nw, nh, image::imageops::FilterType::Triangle);
        let ox = dx + ((dw - nw) / 2) as i32;
        let oy = dy + ((dh - nh) / 2) as i32;
        for y in 0..nh {
            for x in 0..nw {
                let p = resized.get_pixel(x, y);
                self.blend(ox + x as i32, oy + y as i32, [p[0], p[1], p[2], p[3]]);
            }
        }
    }

    /// 原地旋转 180°
    pub fn rotate180(&mut self) {
        let w = self.width as usize;
        let h = self.height as usize;
        let mut out = vec![0u8; self.data.len()];
        for y in 0..h {
            for x in 0..w {
                let src = (y * w + x) * 4;
                let dst = ((h - 1 - y) * w + (w - 1 - x)) * 4;
                out[dst..dst + 4].copy_from_slice(&self.data[src..src + 4]);
            }
        }
        self.data = out;
    }

    pub fn encode_png(&self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut out,
            image::codecs::png::CompressionType::Default,
            image::codecs::png::FilterType::NoFilter,
        );
        encoder
            .write_image(
                &self.data,
                self.width,
                self.height,
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| format!("PNG 编码失败: {}", e))?;
        Ok(out)
    }
}
