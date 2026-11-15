// 图像缓存：按签名缓存已编码的 PNG，避免重复渲染
// LRU 淘汰：容量上限内按最近使用排序，超出时淘汰最久未用项，避免拖动时整表清空

use std::collections::{HashMap, VecDeque};

/// 缓存容量上限：超出后淘汰最久未使用的条目（而非整体清空）
const MAX_CACHE_ENTRIES: usize = 64;

pub struct ImageCache {
    map: HashMap<String, Vec<u8>>,
    /// 插入/访问顺序（队尾为最近使用）
    order: VecDeque<String>,
    max: usize,
}

impl Default for ImageCache {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            max: MAX_CACHE_ENTRIES,
        }
    }
}

impl ImageCache {
    /// 命中时返回克隆的 PNG，并把该键移到最近使用位
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        if self.map.contains_key(key) {
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
                self.order.push_back(key.to_string());
            }
            self.map.get(key).cloned()
        } else {
            None
        }
    }

    pub fn put(&mut self, key: String, png: Vec<u8>) -> Vec<u8> {
        if self.map.contains_key(&key) {
            // 更新已有键：仅刷新访问顺序
            if let Some(pos) = self.order.iter().position(|k| *k == key) {
                self.order.remove(pos);
            }
        } else {
            // 达到上限时淘汰最久未使用项
            while self.map.len() >= self.max {
                if let Some(oldest) = self.order.pop_front() {
                    self.map.remove(&oldest);
                } else {
                    break;
                }
            }
        }
        self.order.push_back(key.clone());
        self.map.insert(key, png.clone());
        png
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }
}
