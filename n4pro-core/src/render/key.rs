// 按键图像渲染：112×112，黑底 + 图标 + 文字，发送前旋转 180°

use crate::config::ActionDef;
use crate::render::canvas::Canvas;
use crate::render::text::{draw_text, TextAlign};
use image::RgbaImage;
use std::path::Path;

pub const KEY_SIZE: u32 = 112;

/// 读取并解码图标（支持 PNG/JPG/GIF 首帧）
/// 优先相对 manifest 目录，其次 Images 子目录
pub fn load_icon(base_dir: &Path, relative: &str) -> Option<RgbaImage> {
    let rel = relative.replace('/', "\\");
    let candidates = [base_dir.join(&rel), base_dir.join("Images").join(&rel)];
    for path in candidates {
        if let Ok(bytes) = std::fs::read(path) {
            if let Ok(img) = image::load_from_memory(&bytes) {
                return Some(img.to_rgba8());
            }
        }
    }
    None
}

/// 渲染纯文字标定图（用于槽位映射测试）
pub fn render_label(text: &str, width: u32, height: u32, size: i32) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(width, height);
    draw_text(
        &mut canvas,
        text,
        "Microsoft YaHei",
        size,
        true,
        false,
        [255, 255, 0],
        TextAlign::Middle,
        2,
    );
    canvas.rotate180();
    canvas.encode_png()
}

/// 渲染单个按键（state_idx 为显示状态）
pub fn render_key(action: Option<&ActionDef>, base_dir: &Path, state_idx: usize) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(KEY_SIZE, KEY_SIZE);
    if let Some(def) = action {
        if !def.states.is_empty() {
            let state = &def.states[state_idx.min(def.states.len() - 1)];
            if let Some(img) = state.image.as_deref().and_then(|s| load_icon(base_dir, s)) {
                canvas.draw_cover(&img, 0, 0, KEY_SIZE, KEY_SIZE);
            }
            if state.show_title {
                if let Some(title) = state.title.as_deref() {
                    draw_text(
                        &mut canvas,
                        title,
                        &state.font_family,
                        state.font_size,
                        state.bold,
                        state.underline,
                        state.title_color,
                        TextAlign::parse(&state.alignment),
                        1,
                    );
                }
            }
        }
    }
    canvas.rotate180();
    canvas.encode_png()
}
