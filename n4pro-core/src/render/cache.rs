// 图像缓存：按签名缓存已编码的 PNG，避免重复渲染

use std::collections::HashMap;

/// 缓存容量上限：超出后整体清空（图像可重新渲染，避免 Control Bar 拖动时无界增长）
const MAX_CACHE_ENTRIES: usize = 64;

pub struct ImageCache {
    map: HashMap<String, Vec<u8>>,
    max: usize,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            max: MAX_CACHE_ENTRIES,
        }
    }
}

impl ImageCache {
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.map.get(key)
    }

    pub fn put(&mut self, key: String, png: Vec<u8>) -> Vec<u8> {
        if self.map.len() >= self.max && !self.map.contains_key(&key) {
            self.map.clear();
        }
        self.map.insert(key, png.clone());
        png
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}
