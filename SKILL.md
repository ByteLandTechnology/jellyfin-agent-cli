# jellyfin-agent-cli

Full-featured command line client for Jellyfin media browsing, playback, administration, and automation workflows.

## Description

`jellyfin-agent-cli` is a terminal-first Jellyfin client that covers everyday browsing and playback, administrative commands, and automation-oriented workflows. The CLI returns structured command results by default, provides a dedicated structured `help` surface, and includes E2E and REPL support.

## Prerequisites

- A reachable Jellyfin server for server-backed commands
- Valid Jellyfin credentials for authenticated commands
- Network access to the configured Jellyfin instance

## Invocation

```bash
# Shipped form
jellyfin-agent-cli [GLOBAL OPTIONS] <COMMAND> [COMMAND OPTIONS]

# Development form
cargo run -- [GLOBAL OPTIONS] <COMMAND> [COMMAND OPTIONS]

# Release binary
./target/release/jellyfin-agent-cli [GLOBAL OPTIONS] <COMMAND> [COMMAND OPTIONS]
```

## Input

### Global Options

| Option      | Short | Type   | Default | Description                                                                 |
| ----------- | ----- | ------ | ------- | --------------------------------------------------------------------------- |
| `--output`  | `-o`  | enum   | `yaml`  | Structured command-result format: `yaml`, `json`, `toml`, `table`, `ndjson` |
| `--server`  | `-s`  | string | —       | Server URL override for one invocation                                      |
| `--profile` | `-P`  | string | —       | Active-context override for one invocation                                  |
| `--debug`   | `-d`  | bool   | `false` | Enable debug logging                                                        |

### Help Contract

- `--help` renders human-readable man-like help and exits `0`
- Bare top-level or non-leaf invocation such as `jellyfin-agent-cli` or `jellyfin-agent-cli e2e` renders human-readable help and exits `0`
- `help --format yaml|json|toml [COMMAND_PATH...]` renders structured help
- Missing required leaf input returns a structured error envelope in the selected `--output` format

### Runtime Directories

| Kind   | Default                                                                     | Scope                   | Notes                                                                   |
| ------ | --------------------------------------------------------------------------- | ----------------------- | ----------------------------------------------------------------------- |
| Config | `~/Library/Application Support/jellyfin-cli/config.toml` on macOS           | user-scoped             | Stores CLI configuration and default server/profile settings            |
| Data   | `~/Library/Application Support/jellyfin-cli/repl_history` on macOS          | user-scoped             | Stores REPL history                                                     |
| State  | `~/Library/Application Support/jellyfin-cli/e2e/config/server.pid` on macOS | user-scoped E2E default | Used by E2E environment state unless `--config-dir` overrides it        |
| Cache  | `~/Library/Caches/jellyfin-cli/e2e/media` on macOS                          | user-scoped E2E default | Used by downloaded E2E media fixtures unless `--cache-dir` overrides it |

### Log Location

Logs are stored at `~/Library/Application Support/jellyfin-cli/e2e/data/log` on macOS. Log location can be overridden via the `--data-dir` option or `JELLYFIN_DATA_DIR` environment variable. Logs are written in structured text format for debugging and troubleshooting.

### Commands

| Command                                      | Description                                 |
| -------------------------------------------- | ------------------------------------------- |
| `login`                                      | Login to a Jellyfin server                  |
| `logout`                                     | Clear saved credentials                     |
| `search`                                     | Search for media                            |
| `latest`                                     | Show latest media                           |
| `continue`                                   | Show continue-watching items                |
| `play` / `pause` / `resume`                  | Playback control                            |
| `libraries` / `items` / `users` / `playback` | Browsing and management command groups      |
| `info` / `stats`                             | Server information                          |
| `config`                                     | Configuration management                    |
| `context show` / `context use <NAME>`        | Inspect or persist the active context       |
| `e2e`                                        | End-to-end environment and fixture commands |
| `repl`                                       | Interactive shell                           |
| `help`                                       | Structured help output                      |

## Output

Structured command results default to YAML. Use `--output` to request `json`, `toml`, `table`, or `ndjson` where supported.

```bash
jellyfin-agent-cli search "The Matrix"
jellyfin-agent-cli search "The Matrix" --output json
jellyfin-agent-cli e2e status --output toml
```

Human-readable help is always requested with `--help` and never shares the structured result formatter:

```bash
jellyfin-agent-cli --help
jellyfin-agent-cli e2e --help
```

Structured help is available only through the `help` command:

```bash
jellyfin-agent-cli help --format yaml
jellyfin-agent-cli help e2e logs --format json
```

Active Context behavior:

- `jellyfin-agent-cli context show` reveals the persisted context
- `jellyfin-agent-cli context use <NAME>` persists a new active context
- `--profile <NAME>` overrides the persisted context for one invocation only
- Command results surface `active_context` metadata when context resolution matters

## REPL Mode

Start an interactive shell with:

```bash
jellyfin-agent-cli repl
```

Within REPL:

- `help` shows plain-text REPL help
- `exit` or `quit` ends the session
- Tab completion is available
- Command history is persisted between sessions
- By default, command results are rendered in human-readable format. Use `--output json`, `--output yaml`, or `--output toml` to request structured output when needed.
- Structured command results still respect the shared output contract

## Errors

All command failures use a structured error envelope with machine-readable `code` and human-readable `message`.

```yaml
command: jellyfin-agent-cli config set-default
status: error
summary: Command failed.
errors:
  - code: input_invalid
    message: "Invalid arguments: the following required arguments were not provided: --name <NAME>"
```

Typical exit codes:

- `0` success or help rendered
- `10` authentication error
- `20` network error
- `30` API error
- `40` input validation error
- `50` internal error

## Examples

```bash
# Human-readable help
jellyfin-agent-cli --help

# Structured help
jellyfin-agent-cli help context --format yaml

# Persist a context
jellyfin-agent-cli context use home

# Override the active context for one invocation
jellyfin-agent-cli info --profile staging --output json

# Update the default profile
jellyfin-agent-cli config set-default --name home

# Start the E2E environment
jellyfin-agent-cli e2e setup
jellyfin-agent-cli e2e start --wait
```
