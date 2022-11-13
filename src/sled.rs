use crate::storage::Storage;
use sled::Db;

pub struct SledStorage {
    db: Db,
}

impl Storage for SledStorage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>) -> () {
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
