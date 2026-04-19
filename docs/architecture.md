# Project Architecture

This document describes the high-level architecture of Nexus Launcher.

## Overview

Nexus Launcher is a modular, high-performance Minecraft launcher written in Rust. It follows a multi-crate workspace structure to ensure separation of concerns and maintainability.

## Modules (Crates)

- **nexus-main**: The entry point of the application.
- **nexus-cli**: Handles command-line argument parsing and user interaction.
- **nexus-launch**: Core logic for constructing launch arguments and starting the JVM.
- **nexus-auth**: Manages Microsoft/Xbox Live authentication and local profiles.
- **nexus-config**: Handles user settings and instance configurations.
- **nexus-version**: Responsible for fetching and parsing Minecraft version manifests.
- **nexus-loader**: Logic for installing Mod loaders like Fabric and Quilt.
- **nexus-java**: Utilities for detecting and downloading appropriate Java runtimes.
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
