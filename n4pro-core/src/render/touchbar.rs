// 触摸条渲染：Control Bar（800×130，5 按钮 + 箭头），
// 以及旋钮模式的 4 个 176×112 分区（由 knob.rs 负责单区）。
// 实测：Control Bar 由 BGPIC 下发且不旋转。

use crate::config::ActionDef;
use crate::render::canvas::Canvas;
use crate::render::key::load_icon;
use crate::render::text::{draw_text_rect, TextAlign};
use std::path::Path;

pub const BAR_WIDTH: u32 = 800;
pub const BAR_HEIGHT: u32 = 130;
pub const BTN_W: u32 = 142;
pub const BTN_H: u32 = 66;
pub const BTN_Y: i32 = 42;
pub const BTN_STEP: i32 = 152;
pub const BTN_X0: i32 = 25;

/// Control Bar：5 个可见按钮 + 横向滚动 + 两侧箭头
pub fn render_control_bar(
    items: &[ActionDef],
    base_dir: &Path,
    scroll: i32,
    show_left: bool,
    show_right: bool,
) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(BAR_WIDTH, BAR_HEIGHT);
    // 深色容器
    canvas.fill_round_rect(8, 10, BAR_WIDTH - 16, BAR_HEIGHT - 20, 10, [26, 26, 26]);

    for (i, def) in items.iter().enumerate() {
        let x = BTN_X0 + i as i32 * BTN_STEP - scroll;
        // 只绘制完整可见的按钮，避免与两侧箭头重叠
        if x < 20 || x + BTN_W as i32 > BAR_WIDTH as i32 - 20 {
            continue;
        }
        // 按钮底
        canvas.fill_round_rect(x, BTN_Y, BTN_W, BTN_H, 10, [42, 42, 42]);
        if let Some(state) = def.states.first() {
            if let Some(img) = state.image.as_deref().and_then(|s| load_icon(base_dir, s)) {
                canvas.draw_cover(&img, x, BTN_Y, BTN_W, BTN_H);
            }
            // 文字层：官方编辑器在按钮图上叠加 States[].Title
            if state.show_title {
                if let Some(title) = state.title.as_deref() {
                    draw_text_rect(
                        &mut canvas,
                        title,
                        &state.font_family,
                        state.font_size.min(18),
                        state.bold,
                        state.underline,
                        state.title_color,
                        TextAlign::parse(&state.alignment),
                        2,
                        x,
                        BTN_Y,
                        BTN_W as i32,
                        BTN_H as i32,
                    );
                }
            }
        }
    }

    if show_left {
        draw_text_rect(
            &mut canvas,
            "◀",
            "Microsoft YaHei",
            22,
            true,
            false,
            [200, 200, 200],
            TextAlign::Middle,
            0,
            8,
            0,
            20,
            BAR_HEIGHT as i32,
        );
    }
    if show_right {
        draw_text_rect(
            &mut canvas,
            "▶",
            "Microsoft YaHei",
            22,
            true,
            false,
            [200, 200, 200],
            TextAlign::Middle,
            0,
            BAR_WIDTH as i32 - 28,
            0,
            20,
            BAR_HEIGHT as i32,
        );
    }

    // 面板相对编辑器旋转 180° 安装：整条图需预旋转（与按键一致）
    canvas.rotate180();
    canvas.encode_png()
}

/// 纯黑整条（切回旋钮模式时用于清除 Control Bar 残留）
pub fn render_black_bar() -> Result<Vec<u8>, String> {
    Canvas::new_black(BAR_WIDTH, BAR_HEIGHT).encode_png()
}
