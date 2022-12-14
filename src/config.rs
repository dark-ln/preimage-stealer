use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(version, about, author)]
/// A utility to automatically claim HTLCs with revealed preimages
pub struct Config {
    #[clap(default_value_t = false, short, long)]
    /// Only monitors and tracks for hash re-use does not execute the exploit
    pub watch_only: bool,
    #[clap(default_value_t = String::from("127.0.0.1"), long)]
    /// Host of the GRPC server for lnd
    pub lnd_host: String,
    #[clap(default_value_t = 10009, long)]
    /// Port of the GRPC server for lnd
    pub lnd_port: u32,
    #[clap(default_value_t = String::from("mainnet"), short, long)]
    /// Network lnd is running on ["mainnet", "testnet", "signet, "simnet, "regtest"]
    pub network: String,
    #[clap(long)]
    /// Path to tls.cert file for lnd
    pub cert_file: Option<String>,
    #[clap(long)]
    /// Path to admin.macaroon file for lnd
    pub macaroon_file: Option<String>,
    #[clap(short, long)]
    /// Type of database ["memory", "sled", "redis"] [default: memory]
    pub database: Option<String>,
    #[clap(long)]
    /// Sled database path
    pub db_path: Option<String>,
    #[clap(long)]
    /// Redis server url
    pub redis_url: Option<String>,
    #[clap(default_value_t = String::from("0.0.0.0"), long)]
    /// Bind address for preimage-stealer's webserver
    pub bind: String,
    #[clap(default_value_t = 3000, long)]
    /// Port for preimage-stealer's webserver
    pub port: u16,
}

fn home_dir() -> String {
    let buf = home::home_dir().expect("Failed to get home directory");
    let str = format!("{}", buf.display());

    // to be safe remove possible trailing '/' and
    // we can manually add it to paths
    match str.strip_suffix('/') {
        Some(stripped) => stripped.to_string(),
        None => str,
    }
}

pub fn default_cert_file() -> String {
    format!("{}/.lnd/tls.cert", home_dir())
}

pub fn default_macaroon_file(network: String) -> String {
    format!(
        "{}/.lnd/data/chain/bitcoin/{}/admin.macaroon",
        home_dir(),
        network
    )
}
