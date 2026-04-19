# Fabric Integration Architecture

## Abstract Purpose
The `fabric` module (implemented in `fabric.rs`) is the **Domain-Specific Resolver** for the Fabric mod loader. Its abstract role is to translate Fabric's specific metadata ecosystem into the launcher's universal resource model.

It encapsulates the rules and API interactions unique to the Fabric project, shielding the rest of the launcher from Fabric's internal versioning and profile formats.

## Core Responsibilities

### 1. Metadata Discovery
- **Version Mapping**: Resolves the mapping between Minecraft game versions and compatible Fabric Loader versions via the Fabric Meta API.
- **Stability Filtering**: Implements logic to prefer "stable" builds over experimental ones to ensure a reliable user experience.

### 2. Profile Localization
- **Blueprint Acquisition**: Downloads the "Loader Profile," which is a JSON document containing the `MainClass` (usually `net.fabricmc.loader.launch.knot.KnotClient`) and a list of required libraries.
- **Persistence**: Saves these profiles within the game instance folder (e.g., `fabric_profile_1.20.1_0.14.22.json`), enabling future launches to bypass the network.

### 3. Classpath Preparation
- **Library Resolution**: Translates Fabric's Maven-style library coordinates into physical download URLs.
- **Integration with Core**: Leverages `nexus-core`'s Maven utilities to determine local storage paths and `nexus-version`'s download engine to acquire the JAR files.

## Implementation Patterns

### File Discovery
The module includes utility functions (`find_game_json`, `find_fabric_json`) that scan the filesystem to verify the presence of base game files and existing loader profiles. This adds a layer of **Physical Validation** before attempting a launch.

## Design Philosophy
- **Isolation**: Fabric-specific logic is entirely contained here. If the launcher adds support for Forge, a new `forge.rs` would be created with a similar interface, keeping the code clean.
- **Network Efficiency**: Fetches the minimum amount of data required to boot the loader.
- **Traceability**: Provides detailed logging of the resolution process to help users diagnose why a specific loader version might not be available.
