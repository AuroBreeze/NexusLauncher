# Nexus Launcher v0.1.2

## What's New

### List Command

```
nexus list instances
nexus list users
nexus list info 1.21.4 -lm
```

- `instances` — list all game instances with version and Java info
- `users` — show online/offline profiles and UUID mappings
- `info <name>` — instance details: version, loader type, mods count, cached users
- `-l` flag shows loader type (fabric/quilt), `-m` lists individual mod filenames

### Uninstall Command

```
nexus uninstall instance 1.20
nexus uninstall mod sodium -i 1.20
```

- `instance <name>` — remove an entire game instance directory
- `mod <query> -i <instance>` — remove mods by name match (case-insensitive)

### Search: Core Versions & Loader Versions

```
nexus search core -v 1.21 -s -l 10
nexus search loader fabric -g 1.21.4 -s
nexus search loader quilt -s
```

- `search core` — fetch version list from Mojang manifest, filter by prefix or stable-only
- `search loader fabric|quilt` — fetch loader versions from meta APIs
- Supports `-g` game version filter, `-s` stable-only, `-l` limit

### Fixes & Improvements

- `handle_launch` no longer panics on missing `version.json` — reports the required `install core` command
- Error handling: `usercache.json` read errors now distinguish `NotFound` from permission/IO failures
- Removed unused `--list` flag from `install core`

### Internal

- `nexus-list`, `nexus-uninstall`, `nexus-search` — handler crates extracted from `nexus-main`
- `cargo fmt` pre-commit hook now auto-formats and stages changes
