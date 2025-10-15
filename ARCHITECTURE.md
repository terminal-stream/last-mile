# TSLM Architecture & Development Guide

This document provides an overview of the TSLM architecture, codebase structure, and development workflows.

## Project Overview

TSLM (Terminal Stream Last Mile) is an asynchronous WebSocket gateway server written in Rust using tokio-tungstenite. It acts as a one-way WebSocket gateway that allows internal/private applications to push messages to public network subscribers without exposing the internal network to security risks.

**Security Model**: Publishers connect outbound from the internal network to push messages. If the gateway is compromised, it cannot connect inward to the intranet, limiting attack surface.

## Workspace Structure

This is a Cargo workspace with three crates:

- **common**: Shared message types and error definitions
- **server**: The WebSocket gateway server (binary: `server`)
- **client**: Client library for connecting to the server

## Common Commands

All build and development tasks are handled via the Makefile. Run `make help` to see all available targets.

### Building and Running

```bash
# Build in debug mode
make build

# Build in release mode
make build-release

# Run the server (debug mode)
make run

# Run the server (release mode)
make run-release

# Clean build artifacts
make clean
```

### Testing and Quality

```bash
# Run all tests
make test

# Run all checks (clippy, fmt, tests)
make check

# Run clippy linter
make clippy

# Format code
make fmt

# Check formatting without modifying
make fmt-check

# Generate and open documentation
make doc
```

### Docker

The project uses a unified multi-stage Dockerfile with Rust 1.90 and musl for static linking.

```bash
# Build Docker image (unified multi-stage build)
make docker-build

# Run containerized server
make docker-run

# View container logs
make docker-logs

# Stop the container
make docker-stop

# Clean Docker images
make docker-clean
```

The Docker image:
- Build stage: Rust 1.90 with musl-tools for static compilation
- Runtime stage: Scratch (minimal ~10MB image)
- Fully static binary with zero runtime dependencies

### Direct Cargo Commands

If you need to use cargo directly:

```bash
# Run tests for specific package
cargo test -p common
cargo test -p last-mile-server

# Run specific test by name
cargo test <test_name>

# Check without building
cargo check
```

## Architecture

### Core Components

**Hub** (server/src/tslm/hub.rs): Central coordinator that creates endpoints and manages the directory. Uses an atomic sequence generator for endpoint IDs.

**Directory** (server/src/tslm/directory.rs): Global registry maintaining two mappings:
- `channels_by_id`: Maps channel IDs to Channel instances
- `endpoints_by_id`: Maps endpoint IDs to Endpoint instances

All lookups go through the Directory using RwLock for concurrent access.

**Endpoint** (server/src/tslm/endpoint.rs): Represents a WebSocket connection. Each endpoint has:
- A unique ID
- Reference to the Directory
- An unbounded channel (`tx`/`rx`) for outgoing messages to the client
- A set of allowed permissions (CreateChannel, NotifyChannel, Subscribe)

Endpoints process TerminalStreamCommands and enforce permission checks.

**Channel** (server/src/tslm/channel.rs): Pub/sub channel implementation. Maintains a BTreeMap of subscribed endpoints. When publishing:
1. Reads subscriber list
2. Attempts to send to each subscriber
3. Automatically prunes failed endpoints (disconnected clients)

### Message Flow

1. **Creating a Channel**: Client sends `CreateChannel(channel_id)` → Endpoint validates permissions → Directory creates Channel instance
2. **Subscribing**: Client sends `Subscribe(channel_id)` → Endpoint looks itself up in Directory → Channel adds Endpoint to subscribers
3. **Publishing**: Client sends `NotifyChannel(channel_id, message)` → Endpoint finds Channel in Directory → Channel fans out message to all subscribers

### Configuration

Server configuration is in `config/tslm.toml`. Define multiple listeners with different permissions:

```toml
[listener.public]
ip = "0.0.0.0"
port = 8080
default_endpoint_permissions = ['Subscribe']

[listener.private]
ip = "127.0.0.1"
port = 8081
default_endpoint_permissions = ['CreateChannel', 'NotifyChannel']
```

**Design Pattern**: Separate public/private listeners with different permission sets. Public listeners typically only allow Subscribe, while private (internal) listeners allow CreateChannel and NotifyChannel.

### Message Schema

Defined in `common/src/message.rs`:

- **TerminalStreamCommand**: Client → Server (CreateChannel, Subscribe, NotifyChannel)
- **ClientCommand**: Server → Client (Text, ChannelMessage)
- **ChannelMessage**: Text or JSON payload

Note: Current message schemas use direct enum serialization and are considered experimental. They will change in future versions.

### Client Library

The `client` crate provides `LastMileClient` with methods:
- `connect(runtime, url)`: Establish WebSocket connection
- `create_channel(channel_id)`: Create a new channel
- `subscribe(channel_id)`: Subscribe to a channel
- `notify_channel(channel_id, text)`: Send text message
- `notify_channel_json(channel_id, value)`: Send JSON message

See `client/examples/client.rs` for usage examples.

### Dependency Notes

- Requires OpenSSL to be installed on the system
- Uses tokio for async runtime
- Uses tokio-tungstenite for WebSocket implementation with native-tls feature
- Uses log4rs for logging

### Key Design Decisions

1. **Lazy cleanup**: When an endpoint disconnects, the Directory doesn't immediately notify all channels. Channels detect disconnection on next publish attempt and prune automatically.

2. **Permission model**: Permissions are set per-listener at server startup. Individual endpoints cannot escalate privileges.

3. **No channel backlog**: Messages are not persisted. Subscribers only receive messages sent after they subscribe.

4. **Unidentified messages**: No request/response correlation IDs yet. All messages are fire-and-forget.
