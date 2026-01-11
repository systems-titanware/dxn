# DXN (Delta Network)

**Self-hosted data management and web server platform built with Rust**

DXN enables individuals to easily manage and share their own data with others, without a third-party middleman. Configure your server via JSON/YAML, define custom business logic, integrate with third-party systems, and share resources with other servers.

---

## Overview

DXN is a Rust-based web server framework that allows you to:

- **Own Your Data**: Host your own server and maintain complete control over your data
- **Customize Easily**: Define custom functions (business logic) and integrations (third-party systems) using configuration files and your own source code
- **Share Resources**: Borrow functions, integrations, and data from other servers (with OAuth-based authorization coming soon)
- **Auto-Generated APIs**: Automatically create REST endpoints for CRUD operations on your data models
- **Flexible Web Routes**: Build web applications with server routes that can call functions, render templates, and process data

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
   cd ../dxn_public/dxn_functions
   cargo build --target wasm32-unknown-unknown --release
   cd ../../dxn-core
   ```

4. Configure your server by editing `config.json` (see [Framework.md](./Framework.md) for details)

5. Run the server:
   ```bash
   cargo run
   ```

The server will start on `http://127.0.0.1:8080`

---

## Architecture

DXN consists of three main components:

### `dxn-core`
The core server application that:
- Reads configuration from `config.json`
- Initializes database schemas (SQLite)
- Creates REST API endpoints for data models
- Manages and executes functions (WASM modules)
- Manages and executes integrations (Rust crates)
- Serves web routes with template rendering

### `dxn_public`
Shared folder containing:
- **Functions**: WASM modules (`dxn_functions/`) compiled to `wasm32-unknown-unknown`
- **Integrations**: Rust crates (`integrations/`) for third-party system connections
- **Files**: Static assets and templates (`routes/`, `assets/`, `_files/`)

### `dxn_shared`
Shared library for communication protocols and common types used across components.

---

## Key Features

### 1. Data Models
Define data schemas in `config.json` to automatically generate:
- Database tables (SQLite, with public/private database separation)
- REST API endpoints (`/api/data/{model_name}/`)
- CRUD operations (Create, Read, Update, Delete)

### 2. Functions
Write custom business logic as WASM modules:
- Compile Rust code to WebAssembly
- Call functions from server routes
- Pass typed parameters (integers, floats, strings, booleans, enums)
- Return typed results

### 3. Integrations
Connect to third-party systems:
- Write Rust crates for external integrations
- Compile and execute as separate processes
- Call integration functions from your functions or server routes
- Support for TCP-based communication

### 4. Server Routes
Build web applications:
- Define routes in `config.json`
- Render HTML templates
- Call functions to process data before rendering
- Nested route structures

### 5. Public vs Private Definitions
- **Public**: Shareable with other servers (functions, integrations, data models)
- **Private**: Only usable by the hosting server

### 6. Server-to-Server Borrowing (Future)
- Borrow functions, integrations, and data from other servers
- OAuth-based authorization (coming soon)
- Automatic scope generation for secure access

---

## Configuration

DXN is configured via `config.json` in the `dxn-core` directory. The configuration includes:

- **Data**: Database models (public/private)
- **Functions**: WASM function definitions (public/private)
- **Integrations**: Integration definitions (public/private)
- **Server**: Web route definitions (public/private)

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
- **Control**: Host your own server or use managed hosting
- **Flexibility**: Customize with your own code and configurations
- **Privacy**: Keep sensitive data private while sharing what you choose
- **Interoperability**: Share and borrow resources from other DXN servers

---

## Project Status

- ✅ Core server with REST APIs
- ✅ Data model definitions and auto-generated CRUD endpoints
- ✅ WASM-based function system
- ✅ Rust crate-based integration system
- ✅ Server routes with template rendering
- ✅ Public/private definition separation
- 🚧 Server-to-server borrowing (basic structure in place)
- 📋 OAuth-based authorization (planned)
- 📋 Automatic scope generation (planned)
- 📋 IDP (Identity Provider) integration (planned)

---

## License

See [LICENSE](./LICENSE) file for details.

