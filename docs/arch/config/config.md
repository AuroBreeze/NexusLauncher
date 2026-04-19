# Configuration Architecture

## Abstract Purpose
The `nexus-config` crate is the **System State & User Intent Manager**. It provides a unified way to persist and retrieve data, ensuring that the launcher's environment and the user's choices are synchronized between memory and the physical disk.

Unlike a simple "settings" folder, this module acts as a **Structured Repository** that validates and manages the lifecycle of data objects.

## Architectural Layers

The module is structured into three distinct layers to ensure separation of concerns:

### 1. Persistence Layer (`config.rs`)
- **The Protocol**: Defines the `Config` trait, which is the "Golden Rule" for how any data structure should be saved to or loaded from the disk.
- **The Engine**: Implements the atomic **Write-Temp-Rename** pattern, ensuring that data corruption is physically impossible at the OS level during a save.
- **The Format**: Enforces **TOML** as the standard serialization format for all persistent state.

### 2. Schema Layer (`models.rs`)
- **Data Blueprint**: Defines the pure data structures (Anemic Models) without logic.
- **Domain Separation**: Intentionally splits state into two separate files to prevent cross-contamination:
    - **User State (`nexus_config.toml`)**: Stores "Identity" (Usernames, UUIDs, Profiles).
    - **System State (`launch_config.toml`)**: Stores "Environment" (Discovered Java paths, Global flags).

### 3. Domain Logic Layer (`userconfig.rs`, `launchconfig.rs`)
- **Connectivity**: Binds the *Schema* to the *Persistence Layer* by implementing the `Config` trait and defining the specific file paths.
- **Active Behavior**: Adds "Smart" methods to the data models (e.g., `get_valid_java`), turning a dumb data structure into an active system component that can re-verify itself.

## Coordination & Flow
The entry point of the crate (`lib.rs`) acts as the **Orchestrator**.

```text
User Input (CLI) -> handle_set (lib.rs)
                          |
           +--------------+--------------+
           |                             |
      UserConfig (Identity)        LaunchConfig (Environment)
           |                             |
      [ Config Trait ]              [ Config Trait ]
           |                             |
    nexus_config.toml             launch_config.toml
```

## Key Architectural Decisions

### Why Two Separate Config Files?
We use a **Dual-Config Strategy**:
1. **User Identity** is relatively stable and potentially sensitive (UserConfig).
2. **Launch Environment** is highly dynamic and depends on the specific machine's hardware/software setup (LaunchConfig).
By separating them, we allow users to share their identity/profile without accidentally overwriting someone else's machine-specific Java paths.

### Validated Retrieval
The architecture implements a **"Re-verify on Load"** pattern. Instead of trusting the configuration file implicitly, the logic in `launchconfig.rs` treats the stored data as a "hint." Before the path is actually used to launch a game, the system re-executes the binary to confirm it's still there and still the correct version.

## Design Principles
- **Atomicity**: No partial saves; the file is either 100% correct or 100% original.
- **Asynchronicity**: All disk I/O is non-blocking to prevent UI/CLI stutter.
- **Human-Centric**: Configurations are designed to be easily read and modified by humans using any text editor.
