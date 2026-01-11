
# ⚙️ DXN

*Host and configure your rust based webserver by JSON or YAML.*

---

## Summary

DXN (Delta Network), allows anyone to host their own web servers and interact with it via the open-source mobile app or web app.

Simply register an account online to generate a new server in seconds to host your own:

* Blog
* Media gallery to share with friends
* Social media or messaging site to contact others
* Digital Wallet to manage off-ramp transactions
and more..

---

## Setup

### Self-Hosted Setup

1. Clone the repository and build the core server (see [README.md](./README.md) for detailed instructions)
2. Configure your server by editing `config.json` in the `dxn-core` directory
3. Build your functions (WASM modules) and integrations (Rust crates)
4. Run the server: `cargo run` in the `dxn-core` directory
5. Access your server at `http://127.0.0.1:8080`

### Managed Hosting (Future)

1. Browse to dxn.io, create a new account
2. Choose to host your own or fork your own version of the server or to have one managed
3. Connect to the server with the mobile app (when available)
4. Start creating content, sharing or modifying your server

## Architecture

The core application is broken up into four core components

1. System

1.1. Serialization
Helper functions that enable JSON and YAML serialization

1.2. Files
Helper functions that enable you to easily read, write or modify files in a shared folder on your server

1.3. Server
Manages the creation of a REST-based web application, with two core server components:

1.3.1. Data
CRUD API endpoints to modify and change all data models you defined in the data section

**Example:** For the profile data model defined below, there will be API endpoints automatically generated:

**List all profiles:**
```
GET http://127.0.0.1:8080/api/data/profile/
Response: [
    {
        "id": 1,
        "email": "john.smith@example.com",
        "phone": "+64 27 00 99 00"
    },
    ...
]
```

**Get specific profile:**
```
GET http://127.0.0.1:8080/api/data/profile/1
Response: {
    "id": 1,
    "email": "john.smith@example.com",
    "phone": "+64 27 00 99 00"
}
```

**Create new profile:**
```
POST http://127.0.0.1:8080/api/data/profile/
Body: {
    "email": "jane.doe@example.com",
    "phone": "+64 21 00 88 00"
}
```

**Update profile:**
```
PUT http://127.0.0.1:8080/api/data/profile/1
Body: {
    "email": "john.smith.updated@example.com",
    "phone": "+64 27 00 99 00"
}
```

**Delete profile:**
```
DELETE http://127.0.0.1:8080/api/data/profile/1
```
1.3.2. Web
API endpoints to serve content or perform operations on your server. Server routes can render files, call functions to process data, and support nested route structures.

**Basic Route (File Rendering):**
If we wanted to create a blog site, we could define routes like this:
```json
{
    "name": "test",
    "file": "post.md",
    "routes": [
        {
            "name": "baby-test",
            "file": "subpage.html",
            "routes": []
        }
    ]
}
```
This creates a GET endpoint at `http://127.0.0.1:8080/server/test` that renders the content of `post.md` from the `dxn_public/routes/` directory.

**Route with Function Processing:**
You can also process data before rendering by calling a function:
```json
{
    "name": "tester",
    "function": "parse_markdown",
    "params": ["post.md"],
    "file": "tester.html",
    "routes": []
}
```
- This server route will call a function called `parse_markdown`
- The function receives the parameter `"post.md"` (a file path)
- The function processes the markdown file and returns processed content
- The result is passed to the template `tester.html` for rendering
- This allows you to pre-process data before returning it to the client

**Route with Typed Function Parameters:**
For functions with typed parameters, you can pass multiple values:
```json
{
    "name": "test",
    "file": "post.md",
    "function": "typed_params",
    "params": [30, 12],
    "routes": [
        {
            "name": "baby-test",
            "file": "subpage.html",
            "routes": []
        }
    ]
}
```
- The function `typed_params` receives two integer parameters: `30` and `12`
- The function result can be used in the template or returned directly

