# DXN Server-to-Server Federation Protocol

## Overview

Federation enables DXN servers to share data directly with each other, with explicit user consent and fine-grained access control. This document specifies the protocol for secure server-to-server communication.

> **Prerequisite**: Core authentication (Phases 1-8) must be complete and stable before implementing federation.

---

## Design Principles

| Principle | Implementation |
|-----------|---------------|
| **Portal stays zero-knowledge** | Federation is direct server-to-server; Portal never touches data |
| **Pull, don't push** | Data queried on-demand, not copied/synced |
| **User consent required** | Both parties must approve the connection |
| **Field-level granularity** | Scopes specify exact fields, not whole resources |
| **Revocation is instant** | Provider revokes → next query fails |
| **Full audit trail** | Both sides log all access |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Federation Layer                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────┐              ┌──────────────────┐         │
│  │  Server A        │              │  Server B        │         │
│  │  (Requester)     │              │  (Provider)      │         │
│  │                  │              │                  │         │
│  │ ┌──────────────┐ │    mTLS +    │ ┌──────────────┐ │         │
│  │ │ Federation   │ │◄────────────►│ │ Federation   │ │         │
│  │ │ Client       │ │  Signed Req  │ │ Server       │ │         │
│  │ └──────────────┘ │              │ └──────────────┘ │         │
│  │        │         │              │        │         │         │
│  │ ┌──────────────┐ │              │ ┌──────────────┐ │         │
│  │ │ Grant Store  │ │  Scope Sync  │ │ Grant Store  │ │         │
│  │ │ (outbound)   │ │◄────────────►│ │ (inbound)    │ │         │
│  │ └──────────────┘ │              │ └──────────────┘ │         │
│  │        │         │              │        │         │         │
│  │ ┌──────────────┐ │              │ ┌──────────────┐ │         │
│  │ │ Audit Log    │ │              │ │ Audit Log    │ │         │
│  │ └──────────────┘ │              │ └──────────────┘ │         │
│  │                  │              │                  │         │
│  └──────────────────┘              └──────────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

Portal is NOT involved in any federation flow.
```

---

## Terminology

| Term | Definition |
|------|------------|
| **Requester** | Server requesting access to another server's data |
| **Provider** | Server that owns the data being requested |
| **Partner** | A server you have a federation relationship with |
| **Grant** | Permission given by provider to requester for specific scopes |
| **Scope** | Fine-grained permission (resource + fields + operations) |

---

## Data Models

```rust
/// A federated partner server
struct FederationPartner {
    id: String,                          // UUID v7
    server_url: String,                  // https://alice.dxn.io
    server_public_key: String,           // Ed25519 for verifying requests
    display_name: String,                // "Alice's Server"
    owner_name: Option<String>,          // "Alice Smith" (user-provided)
    status: PartnerStatus,
    initiated_by: InitiatedBy,           // US | THEM
    connected_at: Option<String>,
    last_activity: Option<String>,
    created_at: String,
}

enum PartnerStatus {
    PENDING_OUTBOUND,   // We sent request, awaiting their approval
    PENDING_INBOUND,    // They sent request, awaiting our approval
    CONNECTED,          // Mutual approval, can exchange data
    BLOCKED,            // Explicitly blocked by user
    EXPIRED,            // Connection expired due to inactivity
}

enum InitiatedBy {
    US,    // We initiated the connection
    THEM,  // They initiated the connection
}

/// A grant for data access
struct FederationGrant {
    id: String,                          // UUID v7
    partner_id: String,                  // FK to FederationPartner
    direction: GrantDirection,
    scopes: Vec<FederationScope>,
    granted_at: String,
    granted_by: String,                  // SA account that approved
    expires_at: Option<String>,          // Optional expiry
    revoked_at: Option<String>,
    revoked_reason: Option<String>,
}

enum GrantDirection {
    INBOUND,   // They can access OUR data
    OUTBOUND,  // We can access THEIR data
}

/// Fine-grained scope definition
struct FederationScope {
    resource: String,                    // Schema name: "profile", "posts"
    fields: Vec<String>,                 // ["name", "avatar"] or ["*"] for all
    operations: Vec<ScopeOperation>,
    filters: Option<ScopeFilter>,        // Optional: limit to subset of records
}

enum ScopeOperation {
    READ,
    WRITE,
    DELETE,
}

/// Optional filter to limit scope to specific records
struct ScopeFilter {
    field: String,                       // e.g., "visibility"
    operator: FilterOperator,            // EQ, IN, etc.
    value: serde_json::Value,            // "public" or ["public", "friends"]
}

