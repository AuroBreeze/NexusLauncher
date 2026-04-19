# Version Download Architecture

## Abstract Purpose
The `download` module is the **Reliable Transport & Verification Layer**. Its abstract role is to ensure the **Atomic Integrity** of the local workspace. It doesn't just "move bytes"; it guarantees that the state of the local disk precisely matches the state described by the remote metadata, regardless of network instability or filesystem errors.

## Future Evolution & Extensibility
This engine is the foundation for all data movement tasks within the Nexus ecosystem:
- **Resumable Transfers**: Implementation of HTTP Range requests to resume interrupted downloads.
- **Bandwidth Governance**: Adding global rate-limiting and task prioritization (e.g., core JARs before assets).
- **Advanced Verification**: Extending beyond SHA-1 to support SHA-256/512 or GPG signature verification for mod loaders.

## Core Responsibilities

### 1. Concurrent Execution Management
- **Task Batching**: Accepts a vector of `DownloadTask` structs to execute in bulk.
- **Concurrency Control**: Utilizes a `Semaphore` to limit the maximum number of concurrent network requests, preventing resource exhaustion.

### 2. Integrity Verification
- **Stream Hashing**: Data is hashed in real-time as it is written to disk.
- **Post-Download Validation**: Mandatory SHA-1 comparison against expected hashes. Corrupted files are immediately deleted.

### 3. Resilience and Caching
- **Smart Caching**: Local files are re-verified via hashing before skipping a download.
- **Exponential Backoff**: Automatic retries for network failures with increasing delays.

## Design Philosophy
- **Fail-Fast on Corruption**: A corrupted file is treated as a critical failure; integrity is non-negotiable.
- **Non-Blocking I/O**: Entirely asynchronous to maintain UI responsiveness.
