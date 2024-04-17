## Build step
FROM terminal.stream/build-amazonlinux-openssl-rust:0.1 as builder
WORKDIR /usr/src/terminal.stream/last-mile
COPY . .
RUN $HOME/.cargo/bin/cargo build --release --target x86_64-unknown-linux-gnu

## Execution
FROM terminal.stream/exec-amazonlinux-openssl:0.1
COPY --from=builder /usr/src/terminal.stream/last-mile/target/x86_64-unknown-linux-gnu/release/server /usr/local/bin/tslm-server
CMD mkdir -p /usr/share/terminal.stream/last-mile/log
ENTRYPOINT ["tslm-server"]