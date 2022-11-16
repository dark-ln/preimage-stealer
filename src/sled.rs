use crate::storage::Storage;
use sled::{Db, Serialize};

pub struct SledStorage {
    db: Db,
}

const STOLEN_KEY: &[u8] = "stolen".as_bytes();
const WATCH_ONLY_STOLEN_KEY: &[u8] = "watch-only-stolen".as_bytes();

impl SledStorage {
    fn total_stolen_impl(&mut self, key: &[u8]) -> u64 {
        let current_opt = self.db.get(key).expect("Failed to read from sled db");

        match current_opt {
            Some(ivec) => {
                let vec = ivec.to_vec();
                let buf = &mut vec.as_slice();
                u64::deserialize(buf).expect("Error reading amount from database")
            }
            None => 0,
        }
    }

    fn add_stolen_impl(&mut self, key: &[u8], amt: u64) -> u64 {
        let current = self.total_stolen_impl(key);
        let new_amt = current + amt;

        self.db
            .insert(key, new_amt.serialize())
            .expect("Failed to write to sled db");
        new_amt
    }
}

impl Storage for SledStorage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>) {
        self.db
            .insert(hash, preimage)
            .expect("Failed to write to sled db");
    }

    fn get(&mut self, hash: Vec<u8>) -> Option<Vec<u8>> {
        self.db
            .get(hash)
            .expect("Failed to read from sled db")
            .map(|res| res.to_vec())
    }

    fn total_stolen(&mut self) -> u64 {
        self.total_stolen_impl(STOLEN_KEY)
    }

    fn add_stolen(&mut self, amt: u64) -> u64 {
        self.add_stolen_impl(STOLEN_KEY, amt)
    }

    fn total_stolen_watch_only(&mut self) -> u64 {
        self.total_stolen_impl(WATCH_ONLY_STOLEN_KEY)
    }

    fn add_stolen_watch_only(&mut self, amt: u64) -> u64 {
        self.add_stolen_impl(WATCH_ONLY_STOLEN_KEY, amt)
    }
}

impl Default for SledStorage {
    fn default() -> Self {
        SledStorage::new("preimages").expect("Failed to create sled storage")
    }
}

impl SledStorage {
    pub fn new(path: &str) -> Result<Self, sled::Error> {
        Ok(Self {
            db: sled::open(path)?,
        })
    }
}
