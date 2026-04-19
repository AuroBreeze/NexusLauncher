# Main Orchestration Architecture

## Abstract Purpose
The `nexus-main` crate is the **System Orchestrator & Entry Point**. Its abstract role is to act as the **Central Nervous System** of the launcher—binding together the various domain-specific crates (Auth, Version, Java, etc.) and directing the high-level flow of the application based on user intent.

It solves the problem of **Service Coordination**, ensuring that components are initialized in the correct order and that data flows seamlessly from one stage of the launch pipeline to the next.

## Core Responsibilities

### 1. Application Lifecycle Management
`nexus-main` manages the global "boot-up" and "tear-down" of the launcher:
- **Environment Initialization**: Triggers the `init_workspace` call in `nexus-core` to ensure the filesystem is ready.
- **Observability Setup**: Configures the global `tracing` subscriber to manage logs and diagnostic output.
- **Graceful Termination**: Captures top-level errors and ensures they are reported meaningfully before the process exits.

### 2. Command Dispatching (The Router)
The crate acts as a high-level router that maps parsed CLI commands to their respective domain handlers.
- **Top-Level Matcher**: Decides whether to initiate a `Launch`, `Auth`, or `Install` workflow.
- **Nested Routing**: Handles multi-level command groups (e.g., `install core` vs `install loader`).

### 3. Workflow Synthesis (The Handlers)
While most logic resides in specialized crates, `nexus-main` contains the "glue logic" (Handlers) that orchestrates complex multi-crate interactions:
- **`handle_launch`**: Synthesizes a game launch by coordinating Java scanning, Authentication (Online/Offline), Version verification, and the final JVM execution.
- **`handle_core`**: Coordinates manifest discovery, version detail fetching, and bulk downloading of JARs, libraries, and assets.

## Coordination & Flow
`nexus-main` acts as the hub in a hub-and-spoke dependency model:

```text
[ User ] -> (CLI Args) -> [ nexus-main ]
                               |
         +---------------------+---------------------+
         |                     |                     |
 [ nexus-cli ]          [ nexus-auth ]        [ nexus-version ]
 (Parser)               (Security)            (Acquisition)
         |                     |                     |
         +---------------------+---------------------+
                               |
                               v
                        [ nexus-launch ]
                        (Process Spawning)
```

## Key Architectural Decisions

### Centralized Error Handling
By returning `Result<(), AnyError>` from the `main` function, the architecture ensures that any failure in any sub-crate is propagated up to a single point. This allows for a unified error reporting strategy and prevents "silent failures" deep in the stack.

### Implicit vs. Explicit Initialization
`nexus-main` performs **Explicit Initialization** of the environment (logging, workspace) before dispatching commands. This guarantees that all downstream crates can safely assume that the logging system is active and the basic directory structure exists.

## Design Philosophy

- **Minimal Logic**: The goal is for `nexus-main` to contain as little business logic as possible. Over time, large handlers (like `handle_launch`) are intended to be refactored into specialized service crates.
- **Fail-Fast**: The orchestrator is designed to halt the launch pipeline at the first sign of an unrecoverable error (e.g., missing Java, failed authentication).
- **Service Integration**: It treats other crates as "Black Box" services, interacting with them primarily through their public APIs and shared models.
