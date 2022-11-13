pub enum StorageType {
    Memory,
    Sled,
    Redis
}

trait Storage {
    fn set(&self, preimage: Vec<u8>, hash: Vec<u8>) -> ();

    fn get(&self, hash: Vec<u8>) -> Option<Vec<u8>>;
}