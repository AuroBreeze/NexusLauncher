# Architectural Overview (Fast Read)

## 1. The Core Philosophy
Nexus Launcher is built on three fundamental architectural pillars:
- **Strict Decoupling**: Interfaces (CLI), Orchestration (Main), and Domain Logic (Auth, Version, etc.) are physically separated into different crates.
- **Declarative Design**: The system uses data models (Contexts) to describe "intent" rather than imperative code.
- **Reliability First**: Every external resource is verified via SHA-1, and every sensitive credential is encrypted using hardware-bound keys.

## 2. System Layering Model
The project follows a 4-tier layered architecture:

| Layer | Responsibility | Crates |
| :--- | :--- | :--- |
| **Interface** | User interaction & command parsing | `nexus-cli` |
| **Orchestration** | High-level workflow control | `nexus-main` |
| **Domain Services** | Specialized business logic | `nexus-auth`, `nexus-version`, `nexus-loader`, `nexus-java` |
| **Infrastructure** | Cross-cutting concerns & state | `nexus-core`, `nexus-config`, `nexus-launch` |

## 3. The 5-Phase Launch Pipeline
When a user types `nexus-launcher launch`, the system executes the following deterministic pipeline:

### Phase 1: Bootstrap (The Setup)
- **Crate**: `nexus-core`, `nexus-config`
- **Action**: Load user preferences, verify the `.minecraft` directory structure, and initialize the workspace.

### Phase 2: Authentication (The Identity)
- **Crate**: `nexus-auth`
- **Action**: Check for existing tokens. If missing, trigger the Device Code Flow. If present, perform a "Silent Refresh" to obtain a valid Minecraft session token.

### Phase 3: Resolution (The Blueprint)
- **Crate**: `nexus-version`, `nexus-loader`, `nexus-java`
- **Action**: Fetch Mojang/Fabric metadata. Determine the exact JARs, assets, and Java version required. Assemble a `LaunchContext` blueprint.

### Phase 4: Acquisition (The Gathering)
- **Crate**: `nexus-version` (download/source)
- **Action**: Parallel download of all missing assets and libraries. Perform real-time SHA-1 integrity checks. Link objects into the game folder.

### Phase 5: Execution (The Launch)
- **Crate**: `nexus-launch`
- **Action**: Synthesize the `LaunchContext` into a final JVM command. Inject memory limits and GC flags. Spawn the Java process and monitor its lifecycle.

## 4. Global Data Flow
```text
[ User Input ]
      |
      v
[ nexus-cli ] --(Parsed Args)--> [ nexus-main ]
                                       |
      +--------------------------------+--------------------------------+
      |               |                |               |                |
[ nexus-auth ] [ nexus-config ] [ nexus-java ] [ nexus-version ] [ nexus-loader ]
      |               |                |               |                |
      +--------------------------------+--------------------------------+
                                       |
                         (Aggregated into LaunchContext)
                                       |
                                       v
                                [ nexus-launch ]
                                       |
                                [ Child Process ]
```

## 5. Key Design Patterns
- **Context-Driven Execution**: All data required to launch is bundled into a `LaunchContext`. This makes the execution engine deterministic and easy to debug.
- **Pure Data Models**: Crates like `nexus-version` and `nexus-auth` use anemic data models for API interaction, keeping logic strictly in orchestrators.
- **Hardware-Locked Storage**: Sensitive data (Refresh Tokens) is encrypted with AES-256-GCM using a key derived from the local machine ID.
