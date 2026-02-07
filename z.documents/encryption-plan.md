# Vault Security & Encryption Architecture

## Overview

Vault servers are **personal data servers** - each server represents one person and their data. The security model enforces:
- Single SA (Super Admin) account per server
- Single device bound at a time
- End-to-end encryption for all client/server communication
- Offline-capable with banking-grade limits

---

## Authentication Model

### Multi-Factor Authentication (SCA)

| Factor | Type | Storage |
|--------|------|---------|
| **Password** | Knowledge | Server (Argon2id hash) |
| **Device Key** | Possession | Client secure storage (Ed25519) |
| **Biometric** | Inherence | Client-only unlock (never transmitted) |
| **Recovery File** | Possession (backup) | User-managed (separate passphrase) |

### Session & Timeout Configuration

| Parameter | Value |
|-----------|-------|
| Session duration | 24 hours |
| Offline read-only mode | Up to 30 days |
| Offline limited-write mode | Up to 7 days (max 100 queued events) |
| Hard offline limit | 90 days (then full re-auth required) |
| Recovery passphrase | Separate from account password |
| Device transfer approval | Requires password re-entry on old device |

---

## Core Authentication Flows

### Flow 1: First-Time Server Setup

```
Client connects → Server returns "uninitialized"
Client generates device key pair (Ed25519)
Client collects credentials + optional PEM file
Client → Server: POST /auth/initialize
Server stores SA + generates server keys + derives shared secret
Server → Client: challenge
Client signs challenge → Server verifies → Session established
Client stores device keys in secure storage (Keychain/Keystore)
```

### Flow 2: Returning User Login

```
Client loads device key from secure storage
Client → Server: POST /auth/challenge
Server generates ephemeral X25519 key pair → encrypted challenge
Client decrypts, prompts for password (biometric can unlock)
Client signs challenge + password proof → Server verifies
Double Ratchet session established
All subsequent communication encrypted
```

### Flow 3: Server Transfer (Migration)

```
Client requests export from Old Server (authenticated)
Old Server generates encrypted export package + transfer token
Client initiates import on New Server with token
Old Server signs confirmation after challenge
New Server verifies signature chain → imports data → rebinds device
Old Server self-destructs (wipes data)
```

### Flow 4: Device Transfer (New Phone)

```
Normal (have old device):
  New device requests transfer → Old device approves (password required)
  Server revokes old → binds new → re-derives session keys

Emergency (lost device):
  New device requests recovery
  User enters recovery code OR loads recovery file
  Server verifies → revokes ALL devices → binds new
  New recovery codes generated (one-time use)
```

### Flow 5: Key Rotation

```
Client requests rotation (authenticated)
Server generates new key pair → rotation challenge
Client generates new device keys → signs with OLD + NEW keys
Server verifies dual signature → updates bindings → re-encrypts data
```

---

## Data Models

```rust
struct ServerIdentity {
    id: String,                         // UUID v7
    server_public_key: String,          // Ed25519
    server_private_key_encrypted: String,
    initialized_at: String,
    last_key_rotation: String,
}

struct SAAccount {
    id: String,
    username: String,
    password_hash: String,              // Argon2id
    password_salt: String,
    recovery_code_hash: String,         // Hashed codes
    recovery_codes_remaining: u8,       // Start with 10
    recovery_file_hash: Option<String>,
    created_at: String,
    last_login: String,
}

struct DeviceBinding {
    id: String,
    sa_account_id: String,
    device_public_key: String,          // Ed25519
    device_name: String,
    device_fingerprint: String,
    status: DeviceStatus,               // BOUND | PENDING | REVOKED
    bound_at: String,
    revoked_at: Option<String>,
    revoked_reason: Option<String>,
}

struct Session {
    id: String,
    device_binding_id: String,
    ratchet_state: String,              // Encrypted Double Ratchet
    created_at: String,
    expires_at: String,                 // 24 hours from creation
}
```

---

## Offline Tiers

| Tier | Duration | Capabilities |
|------|----------|--------------|
| **Full Access** | Online | All operations |
| **Limited Write** | 0-7 days offline | Create/edit, queue up to 100 events |
| **Read Only** | 7-30 days offline | View cached data only |
| **Extended Read Only** | 30-90 days offline | View cached, "sync required" warning |
| **Locked** | >90 days offline | Must go online, full re-auth required |

### Operations Always Requiring Online

- Schema changes
- File uploads > 1MB
- Password/key changes
- Device management
- Recovery operations

---

## Client-Side Storage

```
Secure Storage (Keychain/Keystore):
├── device_private_key
├── device_public_key
├── server_public_key
├── session_token + expiry
├── ratchet_state
├── password_hash_cached (for offline verify, 7-day window)
└── offline_state (last_online, sync_cursor, mode)

Encrypted SQLite Cache:
├── cached_schemas
├── cached_data (per schema)
├── cached_files_metadata
├── events_outbound_queue
└── events_inbound_cache
```

