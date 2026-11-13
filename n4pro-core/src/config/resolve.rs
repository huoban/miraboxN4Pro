// 配置解析：ini → 当前设备/序列号 → 场景路径 → 亮度/模式
// 只读取，不写回任何配置。

use super::ini;
use std::path::{Path, PathBuf};

/// 运行所需的全局配置
pub struct AppConfig {
    pub ini_path: PathBuf,
    pub device: String,
    pub serial: String,
    pub profile_dir: PathBuf,
    pub brightness: u8,
    pub rotation: i32,
    pub knob_mode: bool,
}

pub fn load(profile_override: Option<&Path>) -> Result<AppConfig, String> {
    let ini_path = ini::default_ini_path();
    let text = std::fs::read_to_string(&ini_path)
        .map_err(|e| format!("无法读取配置文件 {}: {}", ini_path.display(), e))?;
    let ini = ini::Ini::parse(&text);

    let device = ini
        .get("StreamDockCurrentDevice", "Device")
        .unwrap_or("N4Pro")
        .to_string();
    let serial = ini
        .get("StreamDockCurrentDevice", &format!("{}\\SerialNumber", device))
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            ini.get_fuzzy("StreamDockCurrentDevice", "SerialNumber")
                .filter(|s| !s.trim().is_empty())
        })
        .unwrap_or("49490078391A")
        .to_string();
    let device_section = format!("{}{}DeviceConfig", device, serial);
    let profile_dir = if let Some(p) = profile_override {
        p.to_path_buf()
    } else {
        let path = ini
            .get(&device_section, "sdProfilePath")
            .ok_or_else(|| format!("配置段 [{}] 缺少 sdProfilePath", device_section))?;
        PathBuf::from(path)
    };

    let brightness = ini
        .get_fuzzy("DeviceBrightness", &format!("{}{}", device, serial))
        .filter(|v| !v.trim().is_empty())
        .and_then(|v| v.trim().parse::<u8>().ok())
        .unwrap_or(60)
        .min(100);
    let rotation = ini
        .get_fuzzy("RotationAngle", &format!("{}{}", device, serial))
        .filter(|v| !v.trim().is_empty())
        .and_then(|v| v.trim().parse::<i32>().ok())
        .unwrap_or(0);

    // 触摸条模式：以设备段的 enableTouchBarMode 为准（true=Control Bar，false=旋钮模式）
    let device_mode_section = format!("{}{}", device, serial);
    let mut control_bar = false;
    if let Some(v) = ini.get(&device_mode_section, "enableTouchBarMode") {
        control_bar = v.trim().eq_ignore_ascii_case("true");
    } else if let Some(v) = ini.get("MBoxN4", "enableTouchBarMode") {
        control_bar = v.trim().eq_ignore_ascii_case("true");
    }
    let knob_mode = !control_bar;

    Ok(AppConfig {
        ini_path,
        device,
        serial,
        profile_dir,
        brightness,
        rotation,
        knob_mode,
    })
}
