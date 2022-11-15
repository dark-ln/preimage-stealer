// This program accepts four arguments: host, port, cert file, macaroon file

mod memory;
#[cfg(feature = "redis")]
mod redis;
#[cfg(feature = "sled")]
mod sled;
mod storage;
mod subscribers;

use axum::response::Html;
use axum::routing::get;
use axum::{Extension, Router};
use dioxus::prelude::*;
use std::sync::Arc;

use crate::memory::MemoryStorage;
#[cfg(feature = "redis")]
use crate::redis::RedisStorage;
#[cfg(feature = "sled")]
use crate::sled::SledStorage;

use crate::storage::Storage;
use crate::subscribers::*;
use tokio::sync::Mutex;
use tokio::task::spawn;

#[tokio::main]
async fn main() {
    let mut args = std::env::args_os();
    args.next().expect("not even zeroth arg given");
    let host = args
        .next()
        .expect("missing arguments: host, port, cert file, macaroon file");
    let port = args
        .next()
        .expect("missing arguments: port, cert file, macaroon file");
    let cert_file = args
        .next()
        .expect("missing arguments: cert file, macaroon file");
    let macaroon_file = args.next().expect("missing argument: macaroon file");
    let host: String = host.into_string().expect("host is not UTF-8");
    let port: u32 = port
        .into_string()
        .expect("port is not UTF-8")
        .parse()
        .expect("port is not u32");
    let cert_file: String = cert_file.into_string().expect("cert_file is not UTF-8");
    let macaroon_file: String = macaroon_file
        .into_string()
        .expect("macaroon_file is not UTF-8");

    // Connecting to LND requires only host, port, cert file, macaroon file
    let mut client = tonic_openssl_lnd::connect(host, port, cert_file, macaroon_file)
        .await
        .expect("failed to connect");
    let client_router = client.router().clone();

    let storage = load_storage(args);

    // HTLC event stream part
    println!("starting htlc event subscription");
    let client_router_htlc_event = client_router.clone();
    let storage_htlc_event = storage.clone();

    spawn(async move {
        start_htlc_event_subscription(client_router_htlc_event, storage_htlc_event).await
    });

    println!("started htlc event subscription");

    // HTLC interceptor part
    println!("starting HTLC interception");
    let storage_htlc_interceptor = storage.clone();

    spawn(async move { start_htlc_interceptor(client_router, storage_htlc_interceptor).await });

    println!("started htlc event interception");

    let stolen = storage.lock().await.total_stolen();
    println!("current amount stolen: {stolen} msats");

    // TODO make port configurable
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on http://{}", addr);

    let router = Router::new()
        .route("/", get(index))
        .route("/stolen", get(get_stolen))
        .layer(Extension(storage));

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

// name this _args to keep clippy happy when it's unused
#[allow(unreachable_code)]
#[allow(dead_code)]
fn load_storage(args: std::env::ArgsOs) -> Arc<Mutex<dyn Storage + Send>> {
    #[cfg(feature = "sled")]
    {
        return Arc::new(Mutex::new(parse_sled_config(args)));
    }
    #[cfg(feature = "redis")]
    {
        return Arc::new(Mutex::new(parse_redis_config(args)));
    }

    Arc::new(Mutex::new(MemoryStorage::new()))
}

#[cfg(feature = "sled")]
fn parse_sled_config(mut args: std::env::ArgsOs) -> SledStorage {
    match args.next() {
        Some(arg) => {
            let str = arg.into_string().expect("Failed to parse sled config arg");
            SledStorage::new(str.as_str()).expect("Failed to create sled storage")
        }
        None => SledStorage::default(),
    }
}

#[cfg(feature = "redis")]
fn parse_redis_config(mut args: std::env::ArgsOs) -> RedisStorage {
    match args.next() {
        Some(arg) => {
            let str = arg.into_string().expect("Failed to parse redis config arg");
            RedisStorage::new(str.as_str()).expect("Failed to create redis storage")
        }
        None => RedisStorage::default(),
    }
}

async fn index(Extension(stolen): Extension<Arc<Mutex<dyn Storage + Send>>>) -> Html<String> {
    let amt = stolen.lock().await.total_stolen();

    Html(dioxus::ssr::render_lazy(rsx! {
            h1 { "Total stolen: {amt} msats" }
    }))
}

async fn get_stolen(Extension(stolen): Extension<Arc<Mutex<dyn Storage + Send>>>) -> String {
    let amt = stolen.lock().await.total_stolen();
    amt.to_string()
}
