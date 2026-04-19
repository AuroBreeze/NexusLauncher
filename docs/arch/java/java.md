# Java Management Architecture

## Abstract Purpose
The `nexus-java` crate is the **Runtime Environment Layer**. Its abstract role is to ensure the **Execution Compatibility** of the game. It solves the problem of "Version Mismatch" by abstracting away the discovery, validation, and acquisition of Java Runtime Environments (JREs) required to run different versions of Minecraft.

It functions as a bridge between the launcher's requirement (e.g., "Needs Java 17") and the host system's capabilities.

## Future Evolution & Extensibility
As the project grows, `nexus-java` will evolve from a simple downloader into a comprehensive JVM manager:
- **JVM Flavor Support**: Support for multiple distributions (Adoptium, GraalVM, Amazon Corretto) to allow for performance tuning.
- **Optimization Profiles**: Future ability to recommend specific JVM arguments based on the detected hardware and selected Java version.
- **Sandboxed Runtimes**: Moving towards isolated, per-instance JREs to prevent system-wide Java conflicts.
- **Auto-Cleanup**: Intelligent management of downloaded runtimes to prune unused or obsolete versions.

## Core Responsibilities

### 1. Environment Discovery & Scanning
The module implements a multi-stage scanning strategy to find existing Java installations:
- **Environmental Context**: Checks `JAVA_HOME` and the system `PATH`.
- **Project Context**: Scans the internal `runtimes/` directory.
- **System Context**: Scans common OS-specific installation paths (e.g., `/usr/lib/jvm`).
- **De-duplication**: Intelligent merging of results to ensure the same installation isn't listed multiple times under different symlinks.

### 2. Version Validation & Parsing
It acts as a **Semantic Version Interpreter**:
- **Executable Verification**: Instead of trusting filenames, it executes `java -version` to confirm the binary is functional and to extract its true identity.
- **Heuristic Parsing**: Translates varied Java version strings (e.g., `1.8.0_382` vs `17.0.8`) into a unified internal representation of "Major Version."

### 3. Automated Runtime Acquisition
When the system environment cannot satisfy a requirement, `nexus-java` acts as a **Platform-Aware Downloader**:
- **Dynamic Detection**: Detects OS and Architecture at runtime to fetch the correct binary (e.g., `linux-x64` vs `windows-aarch64`).
- **Stream-to-Disk**: Downloads and extracts JREs from reliable sources like the Adoptium API, providing real-time progress feedback.

## Design Philosophy

- **Platform Neutrality**: Native handling of both `.tar.gz` and `.zip` archives ensures seamless operation across Linux, macOS, and Windows.
- **Deterministic Resolution**: Prioritizes local, launcher-managed runtimes over system-wide ones to ensure a predictable execution environment.
- **Resilience**: Implements robust error handling for I/O and process execution, ensuring that one corrupted Java path doesn't crash the entire scanning process.