**Nested Routes:**
Routes support nested structures for hierarchical URL paths:
- Parent route: `/server/test`
- Child route: `/server/test/baby-test`

**Route Configuration:**
- `name`: URL path segment (e.g., "test" creates `/server/test`)
- `file`: Template file to render (from `dxn_public/routes/`)
- `function`: (Optional) Function to call before rendering
- `params`: (Optional) Array of parameters to pass to the function
- `routes`: (Optional) Array of nested child routes


1.4. Vault

The Vault system allows you to store sensitive data in an encrypted key-value store on the server. Vault values can be referenced in data model field definitions using the `{vault.path.to.value}` syntax.

**Vault Usage:**
- Store sensitive information (API keys, passwords, tokens)
- Reference vault values in data model field definitions
- Automatic encryption at rest
- Access controlled by the server

**Example:**
```json
{
    "name": "email",
    "datatype": "text",
    "value": "{vault.profile.email}"
}
```

The vault value `profile.email` will be automatically populated when the data model is used.

2. Data

2.1. Data Models

Data models define the structure of your database tables. You can define both public and private data models. Public models can be shared with other servers, while private models are only accessible by the hosting server.

Example: The below example will create a new table in your public database called 'profile', this model has two columns:
- email of type text
- phone of type text

```json
"data": {
    "public": [
        {
            "name": "profile",
            "version": 1,
            "db": "public",
            "fields": [
                {
                    "name": "email",
                    "datatype": "text",
                    "value": "{vault.profile.email}"
                },
                {
                    "name": "phone",
                    "datatype": "text",
                    "value": "{vault.profile.phone}"
                }
            ]
        }
    ],
    "private": [
        {
            "name": "wallet",
            "version": 1,
            "db": "private",
            "fields": [
                {
                    "name": "address",
                    "datatype": "text",
                    "value": "{vault.wallet.address}"
                }
            ]
        }
    ]
}
```

**Database Separation:**
- `"db": "public"` - Creates tables in the public database, accessible via public APIs
- `"db": "private"` - Creates tables in the private database, only accessible internally

**Field Types:**
- `text` - String/text data
- `number` - Numeric data (integer or float)
- `boolean` - True/false values

**Vault Integration:**
Fields can reference vault values using `{vault.path.to.value}` syntax, which will be automatically populated from the encrypted vault storage.

**Auto-Generated API Endpoints:**
For each data model, DXN automatically creates REST endpoints:
- `GET /api/data/{model_name}/` - List all records (with pagination)
- `GET /api/data/{model_name}/{id}` - Get a specific record
- `POST /api/data/{model_name}/` - Create a new record
- `PUT /api/data/{model_name}/{id}` - Update a record
- `DELETE /api/data/{model_name}/{id}` - Delete a record

3. Functions

Functions allow you to define custom business logic that can be executed from server routes or called by other functions. Functions are compiled as WebAssembly (WASM) modules for secure, isolated execution.

3.1. Function System Overview

Functions are:
- Written in Rust
- Compiled to WebAssembly (WASM) targeting `wasm32-unknown-unknown`
- Loaded and executed using Wasmtime runtime
- Can accept typed parameters (i32, f64, String, bool, enums)
- Can return typed results
- Can call integrations and access file system

3.2. Public vs Private Functions

Functions can be defined as public or private:
- **Public Functions**: Can be shared with and called by other servers (when borrowing is implemented)
- **Private Functions**: Only accessible by the hosting server

3.3. Function Configuration

Functions are defined in `config.json` under the `functions` section:

```json
"functions": {
    "public": [
        {
            "name": "parse_markdown",
            "version": 1,
            "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
            "functionName": "parse_markdown"
        },
        {
            "name": "typed_params",
            "version": 1,
            "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
            "functionName": "typed_params",
            "parameters": ["i32", "i32"],
            "return": "i32"
        }
    ],
    "private": [
        {
            "name": "internal_processing",
            "version": 1,
            "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
            "functionName": "internal_processing"
        }
    ]
}
```

