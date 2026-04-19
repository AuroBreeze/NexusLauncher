# Launch Architecture

## Abstract Purpose
The `nexus-launch` crate is the **Process Orchestration Layer**. Its abstract role is to act as the **System Assembler**—taking the disparate results from authentication, versioning, and configuration, and synthesizing them into a single, valid operating system command that spawns the game environment.

It solves the problem of "Environment Synthesis," ensuring that thousands of individual files and parameters are correctly mapped to the JVM's strict execution requirements.

## Architectural Layers

The module follows a structured "Contract-to-Execution" flow:

### 1. The Context Contract ([Models](models.md))
- **The Blueprint**: Defines the `LaunchContext`, a comprehensive data structure that represents the "Resolved State" of everything needed to start the game.
- **Independence**: This layer ensures that the launcher engine doesn't need to know *how* assets were downloaded or *how* the user was authenticated; it only cares about the final results.

### 2. The Execution Engine ([Launcher](launcher.md))
- **Parameter Assembly**: The logic that transforms the `LaunchContext` into a complex CLI command.
- **Process Management**: Handles the spawning and lifecycle monitoring of the Java process.
- **Security**: Implements sensitive information masking in logs (e.g., hiding access tokens).

## Coordination & Flow
`nexus-launch` is the final destination of the launcher's data pipeline:

```text
[Auth] -> UserContext \
                       >-- [LaunchContext] --+--> [Command Assembly] --> [Child Process]
[Version] -> FilePaths /                     |
                                             +--> [JVM Argument Injection]
```

## Key Architectural Decisions

### Context-Driven Execution
The architecture is strictly **Context-Driven**. Instead of passing dozens of individual arguments, the entire intent is bundled into a `LaunchContext`. This allows for easier logging, debugging, and future support for "Launch Profiles" where a context can be saved and replayed.

### Fabric/Loader Awareness
The engine includes specialized logic to detect and merge **Loader Profiles** (like Fabric). It dynamically extends the Classpath based on the presence of a loader, allowing the core engine to support modded environments without being hardcoded for specific loaders.

## Design Philosophy
- **Deterministic**: Given the same `LaunchContext`, the resulting command should always be identical.
- **Platform Sensitivity**: Automatically handles OS-specific details like Classpath separators (`;` vs `:`) and file path formatting.
- **Security-First**: Ensures that authentication tokens are never leaked into the system logs while maintaining full transparency for other debugging parameters.
