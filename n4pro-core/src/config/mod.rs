// 配置模块入口

pub mod ini;
pub mod manifest;
pub mod resolve;

pub use manifest::{load_page_actions, load_scene, ActionDef, Scene, StateDef};
pub use resolve::{load, AppConfig};
