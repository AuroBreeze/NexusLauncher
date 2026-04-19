# Authentication Architecture

## Abstract Purpose
The `nexus-auth` crate is the **Identity & Security Layer**. Its abstract role is to manage the **Chain of Trust** between the user's Microsoft account and the Minecraft game session. It orchestrates the complex multi-step handshake required to transform a user's web-based identity into a valid game session token.

It solves the problem of **Secure Identity Delegation**, ensuring that the launcher can act on behalf of the user without ever possessing their primary password.

## Architectural Layers

The module is structured as a pipeline of token exchanges:

### 1. The Handshake Orchestrator (`lib.rs`, `utils.rs`)
- **Interactive Login**: Implements the OAuth 2.0 Device Code Flow, allowing users to sign in via a browser while the launcher waits for the token.
- **Silent Login**: Handles the "Refresh Flow," using long-lived refresh tokens to automatically renew sessions without user intervention.
- **The Token Chain**: Manages the sequential transformation of tokens:
    `Device Code -> MS Token -> Xbox Live Token -> XSTS Token -> Minecraft Token`

### 2. The Secure Vault ([Storage](storage.md))
- **Encryption Engine**: Handles the encryption and decryption of sensitive refresh tokens using hardware-bound keys (AES-GCM).
- **Persistence**: Manages the storage of these encrypted credentials in the local filesystem (`auth_vault`).

### 3. API Contract ([Models](models.md))
- **Schema Mapping**: Defines the request and response structures for the various Microsoft and Mojang endpoints.

## Coordination & Flow
Authentication is a linear dependency chain where each step validates the previous one:

```text
[User Browser] --(Device Code)--> [Microsoft API]
                                        |
                                [MS Access Token]
                                        |
                                [Xbox Live Token] --(uhs)
                                        |
                                [XSTS Token]
                                        |
                                [Minecraft Token] --(Ownership Check)--> [Launch Context]
```

## Key Architectural Decisions

### Device Code Flow
We chose the **OAuth 2.0 Device Code Flow** because it is the most secure method for a CLI application. It avoids the need for an embedded browser (WebView) or handling raw passwords, delegating the sensitive input phase to the user's trusted system browser.

### Hardware-Bound Encryption
Instead of storing refresh tokens in plain text, `nexus-auth` uses **AES-256-GCM** encryption. The encryption key is derived from a combination of a machine-unique ID and the user's UUID, making it difficult for credentials to be stolen and used on another machine.

## Design Philosophy
- **Verification-First**: The module doesn't just get a token; it performs an **Entitlement Check** to verify that the user actually owns the "Java Edition" of the game before proceeding.
- **Stateless Logic, Stateful Vault**: The logic remains purely functional and asynchronous, while the state is strictly isolated in the encrypted vault.
- **Zero-Password Storage**: The launcher never sees or stores the user's password; it only deals with revocable OAuth tokens.
