pub enum StorageType {
    Memory,
    Sled,
    Redis,
}

pub trait Storage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>) -> ();

    fn get(&self, hash: Vec<u8>) -> Option<Vec<u8>>;
}
