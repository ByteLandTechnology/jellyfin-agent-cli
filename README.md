# Jellyfin CLI

Command line client for Jellyfin media server.

## Features

- Search and browse media libraries
- Control playback on external players
- Manage users and libraries
- End-to-end testing infrastructure

## Installation

```bash
cargo install --path .
```

## Quick Start

### E2E Testing

The CLI includes built-in E2E testing with an isolated Jellyfin server:

```bash
# 1. Initialize the E2E environment
jellyfin e2e setup

# 2. Download test media (open-licensed)
jellyfin e2e media download --download all

# 3. Start the E2E server
jellyfin e2e start --wait

# 4. Access at http://127.0.0.1:8096

# 5. Stop when done
jellyfin e2e stop --cleanup
```

See [docs/e2e-testing.md](docs/e2e-testing.md) for complete E2E testing documentation.

### Connecting to a Server

```bash
# Login to a server
jellyfin login --server https://jellyfin.example.com

# Search for media
jellyfin search "movie title"

# Get latest media
jellyfin latest --limit 10
```

## Output Formats

All commands support structured output:

```bash
# YAML (default)
jellyfin search "query"

# JSON
jellyfin search "query" --output json

# TOML
jellyfin search "query" --output toml
```

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test

# Run E2E integration tests
cargo test e2e
```

## Project Structure

```
jellyfin-cli/
├── crates/
│   ├── api/       # Jellyfin API client
│   ├── core/      # Shared business logic
│   ├── player/    # Media player integration
│   └── cli/       # CLI application
├── specs/         # Feature specifications
├── docs/          # Documentation
└── test-data/     # E2E test data
```

## License

MIT OR Apache-2.0
