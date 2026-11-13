// 图像缓存：按签名缓存已编码的 PNG，避免重复渲染
// LRU 上限 48 项；滚动类条目（volatile）仅保留最近 2 项，防止长拖滑动撑爆内存

use std::collections::{HashMap, VecDeque};

const MAX_ENTRIES: usize = 48;
const MAX_VOLATILE: usize = 2;

#[derive(Default)]
pub struct ImageCache {
    map: HashMap<String, Vec<u8>>,
    /// 插入顺序，用于 LRU 淘汰
    order: VecDeque<String>,
    /// 滚动类条目（Control Bar 拖动），仅保留最近 MAX_VOLATILE 项
    volatile: VecDeque<String>,
}

impl ImageCache {
    pub fn get(&self, key: &str) -> Option<&Vec<u8>> {
        self.map.get(key)
    }

    /// 写入缓存；volatile=true 表示滚动类条目（只保留最近 2 项）
    pub fn put(&mut self, key: String, png: Vec<u8>, volatile: bool) -> Vec<u8> {
        if !self.map.contains_key(&key) {
            self.order.push_back(key.clone());
        }
        if volatile {
            while self.volatile.len() >= MAX_VOLATILE {
                if let Some(old) = self.volatile.pop_front() {
                    self.map.remove(&old);
                    self.order.retain(|k| k != &old);
                }
            }
            self.volatile.push_back(key.clone());
        }
        self.map.insert(key.clone(), png.clone());
        while self.map.len() > MAX_ENTRIES {
            let Some(old) = self.order.pop_front() else { break };
            if self.map.remove(&old).is_none() {
                continue;
            }
            self.volatile.retain(|k| k != &old);
        }
        png
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
        self.volatile.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 普通条目超限按插入顺序淘汰() {
        let mut c = ImageCache::default();
        for i in 0..(MAX_ENTRIES + 5) {
            c.put(format!("k{}", i), vec![i as u8], false);
        }
        assert!(c.map.len() <= MAX_ENTRIES);
        assert!(c.get("k0").is_none());
        assert!(c.get(&format!("k{}", MAX_ENTRIES + 4)).is_some());
    }

    #[test]
    fn 滚动条目只保留最近两项() {
        let mut c = ImageCache::default();
        for i in 0..10 {
            c.put(format!("bar|{}", i), vec![i as u8], true);
        }
        let left: Vec<&String> = c.volatile.iter().collect();
        assert_eq!(left.len(), 2);
        assert!(c.get("bar|0").is_none());
        assert!(c.get("bar|8").is_some());
        assert!(c.get("bar|9").is_some());
    }
}
