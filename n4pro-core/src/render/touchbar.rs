// 触摸条渲染：800×130，Control Bar（5 槽）与旋钮模式（4 槽），发送前旋转 180°

use crate::config::ActionDef;
use crate::render::canvas::Canvas;
use crate::render::key::load_icon;
use crate::render::text::{draw_text_rect, TextAlign};
use std::collections::BTreeMap;
use std::path::Path;

pub const BAR_WIDTH: u32 = 800;
pub const BAR_HEIGHT: u32 = 130;

fn draw_slot(canvas: &mut Canvas, def: &ActionDef, base_dir: &Path, x: i32, w: u32) {
    if def.states.is_empty() {
        return;
    }
    let state = &def.states[0];
    if let Some(img) = state.image.as_deref().and_then(|s| load_icon(base_dir, s)) {
        canvas.draw_cover(&img, x + 2, 2, w - 4, 100);
    }
    if state.show_title {
        if let Some(title) = state.title.as_deref() {
            draw_text_rect(
                canvas,
                title,
                &state.font_family,
                state.font_size.min(18),
                state.bold,
                state.underline,
                state.title_color,
                TextAlign::Bottom,
                2,
                x,
                0,
                w as i32,
                BAR_HEIGHT as i32,
            );
        }
    }
}

/// Control Bar：条目 160px 一槽，可横向滚动
pub fn render_control_bar(items: &[ActionDef], base_dir: &Path, scroll: i32) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(BAR_WIDTH, BAR_HEIGHT);
    let slot = 160i32;
    for (i, def) in items.iter().enumerate() {
        let x = i as i32 * slot - scroll;
        if x + slot <= 0 || x >= BAR_WIDTH as i32 {
            continue;
        }
        draw_slot(&mut canvas, def, base_dir, x, slot as u32);
    }
    canvas.rotate180();
    canvas.encode_png()
}

/// 旋钮模式：4 个 200px 分区，取根配置 0,0~3,0
pub fn render_knob_mode(actions: &BTreeMap<String, ActionDef>, base_dir: &Path) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(BAR_WIDTH, BAR_HEIGHT);
    let slot = 200u32;
    for i in 0..4 {
        let key = format!("{},0", i);
        if let Some(def) = actions.get(&key) {
            draw_slot(&mut canvas, def, base_dir, (i as u32 * slot) as i32, slot);
        }
    }
    canvas.rotate180();
    canvas.encode_png()
}