---

## Portal (Federation Service)

### Portal Responsibilities (ONLY)

- User account creation
- Cloud provider integration
- Server deployment
- DNS/networking setup
- Billing/subscription

### Portal NEVER Has Access To

- Server encryption keys
- User passwords
- Server data
- Recovery capability

### Handoff Flow

```
User creates account → Portal deploys server → Portal provides one-time setup token
User connects to server with token → Server verifies (token expires in 24h)
User completes Auth Flow 1 → Portal loses all access
```

---

## Encryption Stack

| Purpose | Algorithm/Library |
|---------|-------------------|
| Password hashing | Argon2id |
| Signing/verification | Ed25519 (`ed25519-dalek`) |
| Key exchange | X25519 (`x25519-dalek`) |
| Key derivation | HKDF |
| Symmetric encryption | AES-256-GCM (`aes-gcm`) |
| Session encryption | Double Ratchet Algorithm |

---

## API Endpoints

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/auth/initialize` | POST | First-time SA setup |
| `/auth/finalize` | POST | Complete first-time setup |
| `/auth/challenge` | POST | Request login challenge |
| `/auth/verify` | POST | Complete login |
| `/auth/refresh` | POST | Extend session (within 24h) |
| `/auth/rotate-keys` | POST | Key rotation |
| `/auth/device-transfer-request` | POST | Request device transfer |
| `/auth/device-transfer-approve` | POST | Approve from old device |
| `/auth/emergency-recovery` | POST | Lost device recovery |
| `/admin/export-request` | POST | Server migration export |
| `/admin/import` | POST | Server migration import |
| `/admin/transfer-confirm` | POST | Confirm server transfer |

---

## Implementation Phases

| Phase | Components | Status |
|-------|------------|--------|
| **1** | SA account model, password auth, device binding tables | Not Started |
| **2** | `/auth/initialize`, `/auth/challenge`, `/auth/verify` | Not Started |
| **3** | X25519 key exchange, session management | Not Started |
| **4** | Double Ratchet encryption, E2E channel | Not Started |
| **5** | Device transfer flow, recovery codes | Not Started |
| **6** | Server transfer/migration | Not Started |
| **7** | Offline state machine, event queue sync | **In Progress** |
| **8** | Portal integration, cloud deployment | Not Started |
| **9** | Security hardening, audit logging, rate limiting | Not Started |
| **10+** | Server-to-server federation (see `federated-vault-plan.md`) | Not Started |

### In Progress Summary

- **Phase 7**: Event sourcing implemented (`repository_events.rs`, `/api/events` endpoints); offline state machine not started

---

## Key Security Properties

1. **Zero-knowledge portal** - Portal/federation service cannot access user data
2. **Forward secrecy** - Double Ratchet ensures past sessions can't be decrypted
3. **Device binding** - Keys tied to specific device, not portable
4. **Single-device enforcement** - Prevents cloning/multi-device attacks
5. **Offline resilience** - Banking-grade limits balance security/usability
6. **Recovery without backdoors** - User-controlled recovery codes/file

---

## Future: Server-to-Server Federation

> **Note**: Federation is Phase 10+ and should only be implemented after core security (Phases 1-8) is battle-tested.

### What Federation Is NOT

- Portal brokering data between servers (Portal stays zero-knowledge)
- OAuth2 app authorization (different trust model)
- Automatic data copying/syncing

### What Federation IS

- **Direct server-to-server** communication (Portal never involved)
- **User-consented** data sharing via field-level scopes
- **Pull-based** queries (data stays at source, not copied)
- **Mutually authenticated** via server identity keys

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Portal involvement | **None** | Zero-knowledge preserved |
| Data transfer model | **Live queries** | No data duplication |
| Scope granularity | **Field-level** | "profile.name" not "profile.*" |
| Protocol basis | **GNAP / Solid patterns** | OAuth2 doesn't fit server-to-server |

See `federated-vault-plan.md` for detailed protocol specification.

---

## Recovery Strategy

### Dual Recovery Approach

| Method | When Generated | Storage | Use Case |
|--------|---------------|---------|----------|
| **Recovery Codes** | At server finalization | User writes down (offline) | Lost device, forgot password |
| **Recovery File** | At server finalization | User stores securely (USB, cloud) | Device + codes both lost |

### Recovery File Structure

```
recovery_package.dxn (encrypted)
├── recovery_key (encrypted with user-chosen passphrase)
├── server_identity_proof (signed by server)
├── sa_account_id
└── timestamp + expiry
```

The recovery file is a "cold backup" of the device key, encrypted with a separate passphrase chosen during setup.

---

## Device States

```
BOUND    → Active device, can authenticate
PENDING  → New device awaiting approval from old device
REVOKED  → Old device after transfer, cannot authenticate

Only ONE device can be BOUND at any time
```
