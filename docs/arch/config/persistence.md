# Configuration Persistence Architecture

## Abstract Purpose
The `persistence` logic (implemented via the `Config` trait in `config.rs`) is the **Atomic Storage Engine** of the configuration system. Its abstract role is to provide a standardized, fail-safe protocol for moving data between memory and disk.

## Core Mechanisms

### 1. The Config Trait
By using a Trait-based design, the system enforces a uniform interface for all persistent entities. This ensures that whether it's user settings or launch parameters, the calling code interacts with them in a predictable, asynchronous way (`load()` and `save()`).

### 2. Atomic Save Pattern (Write-Temp-Rename)
To prevent data corruption during power failures or crashes, the persistence engine implements an atomic write sequence:
1. **Serialize**: Data is converted to a TOML string in memory.
2. **Stage**: Data is written to a temporary file (`.toml.tmp`).
3. **Commit**: The temporary file is renamed to the target filename via an atomic OS-level operation.

### 3. Resilience Strategy
- **Default Injection**: If a file is missing or contains invalid syntax, the engine automatically falls back to `Default::default()`, ensuring the application can always boot.
- **Async I/O**: All disk operations are non-blocking, preventing persistence tasks from stuttering the main application thread.

## Design Philosophy
- **Fail-Safe by Design**: Prioritizes application stability over strict schema enforcement.
- **Platform Agnostic**: Uses `PathBuf` and standard library renames to ensure atomic behavior across different filesystems.
