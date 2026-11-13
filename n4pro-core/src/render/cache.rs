// 图像缓存：按签名缓存已编码的 PNG，避免重复渲染

use std::collections::HashMap;

#[derive(Default)]
pub struct ImageCache {
    map: HashMap<String, Vec<u8>>,
}

impl ImageCache {
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.map.get(key)
    }

    pub fn put(&mut self, key: String, png: Vec<u8>) -> Vec<u8> {
        self.map.insert(key, png.clone());
        png
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}
