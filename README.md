# preimage-stealer

A utility to automatically claim HTLCs with revealed preimages.
This will automatically execute things like wormhole attacks for the user, allowing them to get free bitcoin.

Currently, this works by connecting to a lnd node and subscribes to the HTLC events to get preimages to save and the
HTLC interceptor to execute the theft.

## Running

In memory storage:

```
cargo run -- --lnd-host {LND_HOST} --lnd-port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With sled db:
```
cargo run-- --database sled --lnd-host {LND_HOST} --lnd-port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With sled db with custom path:
```
cargo run-- --db-path {DB_PATH} --lnd-host {LND_HOST} --lnd-port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With redis db with localhost:
```
cargo run -- --database redis --lnd-host {LND_HOST} --lnd-port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With redis db with specified url:
```
cargo run -- --redis-url {REDIS_URL} --lnd-host {LND_HOST} --lnd-port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON
```

### Docker

#### Build

This option is a local file mounted to a volume that persists between docker runs. Look at the config options above to see what you might like to use with docker.

First create a data directory and put your tls.cert and admin.macaroon files there. If you're running sled (enabled by default) then your sled db will persist there as well once the docker file runs.

```
docker run \
-e FLAGS='--lnd-host host.docker.internal --lnd-port 10009 --cert-file /data/tls.cert --macaroon-file /data/admin.macaroon --database sled --db-path /data/preimages' \
-p 3001:3000 \
-v /YOUR/TEMP/DIR/HERE:/data \
--add-host=host.docker.internal:host-gateway" \
dark-ln/preimage-stealer
```
