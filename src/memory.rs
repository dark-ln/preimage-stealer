use crate::storage::Storage;
use std::any::Any;
use std::collections::HashMap;

pub struct MemoryStorage {
    map: HashMap<Vec<u8>, Box<dyn Any + Send + Sync>>,
}

unsafe impl Send for MemoryStorage {}

impl Storage for MemoryStorage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>) -> () {
        self.map.insert(hash, Box::new(preimage));
    }

    fn get(&mut self, hash: Vec<u8>) -> Option<Vec<u8>> {
        self.map
            .get(&hash)
            .map(|x| x.downcast_ref::<Vec<u8>>().cloned().unwrap())
    }
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            map: HashMap::new(),
        }
    }
}
