# Authentication Models Architecture

## Abstract Purpose
The `models` module in `nexus-auth` defines the **Security Protocol Schema**. Its abstract role is to provide the data structures required to navigate the complex, multi-tenant API ecosystem of Microsoft, Xbox, and Mojang.

## Key Data Contracts

### 1. Microsoft Identity Flow
- **`DeviceCodeResponse`**: The initial contract that defines the user's browser-based handshake (User Code, Verification URI).
- **`MicrosoftToken`**: The foundational identity token that includes the sensitive `refresh_token` used for silent login.

### 2. Xbox Live Integration
- **`XboxLiveResponse`**: Represents the bridge between a general Microsoft identity and an Xbox identity.
- **`Xui` (Xbox User Information)**: Extracts the **UHS (User Hash)**, a critical security parameter required for all subsequent Minecraft service requests.

### 3. Minecraft Service Identity
- **`MinecraftAuthResponse`**: The final token required to launch the game.
- **`MinecraftProfile`**: The semantic profile of the user (Display Name and UUID).
- **`EntitlementsResponse`**: The data contract used for verifying game ownership (License check).

## Design Philosophy

- **Field Isolation**: Only the fields strictly required by the launcher's orchestration logic are deserialized, reducing the surface area for API change breakage.
- **Type Safety**: Uses Rust's strong typing (e.g., `u64` for intervals, nested `DisplayClaims`) to ensure that API responses are valid before they are processed by the orchestration layer.
- **Naming Conventions**: Uses `#[serde(rename_all = "PascalCase")]` where appropriate to bridge the gap between Microsoft's PascalCase JSON and Rust's snake_case naming conventions.