**Configuration Fields:**
- `name`: Unique identifier for the function (used when calling from routes)
- `version`: Version number for function versioning
- `path`: Path to the compiled WASM module file
- `functionName`: Name of the exported function in the WASM module
- `parameters`: (Optional) Array of parameter types for type checking
- `return`: (Optional) Return type for type checking

3.4. Writing Functions

Functions are written in Rust and compiled to WASM. Example function implementations:

```rust
// Simple function with no parameters
#[no_mangle]
pub extern "C" fn no_params() {
    let result = (10 * 1) + 5;
    println!("COMPUTED RESULT {}", result);
}

// Function with return value
#[no_mangle]
pub extern "C" fn no_params_with_result() -> i32 {
    let result = (10 * 1) + 5;
    result
}

// Function with typed parameters
#[no_mangle]
pub extern "C" fn typed_params(left: i32, right: i32) -> i32 {
    left + right
}

// Function with string parameter
#[no_mangle]
pub extern "C" fn parse_markdown(path: String) -> String {
    // Read file, process markdown, return HTML
    // Can also call integrations from within functions
    let file = crate::system::files::manager::read_file(&path).unwrap_or(String::from("err"));
    // Process and return result
    format!("Parsed markdown: {}", file)
}
```

3.5. Calling Functions from Server Routes

Functions can be called from server routes defined in the `server` section of `config.json`:

```json
{
    "name": "blog-post",
    "function": "parse_markdown",
    "params": ["post.md"],
    "file": "post.html",
    "routes": []
}
```

When a request is made to `/server/blog-post`:
1. The function `parse_markdown` is called with parameter `"post.md"`
2. The function processes the markdown file
3. The result is passed to the template `post.html` for rendering
4. The rendered HTML is returned to the client

3.6. Function Parameters

Functions support various parameter types:
- `i32` - 32-bit integer
- `f64` - 64-bit floating point
- `String` - Text/string data
- `bool` - Boolean values
- Custom enums (via serialized JSON)

Parameters are passed as an array in the route configuration and are type-checked at runtime.

3.7. Function Execution

Functions are executed in isolated WASM runtime environments:
- Each function call creates a new execution context
- Functions cannot directly access server state (security isolation)
- Functions can call integrations for external operations
- Functions can read/write files in the `dxn_public` directory

4. Integrations

Integrations allow you to connect your DXN server to third-party systems and external services. Integrations are written as Rust crates and compiled as separate processes that communicate with the core server.

4.1. Integration System Overview

Integrations are:
- Written as standalone Rust crates
- Compiled and executed as separate processes
- Can communicate via TCP or standard I/O
- Can expose multiple functions
- Can be shared between servers (public integrations)

4.2. Public vs Private Integrations

Integrations can be defined as public or private:
- **Public Integrations**: Can be shared with and used by other servers (when borrowing is implemented)
- **Private Integrations**: Only accessible by the hosting server

4.3. Integration Configuration

Integrations are defined in `config.json` under the `integrations` section:

```json
"integrations": {
    "public": [
        {
            "name": "parser",
            "path": "integrations/parser",
            "version": "1",
            "owner": "UUID",
            "functions": [
                {
                    "name": "parse_html",
                    "params": {
                        "parameter1": "hello world"
                    }
                }
            ],
            "crates": "html2markdown"
        },
        {
            "name": "wallet",
            "path": "integrations/wallet",
            "version": "1",
            "owner": "UUID",
            "functions": [
                {
                    "name": "create_transaction",
                    "params": {}
                }
            ],
            "crates": "zksync"
        }
    ],
    "private": [
        {
            "name": "internal_api",
            "path": "integrations/internal_api",
            "version": "1",
            "functions": [
                {
                    "name": "process_data",
                    "params": {}
                }
            ]
        }
    ]
}
```