enum FilterOperator {
    EQ,      // field == value
    NEQ,     // field != value
    IN,      // field in [values]
    GT,      // field > value (for dates, numbers)
    LT,      // field < value
}

/// Audit log for all federation activity
struct FederationAuditLog {
    id: String,                          // UUID v7
    partner_id: String,
    direction: AuditDirection,           // INBOUND_REQUEST | OUTBOUND_REQUEST
    scope_used: String,                  // Serialized scope
    operation: String,                   // "read", "write"
    resource: String,                    // Schema accessed
    record_count: u32,                   // Number of records accessed
    timestamp: String,
    request_signature: String,           // For non-repudiation
    response_status: u16,                // HTTP status returned
    client_ip: Option<String>,           // For inbound requests
}

enum AuditDirection {
    INBOUND_REQUEST,   // They requested our data
    OUTBOUND_REQUEST,  // We requested their data
}
```

---

## Federation Flows

### Flow 1: Partner Discovery & Connection Request

```
User A wants to connect to User B's server.

User A -> Client A: Enter partner server URL (https://bob.dxn.io)
Client A -> Server A: POST /federation/discover { url: "https://bob.dxn.io" }

Server A -> Server B: GET /federation/identity

Server B -> Server A: {
    server_id: "...",
    server_public_key: "...",
    display_name: "Bob's Vault",
    owner_name: "Bob",                   // Optional
    federation_enabled: true,
    supported_scopes: ["profile", "posts", "files"]
}

Server A -> Server A: Validate response signature
Server A -> Client A: { partner_info, can_request: true }

Client A -> User A: Display "Bob's Vault" - Request connection?
User A -> Client A: Approve + select initial scopes to request

Client A -> Server A: POST /federation/connect {
    partner_url: "https://bob.dxn.io",
    requested_scopes: [
        { resource: "profile", fields: ["name", "avatar"], operations: ["read"] }
    ],
    offered_scopes: [
        { resource: "profile", fields: ["name"], operations: ["read"] }
    ]
}

Server A -> Server A: Create partner record (PENDING_OUTBOUND)
Server A -> Server B: POST /federation/request {
    requester_url: "https://alice.dxn.io",
    requester_public_key: "...",
    requester_display_name: "Alice's Vault",
    requested_scopes: [...],
    offered_scopes: [...],
    signature: "..." // Signed by Server A
}

Server B -> Server B: Validate signature
Server B -> Server B: Create partner record (PENDING_INBOUND)
Server B -> Server B: Queue notification for User B

Server B -> Server A: 202 Accepted { request_id: "...", status: "pending_approval" }
Server A -> Client A: { status: "pending", message: "Awaiting Bob's approval" }
```

### Flow 2: Connection Approval

```
User B receives notification about connection request.

Client B -> Server B: GET /federation/pending

Server B -> Client B: [{
    id: "...",
    partner_url: "https://alice.dxn.io",
    partner_name: "Alice's Vault",
    requested_scopes: [...],
    offered_scopes: [...],
    requested_at: "..."
}]

Client B -> User B: Display request details
User B -> Client B: Approve + adjust scopes if desired

Client B -> Server B: POST /federation/approve {
    partner_id: "...",
    approved_inbound_scopes: [
        { resource: "profile", fields: ["name", "avatar"], operations: ["read"] }
    ],
    approved_outbound_scopes: [
        { resource: "profile", fields: ["name"], operations: ["read"] }
    ]
}

Server B -> Server B: Update partner status to CONNECTED
Server B -> Server B: Create grants (INBOUND + OUTBOUND)

Server B -> Server A: POST /federation/confirm {
    request_id: "...",
    status: "approved",
    granted_inbound_scopes: [...],    // What THEY can read from US
    granted_outbound_scopes: [...],   // What WE can read from THEM
    signature: "..."
}

Server A -> Server A: Validate signature
Server A -> Server A: Update partner status to CONNECTED
Server A -> Server A: Create grants (mirrored)

Server A -> Server B: 200 OK { confirmed: true }

Both servers now have:
- Partner record with status CONNECTED
- INBOUND grant (what partner can access)
- OUTBOUND grant (what we can access from partner)
```

### Flow 3: Data Query (Read)

```
User A wants to read User B's profile.

Client A -> Server A: GET /data/profile (with federation source)
Note: This could be a special endpoint or header indicating federated query

Server A -> Server A: Check OUTBOUND grant for Server B "profile" scope
Server A -> Server A: Grant exists with READ permission

Server A -> Server B: POST /federation/query {
    resource: "profile",
    operation: "read",
    fields: ["name", "avatar"],        // Only fields we're granted
    filters: {},                        // Optional filters
    timestamp: "...",
    nonce: "...",
    signature: "..."                    // Signed request
}

Server B -> Server B: Validate signature (Server A's public key)
Server B -> Server B: Check INBOUND grant for Server A
Server B -> Server B: Verify requested fields ⊆ granted fields
Server B -> Server B: Execute query against local data

Server B -> Server B: Log to audit trail
Server B -> Server A: {
    data: { name: "Bob", avatar: "https://..." },
    timestamp: "...",
    signature: "..."
}

Server A -> Server A: Validate response signature
Server A -> Server A: Log to audit trail
Server A -> Client A: { data: {...}, source: "federation", partner: "Bob's Vault" }
```

### Flow 4: Scope Modification

```
User B wants to expand or restrict what User A can access.

Client B -> Server B: PUT /federation/grants/{partner_id} {
    inbound_scopes: [
        { resource: "profile", fields: ["name", "avatar", "bio"], operations: ["read"] },
        { resource: "posts", fields: ["*"], operations: ["read"], 
          filters: { field: "visibility", operator: "eq", value: "public" } }
    ]
}

Server B -> Server B: Update INBOUND grants

Server B -> Server A: POST /federation/grant-update {
    updated_scopes: [...],
    effective_at: "...",
    signature: "..."
}

Server A -> Server A: Update OUTBOUND grants (what we can access from them)
Server A -> Server B: 200 OK { acknowledged: true }

Client A automatically sees expanded/restricted access on next query.
```

### Flow 5: Revocation

```
User B wants to disconnect from User A entirely.

Client B -> Server B: DELETE /federation/partners/{partner_id}

Server B -> Server B: Set partner status to BLOCKED
Server B -> Server B: Revoke all grants (set revoked_at)

Server B -> Server A: POST /federation/disconnect {
    reason: "user_initiated",           // Or "policy_violation", "inactivity"
    effective_immediately: true,
    signature: "..."
}

Server A -> Server A: Validate signature
Server A -> Server A: Set partner status to BLOCKED
Server A -> Server A: Revoke all grants

Server A -> Server B: 200 OK { acknowledged: true }

Any subsequent queries will fail with 403 FORBIDDEN.
```

### Flow 6: Query Rejection (Scope Exceeded)

```
Server A tries to access data outside granted scope.

Server A -> Server B: POST /federation/query {
    resource: "profile",
    operation: "read",
    fields: ["name", "email"],         // "email" NOT in grant!
    ...
}

Server B -> Server B: Validate signature - OK
Server B -> Server B: Check grant - "email" not permitted

Server B -> Server B: Log potential abuse to audit trail
Server B -> Server A: 403 Forbidden {
    error: "scope_exceeded",
    message: "Field 'email' not in granted scope",
    granted_fields: ["name", "avatar"],
    requested_fields: ["name", "email"]
}

Server A -> Server A: Log rejection
Server A -> Client A: { error: "scope_exceeded", ... }
```

---

## API Endpoints

### Discovery & Connection

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/federation/identity` | GET | Return server's federation identity (public) |
| `/federation/discover` | POST | Discover a potential partner server |
| `/federation/connect` | POST | Initiate connection request |
| `/federation/request` | POST | Receive incoming connection request |
| `/federation/pending` | GET | List pending connection requests |
| `/federation/approve` | POST | Approve connection request |
| `/federation/reject` | POST | Reject connection request |
| `/federation/confirm` | POST | Receive approval confirmation |

### Partner Management

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/federation/partners` | GET | List all partners |
| `/federation/partners/{id}` | GET | Get partner details |
| `/federation/partners/{id}` | DELETE | Disconnect/block partner |
| `/federation/disconnect` | POST | Receive disconnect notification |

### Grants & Scopes

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/federation/grants` | GET | List all grants (in/out) |
| `/federation/grants/{partner_id}` | GET | Get grants for partner |
| `/federation/grants/{partner_id}` | PUT | Update grants for partner |
| `/federation/grant-update` | POST | Receive grant update notification |

### Data Access

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/federation/query` | POST | Execute federated query |
| `/federation/write` | POST | Execute federated write (if permitted) |

### Audit

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/federation/audit` | GET | Query audit log |
| `/federation/audit/partner/{id}` | GET | Audit log for specific partner |

---

## Security Considerations

### Request Signing

All federation requests MUST be signed:

```
Signature = Ed25519.sign(
    private_key,
    canonical_json(request_body) + timestamp + nonce
)

Headers:
  X-Federation-Timestamp: ISO8601 timestamp
  X-Federation-Nonce: Random 32-byte hex
  X-Federation-Signature: Base64(signature)
  X-Federation-Key-Id: Server's public key ID
```

### Replay Prevention

- Requests include timestamp + nonce
- Servers reject requests with timestamp > 5 minutes old
- Servers track nonces for 10 minutes to prevent replay

### Rate Limiting

| Limit | Value | Scope |
|-------|-------|-------|
| Discovery requests | 10/minute | Per IP |
| Connection requests | 5/hour | Per server |
| Data queries | 100/minute | Per partner |
| Failed auth | 10/hour | Per partner (then block) |

### Scope Validation

Provider MUST validate every query:

1. Partner status is CONNECTED
2. Grant exists for requested resource
3. Grant includes requested operation (READ/WRITE)
4. Requested fields ⊆ granted fields
5. Query matches any scope filters
6. Grant not expired or revoked

---

## Implementation Phases

| Phase | Components | Priority |
|-------|------------|----------|
| **10.1** | Federation data models, partner table | Required |
| **10.2** | `/federation/identity`, discovery | Required |
| **10.3** | Connection request/approve flow | Required |
| **10.4** | Grant management, scope storage | Required |
| **10.5** | Request signing, signature validation | Required |
| **10.6** | `/federation/query` (read-only) | Required |
| **10.7** | Audit logging | Required |
| **10.8** | Scope modification flow | Required |
| **10.9** | Revocation/disconnect | Required |
| **10.10** | Rate limiting, abuse prevention | Required |
| **10.11** | `/federation/write` (if needed) | Optional |
| **10.12** | Scope filters (partial access) | Optional |
| **10.13** | Expiring grants | Optional |

---

## Comparison to Existing Protocols

| Protocol | Similarity | Difference |
|----------|------------|------------|
| **OAuth2** | Scope-based permissions | OAuth2 is user→app, this is server→server |
| **GNAP** | Grant negotiation | GNAP is more complex, we're simpler |
| **Solid** | Pod-to-pod data sharing | Solid uses RDF/Linked Data, we use JSON |
| **ActivityPub** | Federated servers | ActivityPub is activity-centric, we're data-centric |
| **AT Protocol** | Decentralized identity | AT has global namespace, we're pairwise |

---

## Open Questions

1. **Multi-hop queries?** - Should Server A query Server B for Server C's data? (Probably no - too complex)

2. **Caching?** - Should requester cache federated data? (Maybe with short TTL + cache-control headers)

3. **Offline access?** - Can requester access cached federated data offline? (Probably yes with explicit grant)

4. **Schema compatibility?** - What if Server A and B have different schema versions? (Return superset, let client handle)

5. **Bulk operations?** - Pagination for large result sets? (Yes, standard cursor-based pagination)

---

## Tasks

### Phase 10.1: Foundation
- [ ] Create `federation_partners` table
- [ ] Create `federation_grants` table
- [ ] Create `federation_scopes` table
- [ ] Create `federation_audit_log` table
- [ ] Add federation models to `data/models.rs`

### Phase 10.2: Discovery
- [ ] Implement `/federation/identity` endpoint
- [ ] Implement `/federation/discover` endpoint
- [ ] Add server identity to `ServerIdentity` model

### Phase 10.3: Connection Flow
- [ ] Implement `/federation/connect` endpoint
- [ ] Implement `/federation/request` endpoint
- [ ] Implement `/federation/pending` endpoint
- [ ] Implement `/federation/approve` endpoint
- [ ] Implement `/federation/reject` endpoint
- [ ] Implement `/federation/confirm` endpoint
- [ ] Add notification system for connection requests

### Phase 10.4: Grant Management
- [ ] Implement grant CRUD operations
- [ ] Implement scope validation logic
- [ ] Implement `/federation/grants` endpoints
- [ ] Implement `/federation/grant-update` endpoint

### Phase 10.5: Request Security
- [ ] Implement Ed25519 request signing
- [ ] Implement signature validation middleware
- [ ] Implement nonce tracking (replay prevention)
- [ ] Implement timestamp validation

### Phase 10.6: Data Queries
- [ ] Implement `/federation/query` endpoint
- [ ] Implement scope-filtered data retrieval
- [ ] Add federation source to data responses

### Phase 10.7: Audit
- [ ] Implement audit log writes on all operations
- [ ] Implement `/federation/audit` query endpoint
- [ ] Add audit log retention policy

### Phase 10.8-10.9: Lifecycle
- [ ] Implement scope modification flow
- [ ] Implement `/federation/disconnect` endpoint
- [ ] Implement partner blocking logic
- [ ] Implement grant revocation

### Phase 10.10: Hardening
- [ ] Implement rate limiting per partner
- [ ] Implement abuse detection
- [ ] Add alerting for suspicious activity
