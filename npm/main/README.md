# jellyfin-agent-cli

npm wrapper for the `jellyfin-agent-cli` CLI.

## Install

```bash
npm install -g jellyfin-agent-cli
```

The matching native binary ships in a per-platform npm package that is selected
automatically via `optionalDependencies`. No postinstall download required.

## Supported platforms

- darwin-arm64, darwin-x64
- linux-arm64, linux-x64
- win32-arm64, win32-x64

## Usage

```bash
jellyfin-agent-cli --help
jellyfin-agent-cli search "The Matrix"
jellyfin-agent-cli help --format yaml
```
