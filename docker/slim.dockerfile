## Build step
FROM rust:1.73.0-slim-buster as builder
RUN apt update
RUN apt install librust-openssl-dev -y
WORKDIR /usr/src/last-mile
COPY . .
RUN cargo install --bin server --path ./server --target-dir /usr/local/terminal.stream/server

## Execution
FROM debian:buster-slim
RUN apt update
RUN apt install openssl -y
COPY --from=builder /usr/local/terminal.stream/server/release/server /usr/local/bin/tslm-server
ENTRYPOINT ["/usr/local/bin/tslm-server"]