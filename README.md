# preimage-stealer

A utility to automatically claim HTLCs with revealed preimages.
This will automatically execute things like wormhole attacks for the user, allowing them to get free bitcoin.

Currently, this works by connecting to a lnd node and subscribes to the HTLC events to get preimages to save and the
HTLC interceptor to execute the theft.

## Running

In memory storage:

```
cargo run -- --host {LND_HOST} --port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With sled db:
```
cargo run-- --database sled --host {LND_HOST} --port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With sled db with custom path:
```
cargo run-- --db-path {DB_PATH} --host {LND_HOST} --port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With redis db with localhost:
```
cargo run -- --database redis --host {LND_HOST} --port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With redis db with specified url:
```
cargo run -- --redis-url {REDIS_URL} --host {LND_HOST} --port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON
```
