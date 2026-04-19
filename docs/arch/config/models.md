# Configuration Models Architecture

## Abstract Purpose
The `models` module defines the **Information Schema** of the launcher. Its abstract role is to provide a structured representation of the launcher's universe, categorizing data based on its source and impact.

## Data Domains

### 1. User Domain (`UserConfig`)
Represents the **Social & Identity State**. It maps usernames to internal UUIDs and handles the distinction between offline and online profiles. This model is the foundation for the "Who" aspect of the launch process.

### 2. Execution Domain (`LaunchConfig`)
Represents the **Technical Environment State**. It stores the mapping of discovered Java runtimes and global execution flags. This model is highly dynamic, as it is updated every time a new Java environment is scanned or downloaded.

## Design Philosophy
- **Explicit Hierarchy**: Nested structs are used to create a clear, logical grouping of related settings.
- **Serde Integration**: Every model is tightly coupled with `serde` to ensure that the internal Rust representation is perfectly mirrored in the external TOML format.
- **Anemic Data Structures**: These models are primarily data containers, with logic like validation residing in the implementation files (`userconfig.rs`, `launchconfig.rs`).
