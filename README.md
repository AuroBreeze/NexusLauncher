# Nexus Launcher

A command-line Minecraft launcher written in Rust.

## Overview

Nexus Launcher is designed to be lightweight and operates entirely from the terminal, avoiding the resource overhead of a graphical user interface. It handles asynchronous game asset downloads and automatically sets up the required Java environment to launch the game.

NexusLauncher is an `unofficial` open-source launcher and is not affiliated with Mojang or Microsoft.

## Features

- **Asynchronous downloads**: Concurrent asset and library downloads with connection limits
- **Java management**: Automatic detection and download of required Java versions
- **Authentication**: Microsoft account login with token persistence
- **Version isolation**: Separate directories for each game version
- **Mod loader support**: Fabric and Quilt installation
- **Offline mode**: Play without authentication

## Installation

```bash
git clone https://github.com/AuroBreeze/NexusLauncher.git
cd NexusLauncher
cargo build --release
```

## Usage

### Launch a game
```bash
cargo run -- launch 1.20.1
```

## Commands

- `launch <version>` - Launch Minecraft (options: `--player-name`, `--max-memory`, `--offline`, `--force-scan`)
- `auth --login` - Authenticate with Microsoft
- `auth --logout <username>` - Remove authentication
- `java --scan` - Scan for Java installations
- `java --download --version <N>` - Download Java runtime
- `loader <version> --loader <fabric|quilt>` - Install mod loader
- `set --name <name>` - Set offline username
- `set --show` - Display current settings

## Requirements

- Rust 1.70+ and Cargo
- Internet connection for downloads
- System credential manager for token storage

## License

GPL-3.0