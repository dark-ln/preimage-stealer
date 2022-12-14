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

    fn total_stolen_impl(&mut self, key: &[u8]) -> u64 {
        let value: RedisResult<u64> = self.client.get(key);
        match value {
            Ok(v) => v,
            Err(e) => {
                match e.kind() {
                    // this is common if there's no data to convert to int
                    redis::ErrorKind::TypeError => 0,
                    _ => {
                        println!("redis error looking up amount stolen: {}", e);
                        0
                    }
                }
            }
        }
    }

    fn add_stolen_impl(&mut self, key: &[u8], amt: u64) -> u64 {
        let current = self.total_stolen_impl(key);

        let mut conn = match self.client.get_connection() {
            Ok(conn) => conn,
            Err(_) => {
                println!("could not connect to redis to add amt stolen");
                return 0;
            }
        };

        let new_amt = current + amt;

        match conn.set(key, new_amt) {
            Ok(()) => new_amt,
            Err(e) => {
                println!("could not add amt stolen: {}", e);
                current
            }
        }
    }
}

impl Default for RedisStorage {
    fn default() -> Self {
        RedisStorage::new("redis://127.0.0.1:6379").expect("Failed to create redis storage")
    }
}

const STOLEN_KEY: &[u8] = "stolen".as_bytes();
const WATCH_ONLY_STOLEN_KEY: &[u8] = "watch-only-stolen".as_bytes();

impl Storage for RedisStorage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>) {
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