**Configuration Fields:**
- `name`: Unique identifier for the integration
- `path`: Relative path to the integration crate (from `dxn_public/integrations/`)
- `version`: Version string for the integration
- `owner`: (Optional) UUID of the server that owns this integration
- `functions`: Array of functions exposed by the integration
- `crates`: (Optional) List of external crates/dependencies used

4.4. Writing Integrations

Integrations are standalone Rust crates. Example integration structure:

```
dxn_public/integrations/parser/
├── Cargo.toml
└── src/
    └── main.rs
```

Example integration implementation:

```rust
// integrations/parser/src/main.rs
use std::io::{self, Read, Write};

fn main() {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    
    // Process the input (e.g., parse HTML to Markdown)
    let result = parse_html(&buffer);
    
    // Write result to stdout
    io::stdout().write_all(result.as_bytes()).unwrap();
}

fn parse_html(html: &str) -> String {
    // Integration logic here
    // This could call external libraries, APIs, etc.
    format!("Converted: {}", html)
}
```

4.5. Integration Compilation and Execution

When the server starts:
1. Each integration is compiled using `cargo build` in its directory
2. Compiled binaries are stored for execution
3. Integrations can be invoked via:
   - Direct process execution (stdin/stdout)
   - TCP communication (for long-running integrations)
   - Function calls from WASM functions

4.6. Calling Integrations

Integrations can be called from:
- **Functions**: WASM functions can call integrations using the integration manager
- **Server Routes**: (Future) Direct integration calls from routes

Example of calling an integration from a function:

```rust
#[no_mangle]
pub extern "C" fn parse_markdown(path: String) -> String {
    let file = crate::system::files::manager::read_file(&path).unwrap_or(String::from("err"));
    
    // Call the parser integration
    let result = integrations::manager::run("parser", "parse", Some(&file));
    result.unwrap_or(String::from("err"))
}
```

4.7. TCP-Based Integrations

For long-running integrations or those requiring persistent connections, integrations can communicate via TCP:

```rust
// Integration can connect to server via TCP
use tokio::net::TcpStream;
use dxn_shared::{RequestMessage, ResponseMessage};

async fn connect_to_server() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    // Send/receive messages
}
```

The server can also initiate TCP connections to integrations for bidirectional communication.

4.8. Integration Functions

Each integration can expose multiple functions. Functions are defined in the integration configuration and can accept parameters. The integration's main process handles routing to the appropriate function based on the command passed.



---

5. Server-to-Server Borrowing

DXN servers can "borrow" functions, integrations, and data definitions from other servers. This enables code reuse, sharing of common functionality, and building distributed systems of interconnected DXN servers.

5.1. Current Implementation

The current implementation provides the foundation for borrowing:

**Public Definitions:**
- Functions, integrations, and data models marked as `public` in `config.json` are intended to be shareable
- Public definitions are exposed in the server's public API
- The structure supports adding external server definitions to the configuration

**Public vs Private Separation:**
- `public`: Definitions that can be shared with other servers
- `private`: Definitions only usable by the hosting server

**Current Limitations:**
- Manual configuration required to add borrowed definitions
- No automatic discovery mechanism
- No authentication/authorization for borrowing
- No scope-based access control

5.2. How Public Definitions Work

When you define a function, integration, or data model as public:

```json
{
    "functions": {
        "public": [
            {
                "name": "parse_markdown",
                "version": 1,
                "path": "../dxn_public/dxn_functions/target/wasm32-unknown-unknown/release/dxn_functions.wasm",
                "functionName": "parse_markdown"
            }
        ]
    }
}
```

This definition becomes available for other servers to discover and borrow. The server exposes metadata about public definitions through its API.

5.3. Future: OAuth-Based Borrowing

The future implementation will add secure, OAuth-based borrowing:

**Discovery:**
- Servers will be able to discover other DXN servers
- Public definition catalogs will be queryable
- Server metadata (version, capabilities, owner) will be exposed

