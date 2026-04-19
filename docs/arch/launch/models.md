# Launch Models Architecture

## Abstract Purpose
The `models` module in `nexus-launch` defines the **Execution Contract**. Its abstract role is to provide a "Single Source of Truth" for a launch attempt. It aggregates data from all other modules (Auth, Version, Config) into a unified state that is ready for process execution.

## Core Data Structures

### 1. `LaunchContext`
This is the **Master Blueprint**. It represents a fully resolved game instance ready to be spawned.
- **Path Resolution**: Holds absolute paths to the Java binary, core JAR, and libraries.
- **Resource Linking**: Bridges the game version with its required asset index.
- **Loader Injection**: Optionally carries a `FabricProfile` to support modded environments.

### 2. `UserContext`
Represents the **Security & Identity State**. It encapsulates the results of the authentication flow, providing the necessary credentials (Username, UUID, Access Token) required by Minecraft's internal authentication system.

## Design Philosophy

- **Completeness**: A `LaunchContext` must contain every piece of information required to start the game. The `launcher.rs` should never need to look up external data.
- **Agnosticism**: The models don't care how they were built. This allows for flexible construction—for example, a `LaunchContext` could be built for an "Online" session or a "Local/Offline" session using the same struct.
- **Immutability**: Once a `LaunchContext` is handed to the execution engine, it is treated as a read-only snapshot of the intent.
