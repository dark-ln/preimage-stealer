pub trait Storage {
    fn set(&mut self, preimage: Vec<u8>, hash: Vec<u8>);

    fn get(&mut self, hash: Vec<u8>) -> Option<Vec<u8>>;

    /// Returns the total amount stolen in msats
    fn total_stolen(&mut self) -> u64;

    /// Returns the new total amount stolen in msats
    fn add_stolen(&mut self, amt: u64) -> u64;

    /// Returns the total amount that could have been stolen
    /// while in watch-only mode
    fn total_stolen_watch_only(&mut self) -> u64;

    /// Returns the new total amount that could have been stolen
    /// while in watch-only mode
    fn add_stolen_watch_only(&mut self, amt: u64) -> u64;
}
