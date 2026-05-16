# Nexus Launcher v0.1.1

## What's New

### Mod Download & Install

```
nexus install mod -q sodium --download -i 1.20 -L fabric
```

- Search Modrinth for mods and download directly to your instance's `mods/` folder
- **Auto game version detection** — reads `version.json` from the instance, no need to pass `-g` manually
- **Loader filtering** — `-L fabric|quilt` to match the right mod variant
- **Release channel filter** — `-t release|beta|alpha` to pick stable or pre-release builds
- **SHA1 verification** — downloaded files are checked against Modrinth's published hash
- **Progress bar** — visual download progress with size and ETA

### Mod Manifest

Each instance now maintains a `nexus_mods.toml` tracking all installed mods:

```toml
[[mods]]
name = "Sodium"
project_id = "AANobbMI"
version_number = "mc1.20-0.4.10"
version_type = "release"
filename = "sodium-fabric-mc1.20-0.4.10+build.27.jar"
sha1 = "b11e18bb09f06c3f8028fa2c090072976fa326d0"
loader = "fabric"
game_version = "1.20"

[[mods.dependencies]]
name = "Fabric API"
project_id = "P7dR8mSH"
dependency_type = "required"
```

- Records SHA1 hash for integrity verification (`ModManifest::verify()` checks all files)
- Tracks dependency names, versions, and loaders
- Deduplicates on re-install

### Fixes & Improvements

- HTTP status code checks on all API calls (no more cryptic deserialization errors on 404/429)
- Streaming SHA1 pre-check (no more loading full JARs into memory)
- Concurrent dependency name resolution
- Warn on corrupt manifest instead of silently discarding

### Other

- Added TODOs for: dependency auto-download, download statistics, specific version targeting, loader-based file matching

## Usage

```bash
# Basic download
nexus install mod -q sodium --download -i 1.20 -L fabric

# Beta version
nexus install mod -q sodium --download -i 1.20 -L fabric -t beta

# Search without downloading
nexus install mod -q shader -g 1.21.4 -l 10
```
