//! Bounded, lossless CPU backing store. GPU textures keep their independent visible-only lifetime.
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

type Key = (i32, i32);
pub(super) struct PixelCache {
    images: HashMap<Key, Arc<Vec<u8>>>,
    order: VecDeque<Key>,
    bytes: usize,
    limit: usize,
}

impl PixelCache {
    pub fn new(limit: usize) -> Self {
        Self {
            images: HashMap::new(),
            order: VecDeque::new(),
            bytes: 0,
            limit,
        }
    }
    pub fn contains(&self, key: Key) -> bool {
        self.images.contains_key(&key)
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn limit(&self) -> usize {
        self.limit
    }
    pub fn get(&mut self, key: Key) -> Option<Arc<Vec<u8>>> {
        let image = self.images.get(&key)?.clone();
        self.order.retain(|k| *k != key);
        self.order.push_back(key);
        Some(image)
    }
    pub fn insert(&mut self, key: Key, bytes: Vec<u8>) -> Arc<Vec<u8>> {
        let image = Arc::new(bytes);
        if let Some(old) = self.images.remove(&key) {
            self.bytes -= old.len();
        }
        self.order.retain(|k| *k != key);
        if image.len() > self.limit || self.limit == 0 {
            return image;
        }
        while self.bytes > self.limit - image.len() {
            if let Some(old) = self
                .order
                .pop_front()
                .and_then(|key| self.images.remove(&key))
            {
                self.bytes -= old.len();
            }
        }
        self.bytes += image.len();
        self.images.insert(key, image.clone());
        self.order.push_back(key);
        image
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_pixels_survive_lru_eviction_and_budget_is_enforced() {
        let mut c = PixelCache::new(8);
        c.insert((1, 0), vec![0, 17, 255, 255]);
        c.insert((2, 0), vec![42; 4]);
        assert_eq!(&**c.get((1, 0)).unwrap(), &[0, 17, 255, 255]);
        c.insert((3, 0), vec![9; 4]);
        assert!(!c.contains((2, 0)));
        assert!(c.contains((1, 0)));
        assert_eq!(c.bytes(), 8);
        for i in 0..100 {
            c.insert((i, 1), vec![i as u8; 4]);
            assert!(c.bytes() <= c.limit());
        }
        assert_eq!(c.images.len(), 2);
    }
    #[test]
    fn replacement_oversize_and_disabled_cache_preserve_accounting() {
        let mut c = PixelCache::new(8);
        c.insert((1, 0), vec![1; 8]);
        c.insert((1, 0), vec![2; 4]);
        assert_eq!(c.bytes(), 4);
        assert_eq!(c.order.len(), 1);
        let large = c.insert((1, 0), vec![3; 9]);
        assert_eq!(large.len(), 9);
        assert_eq!(c.bytes(), 0);
        assert!(!c.contains((1, 0)));
        let mut disabled = PixelCache::new(0);
        assert_eq!(&**disabled.insert((0, 0), vec![7; 4]), &[7; 4]);
        assert!(disabled.get((0, 0)).is_none());
        assert_eq!(disabled.bytes(), 0);
    }
}
