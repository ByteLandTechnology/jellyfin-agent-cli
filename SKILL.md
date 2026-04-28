---
name: jellyfin-agent-cli
description: >
  Terminal client for Jellyfin media servers — browse libraries, control playback, manage users,
  run admin tasks, and automate workflows from the command line. Use this skill whenever the user
  mentions Jellyfin, media servers, streaming, home theater automation, or asks to interact with a
  Jellyfin instance programmatically. Also use when the user is working in this repository
  (jellyfin-agent-cli) — building, testing, debugging, or extending the CLI itself.
  Triggers on: "jellyfin", "media server", "jellyfin cli", "jellyfin-agent-cli",
  "streaming automation", "home theater cli", "jellyfin e2e", "jellyfin api",
  "browse my movies", "play media from terminal".
---

# jellyfin-agent-cli

A Rust-based CLI that talks to Jellyfin media servers. The user is either **using** the CLI to interact with a Jellyfin instance, or **developing** the CLI itself. Both cases are covered below.

## Description

`jellyfin-agent-cli` is a terminal-first Jellyfin client that covers everyday browsing and playback, administrative commands, and automation-oriented workflows. The CLI returns structured command results by default (YAML), provides a dedicated structured `help` surface, and includes E2E and REPL support.

## Prerequisites

- A reachable Jellyfin server for server-backed commands
- Valid Jellyfin credentials for authenticated commands
- Network access to the configured Jellyfin instance

## Invocation

The binary name is `jellyfin-agent-cli`. During development use `cargo run --` as a prefix.

```bash
# Installed binary
jellyfin-agent-cli search "The Matrix"

# Development
cargo run -- search "The Matrix"

# Structured output (all commands support --output)
cargo run -- search "The Matrix" --output json

# Human-readable help
cargo run -- --help
cargo run -- items --help

# Structured help (machine-parseable)
cargo run -- help --format json
cargo run -- help items list --format yaml
```

Always prefer `cargo run --` unless the user explicitly says they're using the installed binary.

Most commands need an authenticated session. The login command saves credentials under a profile name:

```bash
cargo run -- login --server http://192.168.1.100:8096 --username admin --password secret --name home
```

After login, commands use the default profile. Switch profiles with `context use <name>` or override per-invocation with `-P <name>`.

## Input

### Global Options

| Option      | Short | Type   | Default | Description                                                                 |
| ----------- | ----- | ------ | ------- | --------------------------------------------------------------------------- |
| `--output`  | `-o`  | enum   | `yaml`  | Structured command-result format: `yaml`, `json`, `toml`, `table`, `ndjson` |
| `--server`  | `-s`  | string | —       | Server URL override for one invocation                                      |
| `--profile` | `-P`  | string | —       | Active-context override for one invocation                                  |
| `--debug`   | `-d`  | bool   | `false` | Enable debug logging                                                        |

### Commands

The CLI has leaf commands and grouped commands with subcommands. Key groups:

**Media browsing:** `search <query>`, `latest`, `continue`, `libraries list`, `items list`, `items get`, `genres`, `studios`, `actors`, `channels list`, `remote-search <query>`

**Playback:** `play <item_id>`, `pause`, `resume`, `playback info`, `playback seek`, `playback stop`, `playback queue`

**Admin:** `users list|get|create|delete`, `sessions`, `activity-log`, `system restart|shutdown`, `scheduled-tasks list|start|stop`, `devices list`, `plugins list|uninstall`

**Collections:** `playlists list|get|create|add|remove|delete`, `items favorite|unfavorite|favorites|rate`

**Notifications:** `notifications list|mark-read|mark-all-read`

**Config & profiles:** `config show|list-servers|set-default|remove-server`, `context show|use`, `info`, `stats`

**Testing:** `e2e setup|start|stop|status|logs|reset|media|config`

**Interactive:** `repl` (tab-completed interactive shell), `help` (structured help in yaml/json/toml)

When constructing commands, use `--help` on any group to discover subcommands.

### Help Contract

- `--help` renders human-readable man-like help and exits `0`
- Bare top-level or non-leaf invocation renders human-readable help and exits `0`
- `help --format yaml|json|toml [COMMAND_PATH...]` renders structured help
- Missing required leaf input returns a structured error envelope in the selected `--output` format

### Runtime Directories

| Kind   | Default                                                            | Scope                   |
| ------ | ------------------------------------------------------------------ | ----------------------- |
| Config | `~/Library/Application Support/jellyfin-cli/config.toml` on macOS  | user-scoped             |
| Data   | `~/Library/Application Support/jellyfin-cli/repl_history` on macOS | user-scoped             |
| State  | `~/Library/Application Support/jellyfin-cli/e2e/config/server.pid` | user-scoped E2E default |
| Cache  | `~/Library/Caches/jellyfin-cli/e2e/media` on macOS                 | user-scoped E2E default |

### Log Location

Logs are stored at `~/Library/Application Support/jellyfin-cli/e2e/data/log` on macOS. Override via `--data-dir` or `JELLYFIN_DATA_DIR` environment variable.

### Active Context

- `context show` reveals the persisted context
- `context use <NAME>` persists a new active context
- `--profile <NAME>` overrides for one invocation only
- Command results surface `active_context` metadata when context resolution matters

## Output

Every command supports `--output` (or `-o`) with values: `yaml` (default), `json`, `toml`, `table`, `ndjson`.

For scripting and automation, `--output json` or `--output ndjson` is usually best. For human consumption, `--output table` or the default YAML works well.

```bash
cargo run -- search "The Matrix"
cargo run -- search "The Matrix" --output json
cargo run -- e2e status --output toml
```

Human-readable help is always requested with `--help` and never shares the structured result formatter. Structured help is available only through the `help` command:

```bash
cargo run -- help --format yaml
cargo run -- help context --format json
```

## REPL Mode

Start an interactive shell with `repl`:

```bash
cargo run -- repl
```

Within REPL: `help` shows plain-text help, `exit`/`quit` ends the session. Tab completion and command history (persisted between sessions) are available. Default REPL output is human-readable; use `--output json|yaml|toml` for structured output.

## Errors

All command failures use a structured error envelope with machine-readable `code` and human-readable `message`.

```yaml
command: jellyfin-agent-cli items get
status: error
summary: Command failed.
errors:
  - code: input_invalid
    message: "Invalid arguments: the following required arguments were not provided: <ITEM_ID>"
```

Typical exit codes: 0 success, 10 authentication error, 20 network error, 30 API error, 40 input validation, 50 internal error.

## Examples

```bash
# Human-readable help
cargo run -- --help

# Structured help
cargo run -- help context --format yaml

# Persist a context
cargo run -- context use home

# Override the active context for one invocation
cargo run -- info --profile staging --output json

# Update the default profile
cargo run -- config set-default --name home

# Start the E2E environment
cargo run -- e2e setup
cargo run -- e2e start --wait

# Search across all libraries
cargo run -- search "The Matrix"

# Get JSON output for scripting
cargo run -- search "The Matrix" --output json
```
