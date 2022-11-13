# preimage-stealer

A utility to connect to claim HTLCs that we have already seen the preimage for.
This will automatically execute things like wormhole attacks for the user, allowing them to get free bitcoin.

Currently, this works by connecting to a lnd node and subscribes to the HTLC events to get preimages to save and the
HTLC interceptor to execute the theft.

## Running

In memory storage:

```
cargo run {LND_HOST} {LND_GRPC_PORT} {PATH_TO_LND_TLS_CERT} {PATH_TO_LND_ADMIN_MACAROON}
```

With sled db:
```
cargo run --features sled {LND_HOST} {LND_GRPC_PORT} {PATH_TO_LND_TLS_CERT} {PATH_TO_LND_ADMIN_MACAROON}
```

With redis db with localhost:
```
cargo run --features redis {LND_HOST} {LND_GRPC_PORT} {PATH_TO_LND_TLS_CERT} {PATH_TO_LND_ADMIN_MACAROON}
```

With redis db with specified url:
```
cargo run --features redis {LND_HOST} {LND_GRPC_PORT} {PATH_TO_LND_TLS_CERT} {PATH_TO_LND_ADMIN_MACAROON} {REDIS_URL}
```
