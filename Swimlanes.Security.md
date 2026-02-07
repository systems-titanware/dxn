title: DXN Security Authentication Flows

# Priority Summary

| Flow | Priority | Status | Reason |
|------|----------|--------|--------|
| Auth 1. First-Time Server Setup | HIGH | Documented | Core initialization flow |
| Auth 2. Returning User Login | HIGH | Documented | Primary auth flow |
| Auth 3. Server Transfer | HIGH | Documented | Migration capability |
| Auth 4. Key Rotation | MEDIUM | Documented | Security maintenance |
| Auth 5. Already Initialized Rejection | HIGH | Documented | Security boundary |
| Auth 6. Device Transfer (Normal) | HIGH | Documented | Core user journey |
| Auth 7. Emergency Recovery | HIGH | Documented | Lost device scenario |
| Auth 8. Failed Auth / Lockout | MEDIUM | Documented | Security posture |
| Auth 9. Session Expiry / Refresh | MEDIUM | Documented | UX consideration |
| Auth 10. Anti-Cloning Detection | MEDIUM | Documented | Attack prevention |
| Auth 11. Portal Handoff | LOW | Documented | Cloud deployment only |

---

_: **Auth 1. First-Time Login / Server Finalization [HIGH]**

Client -> Server: Connect to server URL
Server -> Client: Return server status (uninitialized)

Client -> Client: Generate device key pair (Ed25519)
Client -> Client: Collect SA credentials (username, password)
Client -> Client: Optional: Load PEM/key file

Client -> Server: POST /auth/initialize
Note:
Body: {
  username,
  password_hash (Argon2id, client-side),
  device_public_key,
  optional: pem_public_key
}

Server -> Server: Validate no existing SA
Server -> Server: Store SA credentials + device binding
Server -> Server: Generate server key pair
Server -> Server: Derive shared secret (X25519)

Server -> Client: Return { server_public_key, challenge_nonce }

Client -> Client: Sign challenge with device key
Client -> Server: POST /auth/finalize { signed_challenge }

Server -> Server: Verify signature
Server -> Server: Mark server as initialized
Server -> Server: Create encrypted session
Server -> Server: Generate recovery codes (10)
Server -> Server: Generate recovery file

Server --> Client: { session_token, encrypted_channel_params, recovery_codes, recovery_file }
Client -> Client: Store device keys securely (Keychain/Keystore)
Client -> Client: Prompt user to save recovery codes + file

_: **Auth 2. Returning User Login [HIGH]**

Client -> Client: Load device key from secure storage
Client -> Server: POST /auth/challenge { device_public_key }

Server -> Server: Lookup device binding
Server -> Server: Generate ephemeral key pair (X25519)
Server -> Client: { ephemeral_public_key, encrypted_challenge }

Client -> Client: Derive session key (X25519 + HKDF)
Client -> Client: Decrypt challenge
Client -> Client: Prompt for password (+ optional biometric unlock)
Client -> Client: Sign challenge with device key + password proof

Client -> Server: POST /auth/verify { signed_response, password_proof }

Server -> Server: Verify device signature
Server -> Server: Verify password proof
Server -> Server: Establish Double Ratchet session

Server --> Client: { session_established, ratchet_params }

Note: All subsequent messages use Double Ratchet encryption
Client <-> Server: Encrypted channel active

_: **Auth 3. Server Transfer [HIGH]**

Client -> Old Server: POST /admin/export-request
Old Server -> Old Server: Verify SA authentication
Old Server -> Old Server: Generate export package

Note:
Export includes:
- Encrypted database dump
- Config (encrypted)
- Files manifest
- Transfer token (signed)

Old Server -> Client: { export_package, transfer_token }

Client -> New Server: POST /admin/import { transfer_token }
New Server -> New Server: Verify transfer token signature
New Server -> Client: { import_challenge }

Client -> Old Server: POST /admin/transfer-confirm { challenge }
Old Server -> Old Server: Sign confirmation
Old Server -> Client: { signed_confirmation }

Client -> New Server: POST /admin/import-execute { signed_confirmation, export_package }

New Server -> New Server: Verify chain of signatures
New Server -> New Server: Import data
New Server -> New Server: Rebind device keys

Old Server -> Old Server: Self-destruct (wipe data)
Old Server --> Client: [Webhook] Server.status=transferred

New Server --> Client: [Webhook] Server.status=active
Client -> Client: Update server URL binding

_: **Auth 4. Key Rotation [MEDIUM]**

Client -> Server: POST /auth/rotate-keys (authenticated)
Server -> Server: Generate new server key pair
Server -> Client: { new_server_public_key, rotation_challenge }

Client -> Client: Generate new device key pair
Client -> Client: Sign rotation challenge with OLD key
Client -> Client: Sign rotation challenge with NEW key

Client -> Server: POST /auth/rotate-keys-confirm { old_signature, new_signature, new_device_public_key }

Server -> Server: Verify both signatures
Server -> Server: Update device binding with new key
Server -> Server: Re-derive shared secrets
Server -> Server: Update ratchet state

