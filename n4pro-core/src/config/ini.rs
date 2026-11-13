// INI 解析：兼容 QSettings 生成的 StreamDockConfig.ini 格式
// 支持 [Section] 分组、Key=Value、值中的反斜杠转义（\\ -> \）

use std::collections::HashMap;

#[derive(Default)]
pub struct Ini {
    sections: HashMap<String, HashMap<String, String>>,
}

impl Ini {
    pub fn parse(text: &str) -> Self {
        let mut ini = Ini::default();
        let mut current = String::new();
        for raw in text.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                current = line[1..line.len() - 1].to_string();
                ini.sections.entry(current.clone()).or_default();
                continue;
            }
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = unescape_value(line[pos + 1..].trim());
                ini.sections
                    .entry(current.clone())
                    .or_default()
                    .insert(key, value);
            }
        }
        ini
    }

    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|m| m.get(key))
            .map(|s| s.as_str())
    }

    /// 若指定键不存在，尝试同段中“包含该键”的项（兼容 %5B 转义写法）
    pub fn get_fuzzy(&self, section: &str, key: &str) -> Option<&str> {
        if let Some(v) = self.get(section, key) {
            return Some(v);
        }
        let map = self.sections.get(section)?;
        for (k, v) in map {
            if k.ends_with(key) {
                return Some(v.as_str());
            }
        }
        None
    }

    pub fn section_exists(&self, section: &str) -> bool {
        self.sections.contains_key(section)
    }

    /// 返回全部段名（用于模糊匹配带转义的段名）
    pub fn section_names(&self) -> Vec<String> {
        self.sections.keys().cloned().collect()
    }
}

fn unescape_value(s: &str) -> String {
    s.replace("\\\\", "\\")
}

/// 读取 %APPDATA%\HotSpot\StreamDock\config\StreamDockConfig.ini 的默认路径
pub fn default_ini_path() -> std::path::PathBuf {
    let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(appdata)
        .join("HotSpot")
        .join("StreamDock")
        .join("config")
        .join("StreamDockConfig.ini")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 解析段与转义值() {
        let text = "[StreamDockCurrentDevice]\nDevice=N4Pro\nN4Pro\\SerialNumber=49490078391A\n\n[DeviceBrightness]\nN4Pro49490078391A=60\n\n[X]\nPath=C:\\\\Users\\\\a\n";
        let ini = Ini::parse(text);
        assert_eq!(ini.get("StreamDockCurrentDevice", "Device"), Some("N4Pro"));
        assert_eq!(
            ini.get("StreamDockCurrentDevice", "N4Pro\\SerialNumber"),
            Some("49490078391A")
        );
        assert_eq!(ini.get("DeviceBrightness", "N4Pro49490078391A"), Some("60"));
        assert_eq!(ini.get("X", "Path"), Some("C:\\Users\\a"));
    }
}
