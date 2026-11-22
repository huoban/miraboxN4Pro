// 按键图像渲染：112×112，黑底 + 图标 + 文字，发送前旋转 180°

use crate::config::ActionDef;
use crate::render::canvas::Canvas;
use crate::render::text::{draw_text, TextAlign};
use image::RgbaImage;
use std::path::{Path, PathBuf};

pub const KEY_SIZE: u32 = 112;

/// 官方插件目录：%ProgramFiles(x86)%\StreamDock\plugins
pub fn plugins_dir() -> PathBuf {
    let base = std::env::var("ProgramFiles(x86)")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("C:\\Program Files (x86)"));
    base.join("StreamDock").join("plugins")
}

fn has_image_ext(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [".png", ".jpg", ".jpeg", ".gif"]
        .iter()
        .any(|e| lower.ends_with(e))
}

/// 配置里的图片名常省略扩展名（如 Images/btn_mouse_event），按官方格式补全
fn with_exts(path: PathBuf) -> Vec<PathBuf> {
    if has_image_ext(&path.to_string_lossy()) {
        return vec![path];
    }
    ["", ".png", ".jpg", ".jpeg", ".gif"]
        .iter()
        .map(|ext| {
            let mut raw = path.clone().into_os_string();
            raw.push(ext);
            PathBuf::from(raw)
        })
        .collect()
}

fn decode(path: &Path) -> Option<RgbaImage> {
    let bytes = std::fs::read(path).ok()?;
    image::load_from_memory(&bytes).ok().map(|d| d.to_rgba8())
}

fn try_load(path: PathBuf) -> Option<RgbaImage> {
    with_exts(path).into_iter().find_map(|p| decode(&p))
}

/// 读取并解码图标（支持 PNG/JPG/GIF 首帧）
/// 优先相对 manifest 目录，其次 Images 子目录
pub fn load_icon(base_dir: &Path, relative: &str) -> Option<RgbaImage> {
    load_icon_ex(base_dir, relative, None, false)
}

/// 图标解析（带官方插件回退）
/// - `plugin`：动作 UUID，用于定位 `plugins\<uuid>.sdPlugin` 自带图标
/// - `knob`：旋钮模式，优先匹配官方旋钮专用图（`N4pro_Knob_*` / `N4_Knob_*`）
pub fn load_icon_ex(
    base_dir: &Path,
    relative: &str,
    plugin: Option<&str>,
    knob: bool,
) -> Option<RgbaImage> {
    let rel = relative.trim().replace('/', "\\");
    if rel.is_empty() {
        return None;
    }
    let raw = Path::new(&rel);
    if raw.is_absolute() {
        return try_load(raw.to_path_buf());
    }

    // 1) 模板目录：相对根目录，以及官方存放图片的 Images 子目录
    if let Some(img) = try_load(base_dir.join(&rel)) {
        return Some(img);
    }
    if let Some(img) = try_load(base_dir.join("Images").join(&rel)) {
        return Some(img);
    }
    // "Images/xxx" 也可能直接位于模板根目录
    if let Some(stripped) = rel.strip_prefix("Images\\") {
        if let Some(img) = try_load(base_dir.join(stripped)) {
            return Some(img);
        }
    }

    // 2) 官方插件目录：默认状态图由插件自带
    let plugin_dir = plugins_dir().join(format!("{}.sdPlugin", plugin?));
    if !plugin_dir.is_dir() {
        return None;
    }
    let images = plugin_dir.join("Images");
    let stem = raw.file_stem()?.to_string_lossy().to_string();

    // 2a) 旋钮优先匹配旋钮专用图，官方约定：
    //     N4pro_Knob_* / N4pro_<stem>（176×112 设备图），以及 *_white 单色变体
    if knob {
        let base = stem
            .strip_prefix("btn_")
            .and_then(|s| Some(s.strip_prefix("custom_").unwrap_or(s)))
            .unwrap_or(&stem)
            .to_string();
        let mut names = vec![
            format!("N4pro_Knob_{}", stem),
            format!("N4pro_{}", stem),
            format!("N4_Knob_{}", stem),
            format!("N4_{}", stem),
            format!("{}_white@2x", stem),
            format!("{}_white", stem),
            format!("{}_white@2x", base),
            format!("{}_white", base),
        ];
        names.push(format!("{}_N4pro", stem));
        names.push(stem.clone());
        if let Some(img) = names.into_iter().find_map(|name| try_load(images.join(name))) {
            return Some(img);
        }
    }

    // 2b) 插件内置默认图（如 Images/btn_mouse_event → btn_mouse_event.png/.jpg/.gif）
    for base in [plugin_dir.clone(), images.clone()] {
        if let Some(img) = try_load(base.join(&rel)) {
            return Some(img);
        }
    }

    // 3) 按键设备变体：<stem>_N4pro（256×256 彩色图）
    try_load(images.join(format!("{}_N4pro", stem))).or_else(|| try_load(images.join(stem)))
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
            if let Some(img) = state
                .image
                .as_deref()
                .and_then(|s| load_icon_ex(base_dir, s, Some(&def.uuid), false))
            {
                canvas.draw_fit(&img, 0, 0, KEY_SIZE, KEY_SIZE);
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
