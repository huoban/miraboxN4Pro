// 场景配置数据结构与 manifest.json 解析
// 所有字段按需读取，缺失时使用默认值，保证兼容不同版本配置。

use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 单个状态的外观定义（对应 manifest 的 States[]）
#[derive(Clone)]
pub struct StateDef {
    pub image: Option<String>,
    pub title: Option<String>,
    pub show_title: bool,
    pub title_color: [u8; 3],
    pub font_family: String,
    pub font_size: i32,
    pub bold: bool,
    pub underline: bool,
    pub alignment: String, // top / middle / bottom
}

impl Default for StateDef {
    fn default() -> Self {
        Self {
            image: None,
            title: None,
            show_title: true,
            title_color: [255, 255, 255],
            font_family: "Microsoft YaHei".to_string(),
            font_size: 20,
            bold: false,
            underline: false,
            alignment: "top".to_string(),
        }
    }
}

/// 一个可执行动作（按键/旋钮/触摸条条目）
#[derive(Clone, Default)]
pub struct ActionDef {
    pub uuid: String,
    pub name: String,
    pub controller: String,
    pub settings: Value,
    pub states: Vec<StateDef>,
    pub state_index: usize,
}

impl ActionDef {
    pub fn state(&self) -> StateDef {
        let idx = self.state_index.min(self.states.len().saturating_sub(1));
        self.states.get(idx).cloned().unwrap_or_default()
    }
}

/// 场景（根档案）
pub struct Scene {
    pub dir: PathBuf,
    pub name: String,
    pub current_page: Option<String>,
    pub pages: Vec<String>,
    pub root_actions: BTreeMap<String, ActionDef>,
    pub touchbar: Vec<ActionDef>,
    /// 触摸条滚动偏移（官方 Settings.widgetCurrX，通常为负值）
    pub touchbar_curr_x: i32,
}

#[derive(Deserialize)]
struct RawManifest {
    #[serde(rename = "Name", default)]
    name: String,
    #[serde(rename = "Pages", default)]
    pages: Option<RawPages>,
    #[serde(rename = "Actions", default)]
    actions: BTreeMap<String, Value>,
}

#[derive(Deserialize)]
struct RawPages {
    #[serde(rename = "Current", default)]
    current: String,
    #[serde(rename = "Pages", default)]
    pages: Vec<String>,
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("读取失败 {}: {}", path.display(), e))?;
    serde_json::from_str(&text).map_err(|e| format!("JSON 解析失败 {}: {}", path.display(), e))
}

fn parse_state(v: &Value) -> StateDef {
    StateDef {
        image: v.get("Image").and_then(|x| x.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        title: v.get("Title").and_then(|x| x.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        show_title: v.get("ShowTitle").and_then(|x| x.as_bool()).unwrap_or(true),
        title_color: parse_color(v.get("TitleColor").and_then(|x| x.as_str()).unwrap_or("#ffffff")),
        font_family: v
            .get("FontFamily")
            .and_then(|x| x.as_str())
            .unwrap_or("Microsoft YaHei")
            .to_string(),
        font_size: v.get("FontSize").and_then(|x| x.as_f64()).unwrap_or(20.0) as i32,
        bold: v
            .get("FontStyle")
            .and_then(|x| x.as_str())
            .map(|s| s.to_lowercase().contains("bold"))
            .unwrap_or(false),
        underline: v.get("FontUnderline").and_then(|x| x.as_bool()).unwrap_or(false),
        alignment: v
            .get("TitleAlignment")
            .and_then(|x| x.as_str())
            .unwrap_or("top")
            .to_lowercase(),
    }
}

/// 颜色解析：支持 "#RRGGBB" 与 Qt 的 "#AARRGGBB"
pub fn parse_color(text: &str) -> [u8; 3] {
    let s = text.trim().trim_start_matches('#');
    let hex = if s.len() == 8 { &s[2..] } else { s };
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        [r, g, b]
    } else {
        [255, 255, 255]
    }
}

/// 解析单个动作对象
pub fn parse_action(v: &Value) -> ActionDef {
    let states = v
        .get("States")
        .and_then(|x| x.as_array())
        .map(|arr| arr.iter().map(parse_state).collect())
        .unwrap_or_default();
    let state_index = v.get("State").and_then(|x| x.as_u64()).unwrap_or(0) as usize;
    ActionDef {
        uuid: v.get("UUID").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        name: v.get("Name").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        controller: v
            .get("Controller")
            .and_then(|x| x.as_str())
            .unwrap_or("Keypad")
            .to_string(),
        settings: v.get("Settings").cloned().unwrap_or(Value::Null),
        states,
        state_index,
    }
}

/// 加载场景（根 manifest）
pub fn load_scene(dir: &Path) -> Result<Scene, String> {
    let path = dir.join("manifest.json");
    let json = read_json(&path)?;
    let raw: RawManifest = serde_json::from_value(json).map_err(|e| format!("场景结构解析失败: {}", e))?;
    let mut root_actions = BTreeMap::new();
    let mut touchbar = Vec::new();
    let mut touchbar_curr_x = 0i32;
    for (key, value) in &raw.actions {
        if key == "touchbar" {
            if let Some(list) = value.get("Actions").and_then(|x| x.as_array()) {
                touchbar = list.iter().map(parse_action).collect();
            }
            if let Some(v) = value
                .get("Settings")
                .and_then(|s| s.get("widgetCurrX"))
                .and_then(|v| v.as_i64())
            {
                touchbar_curr_x = v as i32;
            }
        } else {
            root_actions.insert(key.clone(), parse_action(value));
        }
    }
    Ok(Scene {
        dir: dir.to_path_buf(),
        name: raw.name,
        current_page: raw.pages.as_ref().map(|p| p.current.clone()).filter(|s| !s.is_empty()),
        pages: raw.pages.map(|p| p.pages).unwrap_or_default(),
        root_actions,
        touchbar,
        touchbar_curr_x,
    })
}

/// 加载子页面动作
pub fn load_page_actions(dir: &Path) -> Result<BTreeMap<String, ActionDef>, String> {
    let path = dir.join("manifest.json");
    let json = read_json(&path)?;
    let out = if let Some(actions) = json.get("Actions").and_then(|x| x.as_object()) {
        let mut map = BTreeMap::new();
        for (key, value) in actions {
            if key == "touchbar" {
                continue;
            }
            map.insert(key.clone(), parse_action(value));
        }
        map
    } else {
        BTreeMap::new()
    };
    Ok(out)
}
