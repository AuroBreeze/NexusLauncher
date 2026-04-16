# Nexus Launcher

A high-performance, command-line Minecraft launcher written in Rust.

## Features

- **Java Management**: Automatic detection and downloading of required Java versions (defaulting to 17).
- **Game Installation**: Download core JARs, libraries, and assets asynchronously.
- **Mod Loader Support**: Built-in support for Fabric and Quilt installation.
- **Authentication**: Supports both Microsoft online login and offline modes.
- **Performance**: Lightweight and fast, built on the Tokio runtime.

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

- `launch <instance>` - Launch a game instance (options: `--offline`, `--max-memory`, `--force-scan`)
- `install core --game-version <V>` - Download a specific Minecraft version
- `install loader <instance> --loader <fabric|quilt>` - Install a mod loader
- `install mod --query <Q> --game-version <V>` - Search and install mods
- `java --scan` - Scan local system for Java installations
- `java --download --version <N>` - Download a specific Java runtime
- `auth --login` - Authenticate with Microsoft
- `set --show` - Display current configuration and settings

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

GPL-3.0