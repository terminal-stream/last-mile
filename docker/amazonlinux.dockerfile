## Build step
FROM amazonlinux:2023 as builder
RUN yum update -y
RUN yum install gcc openssl-devel -y
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
WORKDIR /usr/src/last-mile
COPY . .
RUN $HOME/.cargo/bin/cargo install --bin server --path ./server --target-dir /usr/local/terminal.stream/server

## Execution
FROM amazonlinux:2023
RUN yum update -y
RUN yum install openssl -y
COPY --from=builder /usr/local/terminal.stream/server/release/server /usr/local/bin/tslm-server
ENTRYPOINT ["tslm-server"]