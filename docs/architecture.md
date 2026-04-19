# Project Architecture

This document describes the high-level architecture of Nexus Launcher.

> **Fast Read**: For a quick summary of the system design and launch lifecycle, see the [Architectural Overview](arch/overview.md).

## Overview

Nexus Launcher is a modular, high-performance Minecraft launcher written in Rust. It follows a multi-crate workspace structure to ensure separation of concerns and maintainability.

## Modules (Crates)

- **nexus-main**: The entry point of the application. ([Detailed Architecture](arch/main/main.md))
- **nexus-core**: Foundational infrastructure layer. Includes [Filesystem Utilities & Common Types](arch/core/core.md).
- **nexus-cli**: Handles command-line argument parsing and user interaction. ([Detailed Architecture](arch/cli/cli.md))
- **nexus-launch**: Core logic for constructing launch arguments and starting the JVM. ([Detailed Architecture](arch/launch/launch.md))
- **nexus-auth**: Manages Microsoft/Xbox Live authentication and local profiles. ([Detailed Architecture](arch/auth/auth.md))
- **nexus-config**: Handles user settings and instance configurations. ([Detailed Architecture](arch/config/config.md))
- **nexus-version**: Responsible for fetching and parsing Minecraft version manifests. Includes [Source Acquisition](arch/version/source.md), [Download & Verification](arch/version/download.md), and [Data Models](arch/version/models.md).
- **nexus-loader**: Logic for installing Mod loaders like Fabric and Quilt. ([Detailed Architecture](arch/loader/loader.md))
- **nexus-java**: Utilities for detecting and downloading appropriate Java runtimes. ([Detailed Architecture](arch/java/java.md))
- **nexus-mods**: (Optional) Module for managing mods via Cursemeta/Modrinth APIs.

## Data Flow

1. **Initialization**: `nexus-main` calls `nexus-cli` to parse arguments.
2. **Configuration**: Load user settings from `nexus-config`.
3. **Pre-launch**:
    - Verify Java environment via `nexus-java`.
    - Check/Download game files via `nexus-version`.
    - Authenticate user via `nexus-auth`.
4. **Execution**: `nexus-launch` generates the final command string and spawns the process.

## Design Principles

- **Performance**: Leverage Tokio for asynchronous I/O (downloads and file scanning).
- **Safety**: Strict use of Rust's ownership model to prevent memory leaks and race conditions.
- **Modularity**: Each component should be usable as a standalone library if possible.
