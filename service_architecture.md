# Service Mesh Architecture - High-Level Solution Plan

## 1. Architecture Overview

### Core Concept
Transform DXN from a single-server architecture to a distributed service mesh where:
- **Primary Server**: User's main server that connects to clients
- **Local Services**: User's own specialized servers (AI, wallet, vault, etc.)
- **Public Services**: Discovered from a service registry (other users' servers)
- **Service Mesh**: Manages discovery, routing, and communication between services

### Component Diagram

```
┌─────────────────────────────────────────────┐
│         Primary Server (dxn-core)           │
│  ┌───────────────────────────────────────┐ │
│  │  Service Mesh Client                  │ │
│  │  - Service Discovery                  │ │
│  │  - Service Registry                    │ │
│  │  - Health Monitoring                   │ │
│  │  - Request Routing                     │ │
│  └───────────────────────────────────────┘ │
│  ┌───────────────────────────────────────┐ │
│  │  Integration Manager (Enhanced)        │ │
│  │  - Local integrations (current)        │ │
│  │  - Remote service calls (new)          │ │
│  └───────────────────────────────────────┘ │
└──────────────┬──────────────────────────────┘
               │
       ┌───────┴───────────────┐
       │                       │
┌──────▼──────┐      ┌─────────▼─────────┐
│   Service   │      │  Local Services   │
│  Registry   │      │  (User's Own)     │
│             │      │  - AI Server      │
│  - Catalog  │      │  - Wallet Server  │
│  - Discovery│      │  - Vault Server   │
│  - Health   │      └───────────────────┘
└──────┬──────┘
       │
┌──────▼──────────────────────────┐
│  Public Services (Others')      │
│  Discovered & Connected         │
└─────────────────────────────────┘
```

## 2. Core Components

### 2.1 Service Mesh Client Module
**Location:** `dxn-core/src/integrations/service_mesh/`

**Responsibilities:**
- Service discovery from registry
- Service registration (for local services)
- Health checking and monitoring
- Request routing and load balancing
- OAuth token management
- Service caching

**Key Files:**
- `client.rs` - Main service mesh client
- `discovery.rs` - Service discovery logic
- `registry.rs` - Registry communication
- `health.rs` - Health check monitoring
- `router.rs` - Request routing

### 2.2 Enhanced Integration Manager
**Location:** `dxn-core/src/integrations/manager.rs` (enhance existing)

**New Capabilities:**
- Support both local (process) and remote (HTTP) integrations
- Unified API for calling integrations regardless of type
- Automatic routing based on integration type
- Error handling and retries

### 2.3 Service Registry (External Service)
**Location:** Separate service (can be hosted by DXN or community)

**Responsibilities:**
- Service catalog and metadata
- Service discovery API
- Health status aggregation
- Service versioning
- Public/private service management

## 3. Data Models

### 3.1 Enhanced Integration Models

```rust
// In integrations/models.rs

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum IntegrationType {
    Local,   // Current: Rust crate process
    Remote,  // New: Remote DXN server
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemIntegrationModel {
    pub(crate) name: String,
    pub(crate) integration_type: IntegrationType,
    
    // For local integrations (backward compatible)
    #[serde(default)]
    pub(crate) path: Option<String>,
    
    // For remote integrations
    #[serde(default)]
    pub(crate) service_name: Option<String>,  // References service in mesh
    #[serde(default)]
    pub(crate) url: Option<String>,            // Direct URL (optional)
    
    pub(crate) version: String,
    pub(crate) functions: Vec<SystemIntegrationFunction>
}

// New: Service Mesh Configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceMeshConfig {
    #[serde(default)]
    pub(crate) registry_url: Option<String>,
    #[serde(default)]
    pub(crate) local_services: Option<Vec<LocalService>>,
    #[serde(default)]
    pub(crate) public_services: Option<Vec<PublicServiceConfig>>,
    #[serde(default)]
    pub(crate) discovery_interval: Option<u64>, // seconds
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocalService {
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) service_type: String,  // "ai", "wallet", "vault", etc.
    pub(crate) capabilities: Vec<String>,
    #[serde(default)]
    pub(crate) auth: Option<ServiceAuth>,
    #[serde(default)]
    pub(crate) health_check: Option<String>, // Health check endpoint
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PublicServiceConfig {
    pub(crate) name: String,
    pub(crate) discover_from: String, // "registry"
    pub(crate) filter: ServiceFilter,
    #[serde(default)]
    pub(crate) preferred_versions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceFilter {
    #[serde(default)]
    pub(crate) service_type: Option<String>,
    #[serde(default)]
    pub(crate) public: Option<bool>,
    #[serde(default)]
    pub(crate) capabilities: Option<Vec<String>>,
    #[serde(default)]
    pub(crate) min_version: Option<String>,
    #[serde(default)]
    pub(crate) owner: Option<String>, // UUID
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceAuth {
    pub(crate) auth_type: String, // "oauth", "api_key", "none"
    #[serde(default)]
    pub(crate) client_id: Option<String>,
    #[serde(default)]
    pub(crate) token: Option<String>,
    #[serde(default)]
    pub(crate) scopes: Option<Vec<String>>,
}

// Service Registry Models
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceRegistryEntry {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) url: String,
    pub(crate) service_type: String,
    pub(crate) public: bool,
    pub(crate) owner: String,
    pub(crate) capabilities: Vec<String>,
    pub(crate) version: String,
    pub(crate) health: ServiceHealth,
    pub(crate) endpoints: ServiceEndpoints,
    #[serde(default)]
    pub(crate) auth_required: bool,
    #[serde(default)]
    pub(crate) scopes: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ServiceHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceEndpoints {
    pub(crate) api: String,
    pub(crate) discovery: String,
    pub(crate) health: Option<String>,
}
```

### 3.2 System Model Updates

```rust
// In system/models.rs

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct System {
    pub(crate) data: SystemData,
    pub(crate) server: SystemServer,
    pub(crate) integrations: SystemIntegrations,
    pub(crate) functions: SystemFunctions,
    #[serde(default)]
    pub(crate) service_mesh: Option<ServiceMeshConfig>, // New
}
```

## 4. Implementation Phases

### Phase 1: Foundation (Core Infrastructure)
**Goal:** Basic service mesh client and models

**Tasks:**
1. Add service mesh data models to `integrations/models.rs`
2. Create `integrations/service_mesh/` module structure
3. Implement basic service registry client
4. Add service mesh config to `System` model
5. Update config.json schema

**Deliverables:**
- Service mesh models defined
- Basic registry client can connect
- Config parsing works

### Phase 2: Service Discovery
**Goal:** Discover and register services

**Tasks:**
1. Implement service discovery from registry
2. Implement local service registration
3. Service caching mechanism
4. Periodic discovery refresh
5. Service filtering logic

**Deliverables:**
- Can discover public services from registry
- Can register local services
- Service cache with TTL

### Phase 3: Remote Integration Calls
**Goal:** Call remote services via HTTP

**Tasks:**
1. Enhance `integration::manager::run()` to support remote calls
2. Implement HTTP client for remote services
3. OAuth token management
4. Request/response serialization
5. Error handling and retries

**Deliverables:**
- Can call remote services via HTTP
- OAuth authentication working
- Unified API for local and remote

### Phase 4: Health Monitoring
**Goal:** Monitor service health

**Tasks:**
1. Implement health check client
2. Periodic health checks for registered services
3. Health status caching
4. Automatic failover (optional)
5. Health status reporting to registry

**Deliverables:**
- Health checks working
- Services marked as healthy/unhealthy
- Automatic retry on failure

### Phase 5: Advanced Features
**Goal:** Load balancing, versioning, capabilities

**Tasks:**
1. Load balancing across multiple services
2. Service version management
3. Capability-based routing
4. Service metrics and monitoring
5. Request tracing

**Deliverables:**
- Load balancing functional
- Version selection working
- Capability filtering

## 5. Service Discovery Flow

### 5.1 Initialization
```
1. Server starts
2. Load config.json
3. Initialize Service Mesh Client
4. Register local services (if any)
5. Discover public services from registry
6. Cache discovered services
7. Start health check monitoring
```

### 5.2 Service Discovery Process
```
1. Query registry with filters
2. Registry returns matching services
3. Validate service endpoints
4. Perform initial health check
5. Cache service metadata
6. Set up periodic refresh
```

### 5.3 Service Call Flow
```
1. Function/route calls integration by name
2. Integration Manager looks up integration
3. Check integration type:
   - Local → Execute process (current behavior)
   - Remote → Route to Service Mesh Client
4. Service Mesh Client:
   - Resolve service name to URL
   - Check health status
   - Get/refresh OAuth token
   - Make HTTP request
   - Return response
```

## 6. Communication Protocols

### 6.1 Service Registry API

**Endpoints:**
- `GET /api/services/discover` - Discover services with filters
- `GET /api/services/{id}` - Get specific service details
- `POST /api/services/register` - Register a new service
- `PUT /api/services/{id}/health` - Update health status
- `GET /api/services/{id}/capabilities` - Get service capabilities

**Request Example:**
```json
GET /api/services/discover?type=ai&public=true&capabilities=llm,rag
```

**Response Example:**
```json
{
    "services": [
        {
            "id": "svc_abc123",
            "name": "public_ai_llm",
            "url": "https://ai-service.dxn.io",
            "type": "ai",
            "capabilities": ["llm", "embeddings", "rag"],
            "version": "1.5",
            "health": "healthy"
        }
    ]
}
```

### 6.2 Remote Service API

**Standard Endpoint:**
- `POST /api/integrations/{function_name}` - Execute integration function

**Request Format:**
```json
POST /api/integrations/generate_text
Headers:
  Authorization: Bearer <oauth_token>
  Content-Type: application/json
Body:
{
    "params": {
        "prompt": "Hello world",
        "max_tokens": 100
    }
}
```

**Response Format:**
```json
{
    "success": true,
    "result": "Generated text here...",
    "metadata": {
        "execution_time_ms": 150
    }
}
```

## 7. Configuration Example

### 7.1 Primary Server Config

```json
{
    "server": {
        "role": "primary",
        "public": [...]
    },
    "serviceMesh": {
        "registryUrl": "https://registry.dxn.io",
        "discoveryInterval": 300,
        "localServices": [
            {
                "name": "my_ai",
                "url": "https://my-ai.example.com",
                "serviceType": "ai",
                "capabilities": ["text_generation", "sentiment_analysis"],
                "auth": {
                    "authType": "oauth",
                    "clientId": "my-server-client-id"
                },
                "healthCheck": "/api/health"
            },
            {
                "name": "my_wallet",
                "url": "https://my-wallet.example.com",
                "serviceType": "wallet",
                "capabilities": ["zksync", "ethereum"],
                "healthCheck": "/api/health"
            }
        ],
        "publicServices": [
            {
                "name": "public_zksync",
                "discoverFrom": "registry",
                "filter": {
                    "serviceType": "zksync",
                    "public": true,
                    "capabilities": ["bridge", "swap"]
                }
            },
            {
                "name": "public_ai",
                "discoverFrom": "registry",
                "filter": {
                    "serviceType": "ai",
                    "public": true,
                    "capabilities": ["llm"]
                }
            }
        ]
    },
    "integrations": {
        "public": [
            {
                "name": "ai",
                "integrationType": "remote",
                "serviceName": "my_ai",
                "version": "1",
                "functions": [
                    {"name": "generate_text"},
                    {"name": "analyze_sentiment"}
                ]
            },
            {
                "name": "wallet",
                "integrationType": "remote",
                "serviceName": "my_wallet",
                "version": "1",
                "functions": [
                    {"name": "create_transaction"},
                    {"name": "get_balance"}
                ]
            },
            {
                "name": "zksync",
                "integrationType": "remote",
                "serviceName": "public_zksync",
                "version": "1",
                "functions": [
                    {"name": "bridge_tokens"}
                ]
            },
            {
                "name": "parser",
                "integrationType": "local",
                "path": "integrations/parser",
                "version": "1",
                "functions": [
                    {"name": "parse_html"}
                ]
            }
        ]
    }
}
```

## 8. Security & OAuth Integration

### 8.1 OAuth Flow for Remote Services

```
1. Service Mesh Client needs to call remote service
2. Check if OAuth token exists and is valid
3. If not, initiate OAuth flow:
   - Redirect to service's OAuth endpoint
   - User authorizes
   - Receive access token
   - Store token securely
4. Include token in HTTP requests
5. Refresh token when expired
```

### 8.2 Token Management

- Store tokens in encrypted vault
- Automatic token refresh
- Scope-based access control
- Token revocation support

## 9. Backward Compatibility

### 9.1 Migration Path

1. Existing local integrations continue to work
2. New `integrationType` field defaults to "local" if not specified
3. Service mesh is optional (only enabled if config present)
4. Gradual migration: add remote services alongside local ones

### 9.2 Compatibility Layer

```rust
// In integration manager
pub fn run(integration_name: &str, command: &str, args: Option<&str>) -> Result<String, IntegrationError> {
    let integration = get(integration_name)?;
    
    match integration.integration_type {
        IntegrationType::Local => {
            // Current behavior - process execution
            run_local(integration, command, args)
        },
        IntegrationType::Remote => {
            // New behavior - HTTP call
            run_remote(integration, command, args)
        }
    }
}
```

## 10. API Design

### 10.1 Enhanced Integration Manager API

```rust
// Unified API for both local and remote
pub fn call_integration(
    integration_name: &str,
    function_name: &str,
    params: &serde_json::Value
) -> Result<serde_json::Value, IntegrationError>

// Service mesh specific
pub fn discover_services(filter: &ServiceFilter) -> Result<Vec<ServiceRegistryEntry>>
pub fn register_local_service(service: &LocalService) -> Result<()>
pub fn get_service_health(service_name: &str) -> Result<ServiceHealth>
```

## 11. Testing Strategy

### 11.1 Unit Tests
- Service discovery logic
- Service filtering
- OAuth token management
- Request routing

### 11.2 Integration Tests
- End-to-end service call flow
- Registry communication
- Health check monitoring
- Error handling and retries

### 11.3 Test Infrastructure
- Mock service registry
- Mock remote services
- Test OAuth server

## 12. Future Enhancements

### 12.1 Advanced Features
- Service mesh metrics and observability
- Distributed tracing
- Circuit breakers for resilience
- Rate limiting per service
- Service mesh federation (multiple registries)

### 12.2 Developer Experience
- CLI tool for service management
- Service templates
- Automatic service documentation
- Service testing framework

## 13. Implementation Priority

**Phase 1 (MVP):**
- Basic service mesh models
- Local service registration
- Simple remote service calls (HTTP)
- OAuth integration

**Phase 2 (Discovery):**
- Service registry integration
- Public service discovery
- Service caching

**Phase 3 (Production):**
- Health monitoring
- Load balancing
- Advanced routing
- Metrics and monitoring

## 14. Example Use Cases

### 14.1 User Adds Their Own AI Server

**Scenario:** User wants to host their own AI server separately from their primary server.

**Configuration:**
```json
{
    "serviceMesh": {
        "localServices": [
            {
                "name": "my_ai",
                "url": "https://my-ai.example.com",
                "serviceType": "ai",
                "capabilities": ["text_generation", "embeddings"]
            }
        ]
    },
    "integrations": {
        "public": [
            {
                "name": "ai",
                "integrationType": "remote",
                "serviceName": "my_ai",
                "functions": [{"name": "generate_text"}]
            }
        ]
    }
}
```

**Flow:**
1. Primary server registers `my_ai` as local service
2. Integration `ai` references `my_ai` service
3. When function calls `ai.generate_text`, request routed to `my_ai` server
4. OAuth token used for authentication
5. Response returned to calling function

### 14.2 User Links to Public ZKSync Service

**Scenario:** User wants to use a public ZKSync service hosted by another user.

**Configuration:**
```json
{
    "serviceMesh": {
        "registryUrl": "https://registry.dxn.io",
        "publicServices": [
            {
                "name": "public_zksync",
                "discoverFrom": "registry",
                "filter": {
                    "serviceType": "zksync",
                    "public": true
                }
            }
        ]
    },
    "integrations": {
        "public": [
            {
                "name": "zksync",
                "integrationType": "remote",
                "serviceName": "public_zksync",
                "functions": [{"name": "bridge_tokens"}]
            }
        ]
    }
}
```

**Flow:**
1. Primary server queries registry for ZKSync services
2. Registry returns matching public services
3. Service cached and health-checked
4. Integration `zksync` references discovered service
5. OAuth flow initiated for access
6. Function calls routed to public service

### 14.3 Mixed Local and Remote Services

**Scenario:** User has local parser, remote AI, and public wallet service.

**Configuration:**
```json
{
    "integrations": {
        "public": [
            {
                "name": "parser",
                "integrationType": "local",
                "path": "integrations/parser"
            },
            {
                "name": "ai",
                "integrationType": "remote",
                "serviceName": "my_ai"
            },
            {
                "name": "wallet",
                "integrationType": "remote",
                "serviceName": "public_wallet"
            }
        ]
    }
}
```

**Flow:**
- `parser` → Executed as local process (current behavior)
- `ai` → HTTP call to user's AI server
- `wallet` → HTTP call to discovered public wallet service
- All use same unified API: `integrations::manager::call_integration()`

## 15. Benefits Summary

### 15.1 For Users
- **Flexibility**: Mix local and remote services
- **Scalability**: Distribute services across servers
- **Community**: Access public services from others
- **Control**: Own and manage your services

### 15.2 For Developers
- **Separation**: Clear separation of concerns
- **Reusability**: Share services with community
- **Modularity**: Each service is independent
- **Standards**: Industry-standard patterns

### 15.3 For Ecosystem
- **Network Effects**: More services = more value
- **Specialization**: Each server can specialize
- **Innovation**: Easy to add new service types
- **Interoperability**: Standard communication protocols

