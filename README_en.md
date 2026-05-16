# Nexus Launcher

[中文](README.md) | English

A high-performance, command-line Minecraft launcher written in Rust.

This is an unofficial launcher and is not affiliated with Mojang or Microsoft.

## Features

- **Game Installation**: Async download of core JAR, libraries, and assets with SHA1 verification and resumable downloads
- **Java Management**: Auto-scan system Java, cache paths, download JRE from Adoptium
- **Mod Search**: Modrinth API integration with full-text search, facet filters, sorting, and pagination
- **Dependency Resolution**: Fetch project dependency trees and version listings
- **Mod Loaders**: Built-in Fabric installation and launch support
- **Authentication**: Microsoft device-code OAuth login + offline mode
- **Performance**: Lightweight and fast, built on the Tokio async runtime

## Installation

```bash
git clone https://github.com/AuroBreeze/NexusLauncher.git
cd NexusLauncher
cargo build --release
```

## Usage

### 1. Install a Game Version
```bash
cargo run -- install core --game-version 1.20.1
```

### 2. Install a Loader (Optional)
```bash
cargo run -- install loader 1.20.1 --loader fabric
```

### 3. Launch the Game
```bash
cargo run -- launch 1.20.1
```

## Command Reference

### Core
- `launch <instance>` — Launch game (`--offline`, `--max-memory`, `--force-scan`)
- `install core --game-version <V>` — Download version (`--name` for custom directory)
- `install loader <instance> --loader <fabric|quilt>` — Install mod loader
- `install mod --query <Q>` — Search mods (`-g` version filter, `-l` count)
- `install mod --query <Q> --download -i <instance> -L fabric` — Search and download mods (`-t` version type)

### Search
- `search mod <query>` — Modrinth full-text search (`-l` limit, `-g` version, `-i` sort, `-o` offset)
- `search java` — List installed Java (`-s` scan, `-v` filter version)
- `search user <instance>` — Read cached player profiles from game instance

### Auth & Config
- `auth --login` — Microsoft device-code login
- `auth --logout <name>` — Clear credentials
- `set -n <name> -u <uuid>` — Set offline username/UUID
- `set --show` — Display current config

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Group

We’re a group of friends who love Minecraft and programming. We’re currently developing a high-performance, user-friendly Minecraft launcher. Whether you’re a developer or just a gaming enthusiast, you’re welcome to join our community and chat with us!

This is our `discord` community: [link](https://discord.gg/gM85PKSYEe)

## License

GPL-3.0
