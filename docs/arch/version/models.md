# Version Models Architecture

## Abstract Purpose
The `models` module is the **Domain Grammar** of the project. Its abstract role is to define the "Universal Language" used by all crates to communicate. It represents the **Immutable State** of the game's metadata—once a model is parsed, it becomes the "Ground Truth" for the entire system's logic.

## Future Evolution & Extensibility
Models are the stable core of the system, but will evolve to support:
- **Version Compatibility Layers**: Mapping legacy Minecraft JSON formats into modern internal representations.
- **Unified Metadata (Nexus-Format)**: Creating a simplified, internal metadata format that abstracts away Mojang's quirks, allowing the launcher to handle diverse game types or non-standard versions with identical logic.

## Core Responsibilities

### 1. Schema Definition
Maps Mojang's complex, deeply nested JSON files into rigid Rust structs using the `serde` framework.

### 2. Type Safety
Guarantees that once data enters the system, it is structurally valid, preventing runtime errors in higher-level logic.

## Key Data Structures
- **`VersionManifest`**: The entry point for version discovery.
- **`VersionDetail`**: The complete blueprint for a specific game version (libraries, assets, execution params).
- **`AssetIndexManifest`**: The virtual-to-physical mapping of game resources.

## Design Philosophy
- **Anemic Domain Model**: These structs are data containers with zero business logic.
- **Serde-Driven**: Leverages advanced serialization attributes to bridge external JSON formats with idiomatic Rust naming.
