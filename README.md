# preimage-stealer

A utility to automatically claim Lightning Network HTLCs with revealed preimages.

This will automatically resolve HTLCs with preimages that your node has seen before. If you have multiple nodes, you can connect them to the same Redis instance to have a wider reach of known preimages.

This will happen on Lightning if an invoice is paid and then another node attempts to make a payment to the same invoice. If the second node routes through any of the same nodes as the first payment, any of the routing nodes may steal the funds before they get to the destination.

Future iterations of this will execute things like [wormhole](https://eprint.iacr.org/2020/303.pdf) attacks for the user, allowing them siphon more routing fees than they should have gotten when connecting multiple routing nodes. Another potential feature could be to have a globally accessible store where preimages could be bought by another peer.

Currently, this works by connecting to a lnd node and subscribes to the HTLC events to get preimages to save and the HTLC interceptor to execute the theft. You can also run this in a watch only mode to analyze how much money you could have stolen. The results are viewable via a webpage that this program also hosts (default location: `http://0.0.0.0:3000/`).

![stolen.png](./docs/images/stolen.png)

![watch.png](./docs/images/watch.png)

## Running

Default in memory storage:

```
cargo run -- --lnd-host {LND_HOST} --lnd-port {LND_GRPC_PORT} --cert-file {PATH_TO_LND_TLS_CERT} --macaroon-file {PATH_TO_LND_ADMIN_MACAROON}
```

With sled db:
```
cargo run -- --database sled  ...
```

With sled db with custom path:
```
cargo run -- --database sled --db-path {DB_PATH}  ...
```

With redis db with localhost:
```
cargo run -- --database redis  ...
```

With redis db with specified url:
```
cargo run -- --database redis --redis-url {REDIS_URL} ...
```

With a custom URL for the results webpage:
```
cargo run -- --bind 0.0.0.0 --port 3000 ...
```

View all options with `cargo run -- --help`. Current options:
```
  -w, --watch-only                     Only monitors and tracks for hash re-use does not execute the exploit
      --lnd-host <LND_HOST>            Host of the GRPC server for lnd [default: 127.0.0.1]
      --lnd-port <LND_PORT>            Port of the GRPC server for lnd [default: 10009]
  -n, --network <NETWORK>              Network lnd is running on ["mainnet", "testnet", "signet, "simnet, "regtest"] [default: mainnet]
      --cert-file <CERT_FILE>          Path to tls.cert file for lnd
      --macaroon-file <MACAROON_FILE>  Path to admin.macaroon file for lnd
  -d, --database <DATABASE>            Type of database ["memory", "sled", "redis"] [default: memory]
      --db-path <DB_PATH>              Sled database path
      --redis-url <REDIS_URL>          Redis server url
      --bind <BIND>                    Bind address for preimage-stealer's webserver [default: 0.0.0.0]
      --port <PORT>                    Port for preimage-stealer's webserver [default: 3000]
  -h, --help                           Print help information
  -V, --version                        Print version information
```

### Docker

#### Pull

We have the docker image hosted on github if you like to pull it down.

```
docker pull ghcr.io/dark-ln/preimage-stealer:latest
```

#### Build

If you would like to build it instead of using the hosted image:
```
docker build -t dark-ln/preimage-stealer .
```

#### Run

After you pull down or build the image, you may run it with these instructions and pass in your own config flags.

This option is a local file mounted to a volume that persists between docker runs. Look at the config options above to see what you might like to use with docker.

First create a data directory and put your tls.cert and admin.macaroon files there. If you're running sled (enabled by default) then your sled db will persist there as well once the docker file runs.

You may have to configure the `--lnd-host` and other docker networking properties like `--add-host` based on your computer and networking set up. If you're connecting to a remote node that is accessible on the internet then you shouldn't need some of the advanced docker networking options.

```
docker run \
-e FLAGS='--lnd-host host.docker.internal --lnd-port 10009 --cert-file /data/tls.cert --macaroon-file /data/admin.macaroon --database sled --db-path /data/preimages' \
-p 3001:3000 \
-v /YOUR/TEMP/DIR/HERE:/data \
--add-host=host.docker.internal:host-gateway" \
dark-ln/preimage-stealer
```

## Disclaimer

We are not responsible for loss of funds.
