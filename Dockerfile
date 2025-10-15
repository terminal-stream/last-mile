# Build stage using Rust nightly (required for edition 2024)
FROM rust:nightly-bookworm AS builder

# Install build dependencies for static linking
RUN apt-get update && \
    apt-get install -y \
    musl-tools \
    musl-dev \
    pkg-config \
    libssl-dev \
    perl \
    make \
    && rm -rf /var/lib/apt/lists/*

# Add musl target for static builds
RUN rustup target add x86_64-unknown-linux-musl

# Set working directory
WORKDIR /build

# Copy the entire workspace
COPY . .

# Build with static linking (musl) in release mode using vendored OpenSSL
ENV OPENSSL_STATIC=true
ENV OPENSSL_VENDORED=true
RUN cargo build --release --target x86_64-unknown-linux-musl --bin server

# Verify the binary is statically linked
RUN ldd /build/target/x86_64-unknown-linux-musl/release/server 2>&1 | grep -qE "(not a dynamic executable|statically linked)" || \
    (echo "Binary is dynamically linked:" && ldd /build/target/x86_64-unknown-linux-musl/release/server && exit 1)

# Runtime stage using scratch for minimal image
FROM scratch

# Copy CA certificates for TLS
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Copy the static binary from builder
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/server /tslm-server

# Set the entrypoint
ENTRYPOINT ["/tslm-server"]
