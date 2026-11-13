// 触摸条旋钮模式单区渲染：176×112，黑底 + 图标 + 文字，发送前旋转 180°
// 配置缺少图片时回退到官方内置图标（plugins 目录，均为 176×112）。

use crate::config::ActionDef;
use crate::render::canvas::Canvas;
use crate::render::key::load_icon;
use crate::render::text::{draw_text, TextAlign};
use image::RgbaImage;
use std::path::{Path, PathBuf};

pub const KNOB_WIDTH: u32 = 176;
pub const KNOB_HEIGHT: u32 = 112;

fn plugins_dir() -> PathBuf {
    let base = std::env::var("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Program Files (x86)"));
    base.join("StreamDock").join("plugins")
}

fn load_builtin(rel: &str) -> Option<RgbaImage> {
    let path = plugins_dir().join(rel);
    let bytes = std::fs::read(path).ok()?;
    image::load_from_memory(&bytes).ok().map(|d| d.to_rgba8())
}

/// 根据动作类型选择官方内置图标
fn builtin_icon(def: &ActionDef, page_number: u8) -> Option<RgbaImage> {
    let uuid = def.uuid.to_lowercase();
    if uuid.contains("profile.rotate") {
        return load_builtin(
            "com.hotspot.streamdock.profile.rotate.sdPlugin\\Images\\btn_switchProfile_knob_N4pro.png",
        );
    }
    if uuid.contains("system.volume") {
        return load_builtin(
            "com.hotspot.streamdock.system.multimedia.sdPlugin\\Images\\N4pro_btn_media_volume.png",
        );
    }
    if uuid.contains("multimedia") {
        let idx = def
            .settings
            .get("actionIdx")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let name = match idx {
            0 => "N4pro_btn_media_play_pause.png",
            1 => "N4pro_btn_media_next_track.png",
            2 => "N4pro_btn_media_prev_track.png",
            3 => "N4pro_btn_media_stop.png",
            4 => "N4pro_btn_media_mute.png",
            5 => "N4pro_btn_media_volume_up.png",
            6 => "N4pro_btn_media_volume_down.png",
            _ => "N4pro_btn_media_play_pause.png",
        };
        return load_builtin(&format!(
            "com.hotspot.streamdock.system.multimedia.sdPlugin\\Images\\{}",
            name
        ));
    }
    if uuid.contains("page.change") {
        let n = page_number.clamp(1, 10);
        return load_builtin(&format!(
            "com.hotspot.streamdock.page.sdPlugin\\Images\\N4pro_changePage{}.png",
            n
        ));
    }
    None
}

fn default_title(def: &ActionDef) -> Option<String> {
    let uuid = def.uuid.to_lowercase();
    if uuid.contains("multimedia") && !uuid.contains("system.volume") {
        return Some("Multimedia".to_string());
    }
    None
}

pub fn render_knob(
    action: Option<&ActionDef>,
    base_dir: &Path,
    state_idx: usize,
    page_number: u8,
) -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(KNOB_WIDTH, KNOB_HEIGHT);
    if let Some(def) = action {
        if !def.states.is_empty() {
            let state = &def.states[state_idx.min(def.states.len() - 1)];
            let image = state
                .image
                .as_deref()
                .and_then(|s| load_icon(base_dir, s))
                .or_else(|| builtin_icon(def, page_number));
            let has_image = image.is_some();
            if let Some(img) = image {
                canvas.draw_cover(&img, 0, 0, KNOB_WIDTH, KNOB_HEIGHT);
            }
            // 有图时不再叠加默认标题（官方内置图已含文字），避免重复
            let title = if state.title.is_some() {
                state.title.clone()
            } else if !has_image {
                default_title(def)
            } else {
                None
            };
            if state.show_title {
                if let Some(text) = title.as_deref() {
                    draw_text(
                        &mut canvas,
                        text,
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

/// 旋钮模式 4 个区域的黑底占位（缺图时避免残留上一帧）
pub fn render_knob_placeholder() -> Result<Vec<u8>, String> {
    let mut canvas = Canvas::new_black(KNOB_WIDTH, KNOB_HEIGHT);
    canvas.rotate180();
    canvas.encode_png()
}
