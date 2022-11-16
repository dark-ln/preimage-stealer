FROM rust:slim-buster as builder
WORKDIR /usr/src
RUN apt-get update 
RUN apt-get install -y build-essential cmake pkg-config libssl-dev libprotobuf-dev protobuf-compiler 
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
RUN apt-get update 
RUN apt-get install -y build-essential cmake pkg-config libssl-dev libprotobuf-dev protobuf-compiler 
RUN mkdir /data
COPY --from=builder /usr/local/cargo/bin/preimage-stealer /usr/local/bin/preimage-stealer
ENTRYPOINT preimage-stealer ${FLAGS}
