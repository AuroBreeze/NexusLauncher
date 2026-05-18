# Nexus Launcher v0.1.3

## What's New

### Crash Log Capture

The launcher now pipes game stderr/stdout, surfaces errors in real time, and automatically
writes crash reports on abnormal exit:

```
~/.minecraft/crash_logs/
├── 1.20-1717543200.log
└── 1.21.4-1717544100.log
```

- Crash logs include exit code, last 500 stderr lines, and last 100 stdout lines
- Ring buffer keeps memory bounded regardless of game uptime
- Old logs pruned automatically when the directory exceeds 20 files

### Manifest Cleanup on Uninstall

`nexus uninstall mod` now removes corresponding entries from `nexus_mods.toml` so the
manifest stays in sync with the filesystem.

## Global Changes

### Logging & Observability

All `info`-level output is now concise and human-readable:

- Mod search shows per-result summaries instead of full struct dumps
- Version selection prints loaders, game versions, file count, and deps in one line
- Debug/trace coverage spans auth, config, download, I/O, and orchestration paths
- Every Modrinth API call logs its URL and response status at debug level

Enable full diagnostics:
```bash
RUST_LOG=debug nexus launch 1.20
RUST_LOG=nexus_mods=debug nexus install mod -q iris --download -i 1.20 -L fabric
```

### Performance

- **Shared HTTP client** — a single `reqwest::Client` per process reuses connection pools
  and TLS sessions across all API calls, eliminating per-request handshake overhead
- **Parallel SHA1 cache check** — existing file verification uses 2x download concurrency
- **Non-blocking I/O** — all `std::fs::read_to_string` calls in async functions replaced
  with `tokio::fs`

### Tools

- New `scripts/test.sh` — dedicated test runner: non-network crates in parallel,
  network crates (Modrinth / Mojang) serially to avoid rate limiting
- `check.sh` (local pre-push) no longer runs tests — keeps push fast; CI runs them in full
- CI workflow now includes test steps (was previously missing)
- `generate_todo.sh` now works when invoked manually; previously it exited early unless
  run as a pre-commit hook

## Fixes

- `nexus_mods.toml` no longer retains stale entries for uninstalled mods
- Pre-commit hook no longer blocks `generate_todo.sh` manual execution
- False-positive test failures from Modrinth rate limiting eliminated