**Borrowing Process:**
1. Server A discovers Server B's public definitions
2. Server A requests access to specific definitions
3. OAuth authorization flow initiated
4. Server B generates scopes for requested definitions
5. Server A receives access tokens with appropriate scopes
6. Server A can use borrowed definitions with token-based authentication

**Configuration for Borrowing:**
```json
{
    "borrowed": {
        "functions": [
            {
                "server": "https://server-b.example.com",
                "name": "parse_markdown",
                "version": 1,
                "scope": "function:parse_markdown:read",
                "token": "oauth_access_token_here"
            }
        ],
        "integrations": [
            {
                "server": "https://server-c.example.com",
                "name": "payment_processor",
                "version": "2",
                "scope": "integration:payment_processor:execute",
                "token": "oauth_access_token_here"
            }
        ],
        "data": [
            {
                "server": "https://server-d.example.com",
                "name": "user_profile",
                "version": 1,
                "scope": "data:user_profile:read",
                "token": "oauth_access_token_here"
            }
        ]
    }
}
```

**Benefits:**
- Code reuse across servers
- Shared integrations for common services
- Distributed data access
- Secure, scoped access control

---

6. OAuth Future Release Plan

OAuth 2.0 and OpenID Connect integration will be baked into the core DXN project, enabling secure server-to-server communication, user authentication, and fine-grained access control for borrowing resources.

6.1. OAuth 2.0 / OpenID Connect Integration

**Architecture:**
- DXN servers will act as both OAuth 2.0 Resource Servers and Authorization Servers
- Support for standard OAuth 2.0 flows:
  - Authorization Code Flow (for web applications)
  - Client Credentials Flow (for server-to-server)
  - Refresh Token Flow
- OpenID Connect for user authentication and identity

**Implementation Approach:**
- Use established Rust OAuth libraries (e.g., `oauth2`, `openidconnect`)
- JWT (JSON Web Tokens) for access tokens
- Token introspection endpoints
- Token revocation support

6.2. Automatic Scope Generation

When you define any function, integration, or data model, DXN will automatically generate OAuth scopes:

**Scope Naming Convention:**
- Functions: `function:{function_name}:{action}`
  - Example: `function:parse_markdown:execute`
- Integrations: `integration:{integration_name}:{action}`
  - Example: `integration:payment_processor:execute`
- Data: `data:{model_name}:{action}`
  - Example: `data:profile:read`, `data:profile:write`

**Scope Actions:**
- `read` - Read/query access
- `write` - Create/update access
- `delete` - Delete access
- `execute` - Execute/run access (for functions and integrations)

**Automatic Scope Registration:**
```json
{
    "functions": {
        "public": [
            {
                "name": "parse_markdown",
                "scopes": [
                    "function:parse_markdown:execute"
                ]
            }
        ]
    },
    "integrations": {
        "public": [
            {
                "name": "payment_processor",
                "scopes": [
                    "integration:payment_processor:execute"
                ]
            }
        ]
    },
    "data": {
        "public": [
            {
                "name": "profile",
                "scopes": [
                    "data:profile:read",
                    "data:profile:write"
                ]
            }
        ]
    }
}
```

6.3. IDP (Identity Provider) Access Patterns

DXN will support multiple identity provider patterns:

**Server as IDP:**
- Each DXN server can act as its own identity provider
- Users authenticate directly with the server
- Server issues OAuth tokens for its own resources

**External IDP Integration:**
- Support for external identity providers (Google, GitHub, custom OIDC providers)
- Users authenticate with external IDP
- DXN server validates external tokens
- Maps external identity to local user accounts

**Federated Identity:**
- Multiple DXN servers in a federation
- Cross-server authentication
- Trust relationships between servers

6.4. Authorization Flows for Borrowing

**Server-to-Server Borrowing Flow:**

