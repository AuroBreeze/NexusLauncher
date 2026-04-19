# Launcher Engine Architecture

## Abstract Purpose
The `launcher` module (implemented in `launcher.rs`) is the **Process Synthesis Engine**. Its abstract role is to translate the high-level `LaunchContext` into the low-level "Language of the Operating System" (Shell commands and process spawning).

It is the final bridge between the Rust environment and the Java Virtual Machine.

## Core Responsibilities

### 1. Parameter Assembly (The Command Builder)
The engine performs a complex mapping of the `LaunchContext` to JVM arguments:
- **Classpath Construction**: Dynamically aggregates libraries, loaders, and core JARs into a platform-specific classpath string.
- **JVM Argument Injection**: Injects performance flags (e.g., G1GC) and memory limits (`-Xmx`).
- **Game Parameter Mapping**: Maps user identity and directory paths to Minecraft's standard `--arguments`.

### 2. Process Lifecycle Management
- **Spawning**: Uses `std::process::Command` to fork the Java process.
- **Monitoring**: Blocks on the child process to capture exit codes and handle the game's termination sequence.
- **Logging**: Provides high-level tracing of the assembled command (with security masking).

### 3. Mod Loader Integration
The engine contains a specialized "Extension Point" for mod loaders:
- **Dynamic Classpath Extension**: If a `FabricProfile` is present, the engine automatically resolves and adds the required loader libraries to the classpath before the core game JAR.
- **Main Class Overriding**: Allows mod loaders to take control of the entry point while passing original game parameters through.

## Implementation Details

### Security Masking
To prevent the accidental logging of sensitive credentials, the engine includes a heuristic masker:
```rust
// Heuristic: Mask long strings that aren't file paths
if s.len() > 20 && !s.contains('/') && !s.contains('\\') {
    "********".to_string()
}
```

## Design Philosophy
- **Separation of Concerns**: The engine only handles *how* to start the process, not *what* to start (which is defined by the Context).
- **Environment Agnostic**: Uses conditional compilation (`#[cfg]`) to ensure the correct command separators and path logic are used for Windows vs Unix systems.
- **Transparency**: Provides detailed logs of the execution command to aid in debugging complex launch failures.
