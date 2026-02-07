# DXN (Delta Network)

**Self-hosted data management and web server platform with service mesh connectivity**

DXN enables you to host your own server that connects with a mesh of other servers through P2P-based connections. Configure data models, functions, and services via JSON, customize your server's behavior, and securely access resources from other servers using OAuth scopes.

---

## Overview

DXN is a Rust-based web server framework that provides:

- **Self-Hosted Server**: Host your own server with complete control over your data
- **Service Mesh Architecture**: Connect to other servers via P2P-based service mesh for distributed functionality
- **Config-Driven Customization**: Define data models, functions, and integrations through JSON configuration
- **Auto-Generated APIs**: Automatically create REST endpoints for CRUD operations on your data models
- **Flexible Functions**: Support for WASM, Native Rust, Remote HTTP, and Script-based functions
- **Secure Resource Sharing**: Access other servers' functions, integrations, and data using OAuth scopes (planned)

---

## Quick Start

### Prerequisites

- Rust (install via [rustup](https://rustup.rs/))
- LLVM (`brew install llvm` on macOS)
- PostgreSQL 18 (optional, for future database support)

### Installation

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd dxn
   ```

2. Build the core server:
   ```bash
   cd dxn-core
   cargo build
   ```

3. Build functions (WASM modules):
   ```bash
   cd ../dxn-wasm-parser
   cargo build --target wasm32-unknown-unknown --release
   cd ../dxn-core
   ```

4. Configure your server by editing `config.json` in the root directory (see [Framework.md](./Framework.md) for details)

5. Run the server:
   ```bash
   cargo run
   ```

**Note:** Cloud registration and mobile client workflow are planned features. Currently, servers are set up manually via configuration files.


## Ongoing runtime steps

### Building WASM Functions

Build WASM function modules (using wasm-bindgen):

```bash
# Build dxn-functions WASM module
cargo build --manifest-path './dxn-functions/Cargo.toml' --target wasm32-unknown-unknown --release

# Build dxn-wasm-wallet WASM module (example)
cargo build --manifest-path './dxn-wasm-wallet/Cargo.toml' --target wasm32-unknown-unknown --release
```

### Building and Running the Core Server

```bash
# Build the core server
cargo build --manifest-path './dxn-core/Cargo.toml'

# Run the server
cargo run --manifest-path './dxn-core/Cargo.toml'
```

### Quick Build and Run (All Steps)

```bash
# Build WASM functions, then build and run core server
cargo build --manifest-path './dxn-wasm-parser/Cargo.toml' --target wasm32-unknown-unknown --release && \
cargo build --manifest-path './dxn-core/Cargo.toml' && \
cargo test --manifest-path './dxn-core/Cargo.toml' -- --show-output && \
cargo run --manifest-path './dxn-core/Cargo.toml'
```


The server will start on `http://127.0.0.1:8080`

---

## Server Spin-Up Workflow (Planned) 📋

**Cloud Registration & Mobile Setup:**
1. Register an account via third-party cloud service
2. Login to your account via the mobile client app
3. Spin up a new server from the mobile app
4. Enter a shared secret between client and server
5. Server is created and encrypted based on the shared secret

**Server Configuration:**
- Customize data models, functions, and integrations via `config.json` in the root directory
- Connect to other servers in the service mesh
- Access resources from other servers using OAuth scopes
- Preload data into KeyVault on initialization or via mobile app
- Servers operate based on CQRS principles with data models

---

## Architecture

### Service Mesh Architecture

DXN servers operate in a distributed service mesh where:
- **Primary Server**: Your main server that connects to clients
- **Local Services**: Your own specialized servers (AI, wallet, vault, etc.)
- **Public Services**: Discovered from service registry (other users' servers)
- **P2P Connections**: Servers communicate via service mesh for distributed functionality
- **Service Discovery**: Automatic discovery and registration of services
- **Health Monitoring**: Continuous health checks and automatic failover

### Core Components

**`dxn-core`**
- Core server application
- Reads configuration from `config.json` in the root directory
- Initializes database schemas (SQLite)
- Creates REST API endpoints for data models
- Manages and executes functions (WASM, Native, Remote, Script)
- Manages integrations (local and remote via service mesh)
- Serves web routes with template rendering

**`dxn_public`** (or `dxn-files`)
- Shared folder containing:
  - Functions: WASM modules, native libraries, scripts
  - Integrations: Rust crates for third-party systems
  - Files: Static assets, templates, routes

**`dxn-shared`**
- Shared library for communication protocols and common types

---

## Key Features

### Current Features ✅

**Data Models & Schemas**
- Define schemas in `config.json` or create at runtime via API
- Automatic CRUD endpoints (`/api/data/{model_name}/`)
- Schema management API (`/api/schema/`)
- Icon support for UI rendering

**Event Sourcing**
- Automatic event emission for all CRUD operations
- Event store with full audit trail
- Query events by aggregate, schema, or time range
- Rebuild data from events (`/api/events/rebuild/{schema}`)

**File Management**
- Flexible file storage with provider abstraction
- Local filesystem provider (extensible to SFTP, S3, etc.)
- Config-defined or runtime-created directories
- Full file operations: list, read, write, delete, mkdir

**Functions**
- Multiple execution types: WASM, Native Rust, Remote HTTP, Script
- Call from server routes or API endpoints
- Pass typed parameters and return results

**Integrations**
- Local integrations: Rust crates for third-party systems
- Remote integrations: Connect to other servers via service mesh

**Server Routes**
- Define routes in `config.json`
- Render HTML templates with Handlebars
- Call functions to process data before rendering

**Vault System**
- Encrypted key-value storage for sensitive data
- Reference vault values in data model definitions

**Service Mesh**
- P2P-based connections between servers
- Service discovery and registration

### Planned Features 📋

**CQRS (Command Query Responsibility Segregation)**
- Automatically generated from CRUD events for all data models specified in the data config
- Separate command and query models for optimized data access
- Servers operate based on CQRS principles with data models

**OAuth Scopes**
- Access other servers' functions, integrations, and data using OAuth scopes
- Automatic scope generation for all public definitions
- Secure, scoped access control
- When accessing another server's files, they are copied into the manifest on both servers

**KeyVault**
- Built-in key-value store in each server
- Data preloaded into KeyVault on app initialization, or via the client/mobile app
- Enhanced encryption and access management

**ZKSync Wallet Integration**
- Access requests to other services/functions/data using wallet trade capabilities
- Cryptocurrency-based access control

**Mobile Client & Cloud Registration**
- Register account via third-party cloud
- Login via mobile client
- Spin up new servers from mobile app
- Server encryption based on shared secrets

---

## Configuration

DXN is configured via `config.json` in the root directory. The configuration includes:

- **Data**: Database models (public/private) - CQRS automatically generated from CRUD events
- **Functions**: Function definitions (WASM, Native, Remote, Script) - public/private
- **Integrations**: Integration definitions (local and remote) - public/private
- **Server**: Web route definitions (public/private)
- **Service Mesh**: Local and public service configurations for distributed architecture

See [Framework.md](./Framework.md) for detailed configuration examples and technical documentation.

---

## Documentation

- **[Framework.md](./Framework.md)**: Comprehensive technical documentation covering:
  - System architecture
  - Data models and database setup
  - Functions (WASM implementation)
  - Integrations (Rust crate implementation)
  - Server routes and web capabilities
  - Server-to-server borrowing
  - OAuth future release plan

---

## Why DXN?

We believe in **ownership of your own data**. Rather than having a third-party company manage your contacts, data, and other important aspects of your life, you should be able to own this data by default and merely opt-in or opt-out of when a business, family member, friend, or colleague should have access to it.

DXN gives you:
- **Control**: Host your own server with complete data ownership
- **Flexibility**: Customize with your own code and configurations
- **Privacy**: Keep sensitive data private while sharing what you choose
- **Interoperability**: Connect to a mesh of other servers via service mesh
- **Distributed Architecture**: Run specialized services (AI, wallet, vault) across multiple servers
- **Secure Sharing**: Access other servers' resources using OAuth scopes (planned)

---

## Project Status

### Implemented ✅
- Core server with REST APIs
- Data models with auto-generated CRUD endpoints
- Schema management API (runtime creation/modification)
- Event sourcing with full audit trail
- File management with provider abstraction
- Function system (WASM, Native, Remote, Script)
- Integration system (local and remote)
- Server routes with Handlebars templates
- Vault system for encrypted storage

### Planned 📋
- OAuth-based authorization and scope management
- Additional file providers (SFTP, S3)
- Mobile client application
- ZKSync wallet integration

---

## License

See [LICENSE](./LICENSE) file for details.

