// 文字渲染：使用 GDI DrawTextW 绘制到离屏 DIB，再按覆盖率合成到画布
// 支持字体、字号、粗体、下划线、颜色、上下居中对齐与末尾省略号
// 性能：DIB 只按文字矩形分配；HFONT 按参数缓存复用（永不删除，避免并行渲染下的竞态）

use crate::platform::win::*;
use crate::render::canvas::Canvas;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// 文本对齐方式
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Top,
    Middle,
    Bottom,
}

impl TextAlign {
    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "middle" | "center" => TextAlign::Middle,
            "bottom" => TextAlign::Bottom,
            _ => TextAlign::Top,
        }
    }
}

/// 字体缓存：键为（字体名, 字号, 粗体, 下划线），值为 HFONT
/// 进程存活期间不删除缓存字体，进程退出时由系统回收（数量有限，内存可忽略）
static FONTS: OnceLock<Mutex<HashMap<(String, i32, bool, bool), HANDLE>>> = OnceLock::new();

fn cached_font(family: &str, size: i32, bold: bool, underline: bool) -> HANDLE {
    let face = if family.trim().is_empty() {
        "Microsoft YaHei"
    } else {
        family
    };
    let key = (face.to_string(), size, bold, underline);
    let fonts = FONTS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = fonts.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(h) = guard.get(&key) {
        return *h;
    }
    let wide = to_wide(face);
    let font = unsafe {
        CreateFontW(
            -size,
            0,
            0,
            0,
            if bold { 700 } else { 400 },
            0,
            if underline { 1 } else { 0 },
            0,
            DEFAULT_CHARSET,
            0,
            0,
            ANTIALIASED_QUALITY,
            0,
            wide.as_ptr(),
        )
    };
    if font != 0 {
        guard.insert(key, font);
    }
    font
}

/// 在画布上绘制一行或多行文字（整幅画布区域）
pub fn draw_text(
    canvas: &mut Canvas,
    text: &str,
    family: &str,
    size_px: i32,
    bold: bool,
    underline: bool,
    color: [u8; 3],
    align: TextAlign,
    padding: i32,
) {
    let w = canvas.width as i32;
    let h = canvas.height as i32;
    draw_text_rect(
        canvas, text, family, size_px, bold, underline, color, align, padding, 0, 0, w, h,
    );
}

/// 在画布的指定矩形内绘制文字（用于触摸条分区）
pub fn draw_text_rect(
    canvas: &mut Canvas,
    text: &str,
    family: &str,
    size_px: i32,
    bold: bool,
    underline: bool,
    color: [u8; 3],
    align: TextAlign,
    padding: i32,
    area_x: i32,
    area_y: i32,
    area_w: i32,
    area_h: i32,
) {
    if text.is_empty() || canvas.width == 0 || canvas.height == 0 {
        return;
    }
    if area_w <= 0 || area_h <= 0 {
        return;
    }
    // 官方 Qt 字号按 pt 计算；实机对比后按一档 10% 回调（20pt ≈ 24px）
    let scaled = ((size_px as f32) * 1.2).round() as i32;
    let size = scaled.clamp(8, area_h.max(8));
    unsafe {
        let screen = GetDC(0);
        let mem = CreateCompatibleDC(screen);
        // DIB 只按文字矩形分配，不再按整幅画布（触摸条单按钮 142×66 而非 800×130）
        let mut bmi = BITMAPINFO::default();
        bmi.bmi_header.bi_size = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmi_header.bi_width = area_w;
        bmi.bmi_header.bi_height = -area_h; // 自上而下
        bmi.bmi_header.bi_planes = 1;
        bmi.bmi_header.bi_bit_count = 32;
        bmi.bmi_header.bi_compression = 0;
        let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
        let bitmap = CreateDIBSection(mem, &bmi, 0, &mut bits, 0, 0);
        if bitmap == 0 || bits.is_null() {
            DeleteDC(mem);
            ReleaseDC(0, screen);
            return;
        }
        let old = SelectObject(mem, bitmap);

        let font = cached_font(family, size, bold, underline);
        if font == 0 {
            SelectObject(mem, old);
            DeleteObject(bitmap);
            DeleteDC(mem);
            ReleaseDC(0, screen);
            return;
        }
        let old_font = SelectObject(mem, font);

        SetBkMode(mem, TRANSPARENT);
        let colorref = (color[0] as u32) | ((color[1] as u32) << 8) | ((color[2] as u32) << 16);
        SetTextColor(mem, colorref);

        let wide: Vec<u16> = text.encode_utf16().collect();
        // 局部坐标：DIB 即文字矩形本身
        let mut rect = RECT {
            left: padding,
            top: 0,
            right: area_w - padding,
            bottom: area_h,
        };
        let mut flags = DT_CENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOCLIP;
        flags |= match align {
            TextAlign::Top => DT_TOP,
            TextAlign::Middle => DT_VCENTER,
            TextAlign::Bottom => DT_BOTTOM,
        };
        if align == TextAlign::Top {
            rect.top = padding;
        }
        if align == TextAlign::Bottom {
            rect.bottom = area_h - padding;
        }
        DrawTextW(mem, wide.as_ptr(), wide.len() as i32, &mut rect, flags);

        // 读取 DIB 并合成（未绘制像素为 0，用最大通道近似覆盖率）
        let pixels = std::slice::from_raw_parts(bits as *const u8, (area_w * area_h * 4) as usize);
        for y in 0..area_h {
            for x in 0..area_w {
                let idx = ((y * area_w + x) * 4) as usize;
                let b = pixels[idx];
                let g = pixels[idx + 1];
                let r = pixels[idx + 2];
                let a = r.max(g).max(b);
                if a > 0 {
                    canvas.blend_text_pixel(area_x + x, area_y + y, [r, g, b, a]);
                }
            }
        }

        // 缓存字体不删除
        SelectObject(mem, old_font);
        SelectObject(mem, old);
        DeleteObject(bitmap);
        DeleteDC(mem);
        ReleaseDC(0, screen);
    }
}
