use redis::{Commands, RedisResult};

use crate::storage::Storage;

#[derive(Clone)]
pub(crate) struct RedisStorage {
    pub client: redis::Client,
}

impl RedisStorage {
    pub fn new(url: &str) -> RedisResult<Self> {
        let client = redis::Client::open(url)?;
        Ok(RedisStorage { client })
    }
}

impl Default for RedisStorage {
    fn default() -> Self {
        RedisStorage::new("redis://127.0.0.1:6379").expect("Failed to create redis storage")
    }
}

impl Storage for RedisStorage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>) -> () {
        let mut conn = match self.client.get_connection() {
            Ok(conn) => conn,
            Err(_) => {
                println!("could not connect to redis to save preimage");
                return;
            }
        };

        match conn.set(hash, preimage) {
            Ok(()) => (),
            Err(e) => println!("could not save preimage: {}", e),
        }
    }

    fn get(&mut self, hash: Vec<u8>) -> Option<Vec<u8>> {
        let value: RedisResult<Vec<u8>> = self.client.get(hash);
        match value {
            Ok(v) => {
                if v.is_empty() {
                    None
                } else {
                    Some(v)
                }
            }
            Err(e) => {
                println!("redis error looking up preimage: {}", e);
                None
            }
        }
    }
}
