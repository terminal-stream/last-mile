# TSLM - Terminal Stream Last Mile

[![CI](https://github.com/terminal-stream/last-mile/workflows/CI/badge.svg)](https://github.com/terminal-stream/last-mile/actions)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

An asynchronous WebSocket gateway server written in Rust. TSLM provides a secure pub/sub messaging system with resource protection and rate limiting.

## Overview

TSLM acts as a WebSocket gateway enabling publishers to push messages from internal applications to the gateway, which fans out to subscribers on public networks. In a security breach scenario, the gateway cannot connect back into the internal network, limiting the attack surface.

### Features

- **Pub/Sub Messaging**: Create channels, publish messages, subscribe to updates
- **Security**: Token authentication, permission-based access control (CreateChannel, Subscribe, NotifyChannel)
- **Resource Protection**: Configurable connection limits, rate limiting (token bucket), message size validation
- **Reliability**: Backpressure control with bounded channels, graceful shutdown, type-safe error handling
- **Flexible Configuration**: Multiple listeners with independent settings, per-listener authentication and limits

## Quick Start

### Installation

**Prerequisites:** Rust 1.90+ (2024 edition), OpenSSL development libraries (for local builds)

```bash
git clone https://github.com/terminal-stream/last-mile.git
cd last-mile
make build-release
make test
make run-release
```

### Configuration

Create `config/tslm.toml`:

```toml
# Public listener for subscribers
[listener.public]
ip = "0.0.0.0"
port = 8080
default_endpoint_permissions = ['Subscribe']
auth_tokens = ['public-token-1']
max_connections = 10000
max_message_size = 65536
channel_buffer_size = 100
rate_limit_per_second = 10

# Private listener for publishers
[listener.private]
ip = "127.0.0.1"
port = 8081
default_endpoint_permissions = ['CreateChannel', 'NotifyChannel']
max_connections = 100
max_message_size = 131072
```

### Message Protocol

**Subscribe to a channel:**
```json
{"Subscribe": "channel-name"}
```

**Create a channel:**
```json
{"CreateChannel": "channel-name"}
```

**Publish to a channel:**
```json
{"NotifyChannel": ["channel-name", {"Text": "Hello, World!"}]}
```

**Server responses:**
```json
{"Success": "Command executed: Subscribe(\"channel-name\")"}
{"Error": "Permission denied: CreateChannel"}
{"ChannelMessage": ["channel-name", {"Text": "Hello, World!"}]}
```

## Configuration Reference

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `ip` | String | Bind IP address | Required |
| `port` | u16 | Bind port | Required |
| `default_endpoint_permissions` | Array | Allowed permissions: `Subscribe`, `CreateChannel`, `NotifyChannel` | `[]` |
| `auth_tokens` | Array | Authentication tokens (optional) | None |
| `max_connections` | Number | Maximum concurrent connections | Unlimited |
| `max_message_size` | Number | Maximum message size (bytes) | 65536 |
| `max_frame_size` | Number | Maximum WebSocket frame size (bytes) | 16777216 |
| `channel_buffer_size` | Number | Buffer size for backpressure (0 = unbounded) | 0 |
| `rate_limit_per_second` | Number | Messages per second per connection | None |

### Configuration Tips

**Public-facing listeners:** Use authentication tokens, set connection/rate limits, restrict to `Subscribe` permission only.

**Internal listeners:** Higher limits, allow `CreateChannel` and `NotifyChannel` permissions, bind to `127.0.0.1`.

**Resource tuning:** Set `channel_buffer_size` to prevent memory exhaustion from slow consumers. Set `rate_limit_per_second` to prevent abuse.

## Docker Deployment

```bash
make docker-build   # Build Docker image (unified multi-stage build)
make docker-run     # Run container with mounted config
make docker-logs    # View container logs
make docker-stop    # Stop the container
```

The Docker build uses a unified multi-stage build:
- **Build stage**: Rust 1.90 with musl for static linking
- **Runtime stage**: Scratch base for minimal image size (~10MB)
- Fully static binary with no runtime dependencies

## Development

See [CLAUDE.md](CLAUDE.md) for architecture overview and development guide.

**Common commands:**
```bash
make help            # Show all available targets
make build           # Build in debug mode
make test            # Run all tests (15 tests)
make check           # Run clippy, fmt check, and tests
make run             # Build and run in debug mode
make doc             # Generate and view API docs
```

## Architecture

- **Hub**: Central coordination point
- **Directory**: Registry of channels and endpoints (RwLock-based)
- **Channel**: Pub/sub topic with automatic pruning of failed endpoints
- **Endpoint**: Represents a WebSocket connection with permissions
- **WebSocket Server**: Handles connections with rate limiting and backpressure

## Roadmap

- Timeout configurations (idle, max duration)
- Correlation IDs for request/response matching
- Channel backlog/history
- TLS/SSL support
- Prometheus metrics
- OAuth2 integration
- Message schema improvements (move away from enum serialization)

## Contributing

Contributions welcome! The CI pipeline runs on all PRs with tests and linting.

## License

Apache-2.0

## Notes

The current message protocol serializes Rust enums directly to JSON, which is practical but not ideal for stable APIs. Future versions will provide more language-agnostic message schemas.