Server --> Client: { rotation_complete, new_ratchet_params }
Client -> Client: Replace old keys with new keys in secure storage

_: **Auth 5. Already Initialized Server Rejection [HIGH]**

Note: When unauthorized user attempts to access an initialized server

Client -> Server: Connect to server URL
Server -> Client: Return server status (initialized)

Client -> Server: POST /auth/initialize { credentials }

Server -> Server: Check SA already exists
Server -> Server: Log suspicious activity (IP, timestamp, fingerprint)
Server -> Server: Increment failed attempt counter

Server -> Client: 403 Forbidden { error: "server_already_initialized", message: "This server is already configured" }

Note: Rate limiting applies
alt: [If rate limit exceeded]
Server -> Client: 429 Too Many Requests { retry_after: 900 }
end

_: **Auth 6. Device Transfer - Normal Path [HIGH]**

Note: User has access to old device and wants to switch to new device

New Device -> Server: POST /auth/device-transfer-request { new_device_public_key }

Server -> Server: Validate request (no pending transfers)
Server -> Server: Create pending transfer record
Server -> Old Device: Push notification "New device requesting access"

Old Device -> User: Display transfer request details
User -> Old Device: Confirm transfer + enter password

Old Device -> Server: POST /auth/device-transfer-approve { transfer_id, password_proof }

Server -> Server: Verify password proof
Server -> Server: Revoke old device binding (status: REVOKED)
Server -> Server: Bind new device (status: BOUND)
Server -> Server: Clear pending transfer

Server --> Old Device: { transfer_complete: true, device_revoked: true }
Old Device -> Old Device: Clear local keys and session

Server --> New Device: { transfer_approved: true }
New Device -> Server: POST /auth/challenge { new_device_public_key }

Note: Continue with normal Auth 2 login flow
Server --> New Device: Session established

_: **Auth 7. Emergency Recovery - Lost Device [HIGH]**

Note: User lost device and needs to recover access

New Device -> Server: POST /auth/emergency-recovery-init

Server -> Client: { recovery_challenge, recovery_methods: ["codes", "file"] }

alt: [Using Recovery Codes]
Client -> User: Prompt for recovery code
User -> Client: Enter recovery code
Client -> Server: POST /auth/emergency-recovery-verify { method: "code", recovery_code }
end

alt: [Using Recovery File]
Client -> User: Prompt for recovery file + passphrase
User -> Client: Load file + enter passphrase
Client -> Client: Decrypt recovery file with passphrase
Client -> Server: POST /auth/emergency-recovery-verify { method: "file", recovery_proof }
end

Server -> Server: Verify recovery proof
Server -> Server: Revoke ALL existing device bindings
Server -> Server: Log emergency recovery event

Server -> Client: { recovery_verified: true, device_binding_challenge }

Client -> Client: Generate new device key pair
Client -> Server: POST /auth/emergency-recovery-bind { new_device_public_key, signed_challenge }

Server -> Server: Bind new device
Server -> Server: Invalidate used recovery code (if applicable)
Server -> Server: Generate new recovery codes

Server --> Client: { session_token, new_recovery_codes }
Client -> Client: Store keys securely
Client -> User: Display new recovery codes (MUST save)

Note: Old recovery codes are now invalid

_: **Auth 8. Failed Authentication / Lockout [MEDIUM]**

Client -> Server: POST /auth/verify { wrong_credentials }

Server -> Server: Verify credentials - FAIL
Server -> Server: Increment failure counter for device/IP
Server -> Server: Log failed attempt

alt: [Attempts remaining]
Server -> Client: 401 Unauthorized { error: "invalid_credentials", attempts_remaining: N }
end

alt: [Soft lockout (5 failures)]
Server -> Server: Set lockout for 15 minutes
Server -> Client: 429 Too Many Requests { error: "too_many_attempts", lockout_until: timestamp, lockout_minutes: 15 }
end

alt: [Hard lockout (10 failures)]
Server -> Server: Set lockout for 1 hour
Server -> Server: Send alert (if configured)
Server -> Client: 429 Too Many Requests { error: "account_locked", lockout_until: timestamp, lockout_minutes: 60 }
end

alt: [Severe lockout (20 failures)]
Server -> Server: Require recovery to unlock
Server -> Client: 423 Locked { error: "recovery_required", message: "Too many failed attempts. Use recovery to unlock." }
end

_: **Auth 9. Session Expiry / Refresh [MEDIUM]**

Note: Sessions expire after 24 hours

alt: [Proactive refresh (before expiry)]
Client -> Client: Check session expiry approaching
Client -> Server: POST /auth/refresh { session_token }

Server -> Server: Validate session still valid
Server -> Server: Extend session by 24 hours
Server -> Server: Update ratchet state

Server --> Client: { session_extended: true, new_expiry: timestamp }
end

alt: [Session expired]
Client -> Server: Any authenticated request
Server -> Server: Check session - EXPIRED

Server -> Client: 401 Unauthorized { error: "session_expired", message: "Please re-authenticate" }

