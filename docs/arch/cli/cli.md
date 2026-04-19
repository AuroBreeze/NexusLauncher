# CLI Architecture

## Overview
The `nexus-cli` crate serves as the **declarative interface layer** for Nexus Launcher. It acts as a formal contract between user input and system execution, abstracting the complexities of argument parsing into a structured, type-safe data model.

By isolating the CLI definition from the execution logic, the project ensures that the core engine remains independent of the interface, facilitating easier testing and future frontend expansions.

## Core Technology: Declarative Parsing
We utilize the **Clap (v4) Derive API** to implement a "Code-as-Configuration" pattern. This allows the CLI structure to be defined through Rust metadata (Attributes), ensuring that the documentation, validation, and parsing logic are always in sync with the source code.

### Architectural Mapping Example
The following snippet demonstrates how the abstract command hierarchy is mapped to Rust types:

```rust
// The Root: Entry point and Global Context
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(short, long, global = true)]
    pub debug: bool, // Global flag accessible to all subcommands
}

// Subcommand Groups: Logical task partitioning
#[derive(Subcommand)]
pub enum Commands {
    Launch(LaunchArgs),  // Instance execution domain
    Auth(AuthArgs),      // Identity domain
    Install(InstallArgs), // Resource management domain (nested)
}

// Leaf Args: Specific operation parameters
#[derive(Args)]
pub struct LaunchArgs {
    pub instance_name: String,
    #[arg(short, long, default_value = "2048")]
    pub max_memory: u32,
}
```

## Command Hierarchy & Topology
The CLI is organized into a **multi-level tree structure** designed for scalability:

1.  **Global Level**: Manages application-wide state (e.g., `--debug`).
2.  **Domain Level (Subcommands)**: Groups related operations into namespaces such as `auth`, `java`, or `install`.
3.  **Action Level (Leaf Commands)**: Executes specific tasks within a domain.
4.  **Parameter Level (Arguments/Flags)**: Refines the behavior of a specific action.

## Integration & Data Flow
`nexus-cli` follows a **Pure Definition Crate** pattern. It does not contain any "heavy" logic or side effects.

*   **Step 1: Definition**: `nexus-cli` defines the shapes (`structs` and `enums`).
*   **Step 2: Parsing**: `nexus-main` uses `Cli::parse()` to transform `std::env::args()` into a `Cli` object.
*   **Step 3: Routing**: The application entry point performs a `match` on the parsed object and delegates work to specialized service crates (`nexus-auth`, `nexus-launch`, etc.).

## Design Principles

### 1. Type-Safe Validation
Arguments are validated at the "edge" of the application. For example, memory limits or loader types are constrained by Rust enums and range parsers, preventing invalid data from ever reaching the core logic.

### 2. Separation of Concerns
The CLI crate is unaware of how the game is launched or how authentication works. It only knows what parameters those operations require.

### 3. Discoverability
By leveraging Clap's automatic help generation, the architecture ensures that the system is self-documenting. Every command group and argument defined in the code is automatically reflected in the `--help` output.
