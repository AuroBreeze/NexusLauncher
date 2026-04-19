# Authentication Storage Architecture

## Abstract Purpose
The `storage` module in `nexus-auth` is the **Cryptographic Vault** of the launcher. Its abstract role is to implement a **Zero-Trust Persistence Strategy** for sensitive OAuth credentials. It ensures that even if the launcher's data folder is compromised, the tokens cannot be easily reused on a different hardware environment.

## Core Mechanisms

### 1. Hardware-Bound Key Derivation
To protect the encryption key, the module implements a **Machine-Specific KDF (Key Derivation Function)**:
- **Salt**: Uses the system's unique hardware identifier (retrieved via `machine_uid`).
- **Key Generation**: Derives a 32-byte key for AES-256 using the SHA-256 hash of the machine ID.
- **Nonce Generation**: Derives a unique 12-byte nonce for each user using the SHA-256 hash of their specific Minecraft UUID.

This ensures that the encryption is locked to both the **physical machine** and the **specific user**.

### 2. AES-GCM Encryption
The module utilizes the **AES-256-GCM** (Galois/Counter Mode) authenticated encryption algorithm:
- **Confidentiality**: Encrypts the refresh tokens so they are unreadable in the filesystem.
- **Authenticity**: GCM provides a "tag" that guarantees the data has not been tampered with. If the hardware ID changes or the file is modified, decryption will fail rather than producing garbage data.

### 3. File-Based Isolation
- **Vault Directory**: Stores credentials in a dedicated `auth_vault` folder within the workspace.
- **UUID-based Naming**: Files are named after the user's UUID, allowing for multi-account management without leaking the actual username in the filename.

## Design Philosophy

- **Hardware-Locking**: Credentials are "sticky" to the machine they were created on. This mitigates the risk of a user's `.minecraft` folder being copied to another device.
- **Fail-Safe Decryption**: If decryption fails (e.g., due to a hardware change), the system treats it as an "Invalid Session" and triggers a re-login rather than crashing.
- **Physical Destruction**: The `delete_token` function performs a physical removal of the encrypted file, ensuring that "Logout" is a permanent operation.
