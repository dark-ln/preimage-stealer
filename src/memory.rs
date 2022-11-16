use crate::storage::Storage;
use std::any::Any;
use std::collections::HashMap;

pub struct MemoryStorage {
    map: HashMap<Vec<u8>, Box<dyn Any + Send + Sync>>,
    stolen: u64,
    watch_only_stolen: u64,
}

impl Storage for MemoryStorage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>) {
        self.map.insert(hash, Box::new(preimage));
    }

    fn get(&mut self, hash: Vec<u8>) -> Option<Vec<u8>> {
        self.map
            .get(&hash)
            .map(|x| x.downcast_ref::<Vec<u8>>().cloned().unwrap())
    }

    fn total_stolen(&mut self) -> u64 {
        self.stolen
    }

    fn add_stolen(&mut self, amt: u64) -> u64 {
        self.stolen += amt;
        self.stolen
    }

    fn total_stolen_watch_only(&mut self) -> u64 {
        self.watch_only_stolen
    }

    fn add_stolen_watch_only(&mut self, amt: u64) -> u64 {
        self.watch_only_stolen += amt;
        self.watch_only_stolen
    }
}

impl MemoryStorage {
    pub fn new() -> Self {
        MemoryStorage {
            map: HashMap::new(),
            stolen: 0,
            watch_only_stolen: 0,
        }
    }
}
