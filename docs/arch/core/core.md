# Core Architecture

## Abstract Purpose
The `nexus-core` crate is the **Platform Abstraction & Common Language Layer**. Its abstract role is to provide a "Virtual Environment" for the rest of the application. It shields higher-level logic from the messy realities of the operating system, hardware paths, and shared data types. It is the **Foundational Infrastructure** on which the entire Nexus Launcher is built.

## Future Evolution & Extensibility
As the launcher matures, `core` will become the central hub for:
- **Hardware/System Discovery**: Abstracting GPU detection, memory limits, and CPU architecture checks.
- **Cross-Crate Traits**: Defining the core traits (e.g., `Component`, `StorageProvider`) that other crates must implement.
- **IPC & Event Definitions**: Housing the shared message types for inter-process communication or internal event bus systems.

## Core Responsibilities

### 1. Unified FileSystem Abstraction
Acts as the "Single Source of Truth" for the Minecraft environment's directory structure, ensuring cross-platform consistency.

### 2. Common Type Definitions
Centralizes shared types (like `AnyError`) and error handling patterns to ensure interoperability across the workspace.

### 3. Repository & Mapping Logic
Handles the translation of abstract metadata (like Maven coordinates) into concrete system resources.

## Design Philosophy
- **Stability**: Prioritizes a stable, minimal-change API as the most depended-upon crate.
- **Zero-Logic Policy**: Contains only infrastructure utilities and data structures; no business logic.
- **Platform Agnostic**: All interactions are designed to work seamlessly on Windows, Linux, and macOS.
