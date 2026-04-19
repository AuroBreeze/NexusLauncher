# Mod Loader Architecture

## Abstract Purpose
The `nexus-loader` crate is the **Extensibility & Modding Layer**. Its abstract role is to provide a **Component Injection Engine**—handling the discovery, installation, and integration of third-party mod loaders (like Fabric and Quilt) into the standard Minecraft environment.

It solves the problem of "Environment Transformation," allowing the launcher to modify the game's execution path (Main Class) and dependency graph (Classpath) dynamically based on user-selected loaders.

## Architectural Layers

The module is structured around the lifecycle of a mod loader:

### 1. Discovery & Metadata Layer ([Fabric](fabric.md))
- **Remote Resolution**: Interacts with loader-specific APIs (e.g., Fabric Meta) to resolve the latest stable versions for a given game version.
- **Profile Management**: Fetches and caches the "Profile" (a JSON blueprint) that defines how the loader should be integrated.

### 2. Dependency Management ([Models](models.md))
- **Mapping**: Transforms loader-specific library coordinates into concrete download tasks.
- **Resource Integration**: Works with `nexus-version` to ensure loader libraries are downloaded and linked into the game instance's local object pool.

## Coordination & Flow
`nexus-loader` acts as a middleman during the preparation phase:

```text
[nexus-main] -> [nexus-loader]
                      |
        +-------------+-------------+
        |                           |
[Fabric API] --(Fetch Metadata)--> [Local JSON Profile]
                                    |
                          +---------+---------+
                          |                   |
                  [Download Libs]      [Export MainClass]
                          |                   |
                          +--------+----------+
                                   |
                          [nexus-launch] (Final Integration)
```

## Key Architectural Decisions

### Late-Bound Integration
The architecture uses a **Late-Binding** approach. Instead of modifying the core Minecraft `version.json`, the loader creates its own profile. The `nexus-launch` engine then merges these two profiles at runtime. This prevents the launcher from corrupting original game files and allows for easy uninstallation of mod loaders.

### Object Pooling for Loader Libs
To maintain consistency with the core game's storage strategy, loader libraries are downloaded into a local object pool and then linked. This ensures that the same version of a loader shared across multiple instances only occupies disk space once.

## Design Philosophy
- **Modular by Loader Type**: While currently focused on Fabric, the architecture is designed to be extensible to other loaders (Quilt, Forge) by following the same "Resolve -> Profile -> Link" pattern.
- **Persistence of Intent**: Loader profiles are saved locally to enable offline launches once the initial installation is complete.
- **Graceful Failure**: If a loader cannot be resolved or installed, the system provides clear feedback while maintaining the integrity of the base game files.
