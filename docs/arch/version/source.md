# Version Source Architecture

## Abstract Purpose
The `source` module is the **External Knowledge Base** of the launcher. Its abstract role is to provide a "Universal Discovery Interface" for game-related entities. It exists to solve the problem of **Resource Locality**: turning an abstract intent (e.g., "I want Minecraft 1.20") into a concrete roadmap of requirements.

It functions as the "brain" that knows where things are, even when they are spread across different remote repositories and formats.

## Future Evolution & Extensibility
The module is designed to grow from a simple Mojang-specific fetcher into a multi-provider ecosystem:
- **Third-Party Providers**: Future support for Modrinth, CurseForge, or Quilt/Fabric metadata APIs by implementing a unified discovery interface.
- **Standardized Manifest API**: Abstracting away the differences between various metadata formats into a consistent internal representation.
- **Local Source Discovery**: The same interface can be extended to resolve "Local Sources" (manually installed or cached versions) alongside remote ones.

## Core Responsibilities

### 1. Remote Manifest Orchestration
The module manages the discovery of available game versions:
- **Global Manifest Resolution**: Fetches and deserializes the top-level `version_manifest_v2.json` to identify current releases and snapshots.
- **Detailed Blueprint Resolution**: Resolves the specific URL for a version's JSON descriptor to fetch its complete dependency graph (`VersionDetail`).

### 2. Dependency Graph Flattening
Minecraft's assets and libraries are defined as recursive structures in the version metadata. The `source` module flattens these into actionable download tasks:
- **Library Resolution**: Maps abstract library coordinates to concrete `DownloadTask` objects.
- **Asset Index Processing**: Resolves the asset index JSON and maps thousands of individual object hashes into the hierarchical `objects/xx/hash` structure.

## Design Philosophy
- **Stateless Orchestration**: It produces task lists or data structures without holding side-effect-heavy state.
- **Abstraction of Complexity**: It hides the complexity of remote directory structures from the rest of the launcher.