1. **Discovery Phase:**
   ```
   Server A → Server B: GET /api/public/definitions
   Server B → Server A: { functions: [...], integrations: [...], data: [...] }
   ```

2. **Authorization Request:**
   ```
   Server A → Server B: POST /oauth/authorize
   {
       "client_id": "server-a-id",
       "scope": "function:parse_markdown:execute integration:payment:execute",
       "response_type": "code",
       "redirect_uri": "https://server-a.example.com/oauth/callback"
   }
   ```

3. **Token Exchange:**
   ```
   Server A → Server B: POST /oauth/token
   {
       "grant_type": "authorization_code",
       "code": "authorization_code",
       "client_id": "server-a-id",
       "client_secret": "server-a-secret"
   }
   Server B → Server A: {
       "access_token": "jwt_token",
       "token_type": "Bearer",
       "expires_in": 3600,
       "scope": "function:parse_markdown:execute integration:payment:execute",
       "refresh_token": "refresh_token"
   }
   ```

4. **Resource Access:**
   ```
   Server A → Server B: POST /api/execute/function/parse_markdown
   Headers: { "Authorization": "Bearer jwt_token" }
   ```

6.5. Scope-Based Access Control

**Fine-Grained Permissions:**
- Each scope grants specific permissions
- Scopes can be combined for multiple permissions
- Token validation checks scope requirements

**Example Scope Combinations:**
- `function:parse_markdown:execute` - Can execute parse_markdown function
- `data:profile:read data:profile:write` - Can read and write profile data
- `integration:payment:execute` - Can execute payment integration

**Scope Validation:**
When a borrowed resource is accessed:
1. Extract scopes from the access token
2. Check if required scope is present
3. Deny access if scope is missing
4. Log access attempts for auditing

6.6. Technical Implementation Details

**Token Storage:**
- Access tokens stored securely (encrypted at rest)
- Token expiration and refresh handling
- Token revocation lists

**Security Considerations:**
- HTTPS required for all OAuth flows
- PKCE (Proof Key for Code Exchange) for public clients
- Token signing with RS256 (RSA)
- Secure token storage and transmission

**API Endpoints:**
- `GET /oauth/authorize` - Authorization endpoint
- `POST /oauth/token` - Token endpoint
- `POST /oauth/introspect` - Token introspection
- `POST /oauth/revoke` - Token revocation
- `GET /api/public/definitions` - Public definitions catalog
- `GET /.well-known/openid-configuration` - OpenID Connect discovery

**Configuration:**
```json
{
    "oauth": {
        "enabled": true,
        "issuer": "https://server.example.com",
        "authorization_endpoint": "/oauth/authorize",
        "token_endpoint": "/oauth/token",
        "jwks_uri": "/.well-known/jwks.json",
        "scopes_supported": [
            "function:*:execute",
            "integration:*:execute",
            "data:*:read",
            "data:*:write"
        ],
        "idp": {
            "type": "internal",
            "external_providers": []
        }
    }
}
```

6.7. Migration Path

**Phase 1: Foundation**
- OAuth 2.0 server implementation
- Basic scope generation
- Token issuance and validation

**Phase 2: Borrowing**
- Server discovery
- Borrowing workflow
- Scope-based access control

**Phase 3: IDP Integration**
- External IDP support
- Federated identity
- User management

**Phase 4: Advanced Features**
- Token introspection
- Audit logging
- Rate limiting per scope
- Dynamic scope requests

---

## Why

We believe in **ownership of your own data**. Rather than having a third-party company manage your contacts, data, and other important aspects of your life, you should be able to own this data by default and merely opt-in or opt-out of when a business, family member, friend, or colleague should have access to it.

DXN gives you:
- **Control**: Host your own server or use managed hosting
- **Flexibility**: Customize with your own code and configurations
- **Privacy**: Keep sensitive data private while sharing what you choose
- **Interoperability**: Share and borrow resources from other DXN servers

---