Client -> Client: Clear session state
Client -> Client: Initiate Auth 2 (Returning User Login)
end

alt: [Refresh window expired (>24h since last activity)]
Client -> Server: POST /auth/refresh { session_token }

Server -> Server: Session too old to refresh
Server -> Client: 401 Unauthorized { error: "session_expired", refresh_allowed: false }

Client -> Client: Must complete full re-authentication
end

_: **Auth 10. Anti-Cloning Detection [MEDIUM]**

Note: Detects if device keys have been copied/cloned

Client A -> Server: POST /auth/challenge { device_public_key }
Server -> Server: Generate challenge for Client A

Note: Meanwhile, cloned device attempts auth
Client B -> Server: POST /auth/challenge { same_device_public_key }

Server -> Server: Detect concurrent challenge for same device key
Server -> Server: Flag potential clone attack
Server -> Server: Revoke ALL sessions for this device key
Server -> Server: Log security incident

Server -> Client A: 401 { error: "device_conflict", message: "Security alert: concurrent access detected" }
Server -> Client B: 401 { error: "device_conflict", message: "Security alert: concurrent access detected" }

Server -> Server: Set device status to SUSPENDED

Note: Recovery required to restore access
Client -> User: Display "Your device keys may have been compromised. Use recovery to restore access."

Client -> Server: POST /auth/emergency-recovery-init
Note: Continue with Auth 7 Emergency Recovery

_: **Auth 11. Portal Handoff - Cloud Deployment [LOW]**

Note: Only applicable when using DXN Portal for cloud deployment

User -> Portal: Create account + choose plan
Portal -> Portal: Provision cloud resources
Portal -> Cloud Provider: Deploy server instance
Cloud Provider -> Portal: { server_ip, server_url }

Portal -> Server: POST /internal/portal-setup { one_time_token, portal_signature }
Server -> Server: Store portal token (expires 24h)
Server -> Server: Verify portal signature

Portal -> User: { server_url, setup_token }

User -> Client: Enter server URL
Client -> Server: POST /auth/portal-verify { setup_token }

Server -> Server: Verify token matches + not expired
Server -> Server: Mark portal handoff complete
Server -> Server: Delete portal token (one-time use)

Server -> Client: { status: "uninitialized", portal_verified: true }

Note: Continue with Auth 1 (First-Time Setup)
Client -> Server: POST /auth/initialize { ... }

Portal -> Portal: Lose all access to server permanently

Note: Portal can NEVER access server data, keys, or credentials

---

# Implementation Tasks

## Phase 1: Core Authentication
- [ ] Implement SA account model with Argon2id password hashing
- [ ] Implement device binding table and CRUD operations
- [ ] Create `/auth/initialize` endpoint
- [ ] Create `/auth/finalize` endpoint
- [ ] Create `/auth/challenge` endpoint
- [ ] Create `/auth/verify` endpoint
- [ ] Implement Ed25519 key generation and verification
- [ ] Implement X25519 key exchange

## Phase 2: Session Management
- [ ] Implement session table with expiry
- [ ] Create `/auth/refresh` endpoint
- [ ] Implement session expiry checks middleware
- [ ] Add session cleanup job (expired sessions)

## Phase 3: Security Controls
- [ ] Implement failed attempt tracking
- [ ] Implement rate limiting (soft/hard lockout)
- [ ] Add security event logging
- [ ] Implement already-initialized rejection with logging
- [ ] Add anti-cloning detection logic

## Phase 4: Device Management
- [ ] Create `/auth/device-transfer-request` endpoint
- [ ] Create `/auth/device-transfer-approve` endpoint
- [ ] Implement push notification for transfer requests
- [ ] Implement device revocation logic

## Phase 5: Recovery
- [ ] Generate recovery codes at finalization
- [ ] Generate recovery file at finalization
- [ ] Create `/auth/emergency-recovery-init` endpoint
- [ ] Create `/auth/emergency-recovery-verify` endpoint
- [ ] Create `/auth/emergency-recovery-bind` endpoint
- [ ] Implement recovery code validation + invalidation

## Phase 6: Key Rotation
- [ ] Create `/auth/rotate-keys` endpoint
- [ ] Create `/auth/rotate-keys-confirm` endpoint
- [ ] Implement dual-signature verification
- [ ] Implement key re-derivation logic

## Phase 7: Encryption
- [ ] Implement Double Ratchet Algorithm
- [ ] Implement AES-256-GCM for message encryption
- [ ] Implement HKDF for key derivation
- [ ] Add encrypted channel middleware

## Phase 8: Server Transfer
- [ ] Create `/admin/export-request` endpoint
- [ ] Create `/admin/import` endpoint
- [ ] Create `/admin/transfer-confirm` endpoint
- [ ] Create `/admin/import-execute` endpoint
- [ ] Implement server self-destruct logic

## Phase 9: Portal Integration (Optional)
- [ ] Create `/internal/portal-setup` endpoint
- [ ] Create `/auth/portal-verify` endpoint
- [ ] Implement one-time token verification
