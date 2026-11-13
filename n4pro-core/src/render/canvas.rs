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

    /// 填充矩形（不透明）
    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: [u8; 3]) {
        for yy in y..(y + h as i32) {
            for xx in x..(x + w as i32) {
                self.blend(xx, yy, [color[0], color[1], color[2], 255]);
            }
        }
    }

    /// 填充圆角矩形（半径 r）
    pub fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, r: i32, color: [u8; 3]) {
        let r = r.max(0);
        for yy in 0..h as i32 {
            for xx in 0..w as i32 {
                let cx = if xx < r {
                    r - xx
                } else if xx >= w as i32 - r {
                    xx - (w as i32 - r - 1)
                } else {
                    0
                };
                let cy = if yy < r {
                    r - yy
                } else if yy >= h as i32 - r {
                    yy - (h as i32 - r - 1)
                } else {
                    0
                };
                if cx * cx + cy * cy <= r * r {
                    self.blend(x + xx, y + yy, [color[0], color[1], color[2], 255]);
                }
            }
        }
    }

    /// 官方图标规则：等比缩放完整放入目标框并居中（小图放大、大图缩小，均不裁剪、不拉伸）
    pub fn draw_fit(&mut self, img: &RgbaImage, dx: i32, dy: i32, dw: u32, dh: u32) {
        if dw == 0 || dh == 0 || img.width() == 0 || img.height() == 0 {
            return;
        }
        let (iw, ih) = (img.width(), img.height());
        let scale = f64::min(dw as f64 / iw as f64, dh as f64 / ih as f64);
        let nw = (iw as f64 * scale).round().max(1.0) as u32;
        let nh = (ih as f64 * scale).round().max(1.0) as u32;
        let resized = if nw == iw && nh == ih {
            None
        } else {
            Some(image::imageops::resize(
                img,
                nw,
                nh,
                image::imageops::FilterType::Triangle,
            ))
        };
        let src = resized.as_ref().unwrap_or(img);
        let ox = dx + ((dw.saturating_sub(nw)) / 2) as i32;
        let oy = dy + ((dh.saturating_sub(nh)) / 2) as i32;
        for y in 0..nh {
            for x in 0..nw {
                let p = src.get_pixel(x, y);
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
        // 最高压缩：并行渲染已摊薄编码成本，换更小的分片数（更少 USB 帧）
        // 实测（同一页面）：Fast 846 帧 / Default 376 帧 / Best 375 帧，Best 总耗时最短
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut out,
            image::codecs::png::CompressionType::Best,
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

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn 等比放入且小图不放大() {
        // 176×112 → 112×71，垂直居中（上边距 20）
        let mut canvas = Canvas::new_black(112, 112);
        let img = RgbaImage::from_pixel(176, 112, Rgba([255, 0, 0, 255]));
        canvas.draw_fit(&img, 0, 0, 112, 112);
        let px = |x: u32, y: u32| {
            let i = ((y * 112 + x) * 4) as usize;
            [canvas.data[i], canvas.data[i + 1], canvas.data[i + 2]]
        };
        assert_eq!(px(56, 56), [255, 0, 0]);
        assert_eq!(px(0, 0), [0, 0, 0]);
        assert_eq!(px(56, 0), [0, 0, 0]);

        // 10×20 小图等比放大到 56×112 并居中
        let small = RgbaImage::from_pixel(10, 20, Rgba([0, 255, 0, 255]));
        let mut c2 = Canvas::new_black(112, 112);
        c2.draw_fit(&small, 0, 0, 112, 112);
        let px2 = |x: u32, y: u32| {
            let i = ((y * 112 + x) * 4) as usize;
            [c2.data[i], c2.data[i + 1], c2.data[i + 2]]
        };
        assert_eq!(px2(56, 56), [0, 255, 0]);
        assert_eq!(px2(28, 10), [0, 255, 0]);
        assert_eq!(px2(0, 0), [0, 0, 0]);
    }
}
