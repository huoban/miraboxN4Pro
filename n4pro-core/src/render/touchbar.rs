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

/// Control Bar：条目在 x=25~775 安全区内横向滚动，半可见条目按边界裁切
pub fn render_control_bar(
    items: &[ActionDef],
    base_dir: &Path,
    scroll: i32,
) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(BAR_WIDTH, BAR_HEIGHT);
    // 官方抓包：条目只在安全区内显示，两侧 25px 留黑；滚动中的条目在边界处裁切
    canvas.set_clip(BTN_X0, 0, BAR_WIDTH as i32 - BTN_X0, BAR_HEIGHT as i32);

    for (i, def) in items.iter().enumerate() {
        let x = BTN_X0 + i as i32 * BTN_STEP - scroll;
        // 完全移出安全区的条目跳过；半可见条目照常绘制，由裁剪区裁切
        if x + BTN_W as i32 <= BTN_X0 || x >= BAR_WIDTH as i32 - BTN_X0 {
            continue;
        }
        // 按钮底
        canvas.fill_round_rect(x, BTN_Y, BTN_W, BTN_H, 10, [42, 42, 42]);
        if let Some(state) = def.states.first() {
            if let Some(img) = state.image.as_deref().and_then(|s| load_icon(base_dir, s)) {
                canvas.draw_fit(&img, x, BTN_Y, BTN_W, BTN_H);
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

    canvas.clear_clip();
    // 面板相对编辑器旋转 180° 安装：整条图需预旋转（与按键一致）
    canvas.rotate180();
    canvas.encode_png()
}

/// 纯黑整条（切回旋钮模式时用于清除 Control Bar 残留）
pub fn render_black_bar() -> Result<Vec<u8>, String> {
    Canvas::new_black(BAR_WIDTH, BAR_HEIGHT).encode_png()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 统计每一列是否存在非黑像素
    fn content_columns(png: &[u8]) -> Vec<bool> {
        let img = image::load_from_memory(png).expect("PNG 解码失败").to_rgba8();
        (0..img.width())
            .map(|x| {
                (0..img.height()).any(|y| {
                    let p = img.get_pixel(x, y);
                    p[0] as u32 + p[1] as u32 + p[2] as u32 > 12
                })
            })
            .collect()
    }

    /// 把列标记合并为连续区间
    fn runs(columns: &[bool]) -> Vec<(u32, u32)> {
        let mut out = Vec::new();
        let mut start = None;
        for (x, &v) in columns.iter().enumerate() {
            match (v, start) {
                (true, None) => start = Some(x as u32),
                (false, Some(s)) => {
                    out.push((s, x as u32 - 1));
                    start = None;
                }
                _ => {}
            }
        }
        if let Some(s) = start {
            out.push((s, columns.len() as u32 - 1));
        }
        out
    }

    #[test]
    fn 静止时显示五个完整条目() {
        let items = vec![ActionDef::default(); 7];
        let png = render_control_bar(&items, Path::new("."), 0).unwrap();
        let r = runs(&content_columns(&png));
        assert_eq!(
            r,
            vec![(25, 166), (177, 318), (329, 470), (481, 622), (633, 774)]
        );
    }

    #[test]
    fn 滚动时两侧条目按边界裁切() {
        let items = vec![ActionDef::default(); 7];
        let png = render_control_bar(&items, Path::new("."), 76).unwrap();
        let r = runs(&content_columns(&png));
        // 官方行为：首尾条目在 25 / 775 边界被裁切，共 6 段
        assert_eq!(r.len(), 6, "实际区间：{:?}", r);
        assert_eq!(r[0].0, 25);
        assert_eq!(r[5].1, 774);
    }
}
