// 旋钮图像渲染：176×112，黑底 + 图标 + 文字，发送前旋转 180°

use crate::config::ActionDef;
use crate::render::canvas::Canvas;
use crate::render::key::load_icon;
use crate::render::text::{draw_text, TextAlign};
use std::path::Path;

pub const KNOB_WIDTH: u32 = 176;
pub const KNOB_HEIGHT: u32 = 112;

pub fn render_knob(
    action: Option<&ActionDef>,
    base_dir: &Path,
    state_idx: usize,
) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(KNOB_WIDTH, KNOB_HEIGHT);
    if let Some(def) = action {
        if !def.states.is_empty() {
            let state = &def.states[state_idx.min(def.states.len() - 1)];
            if let Some(img) = state.image.as_deref().and_then(|s| load_icon(base_dir, s)) {
                canvas.draw_cover(&img, 0, 0, KNOB_WIDTH, KNOB_HEIGHT);
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
                        4,
                    );
                }
            }
        }
    }
    canvas.rotate180();
    canvas.encode_png()
}
