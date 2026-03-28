# DXN-Setup Tasks

## Provisioning pipeline (execution order)

After validate → identity → SA file → KDF metadata, the **on-disk** steps run in this order:

1. **Phase E — Config** — Patch `<project>/dxn-core/config.json` (`settings`, `vault`).
2. **Phase D — Keystore seed staging** — Write `dxn-core/.dxn-keystore-seed/` from `--secrets-file`.
3. **Phase F — Lock file** — Write `dxn-core/.dxn-setup-lock.json` last (skipped if any earlier step fails).

**`dxn-core` (first init):** creates the securestore vault and key (`SecretsManager`, `save_as`, `export_key`, etc. per `keystore.md`), **ingests** the staged seeds into the vault, then **deletes the staging directory** (or moves it to a quarantine and deletes) once initialization succeeds. Until then, treat the staging dir as **sensitive plaintext** (tight permissions, documented in operator runbooks).

---

## Phase A: CLI and validation

- [x] CLI parsing (project-root, secrets-file, kdf-profile, non-interactive)
- [x] Path resolution and validation (project-root, dxn-core, `dxn-core/config.json`, secrets)
- [x] Lock file check; exit 10 when already provisioned
- [x] Exit codes 0, 10, 11–15 in error.rs
- [x] Main orchestration: parse → validate → exit with correct code
- [x] Integration tests for exit 11 and exit 10

## Phase B: Identity and secret

- [x] Add rpassword (and optional secrecy) to Cargo.toml
- [x] Add identity.rs: collect_identity(args) → (username, secret)
- [x] Interactive: prompt Username then Shared secret (no echo)
- [x] Non-interactive: read DXN_SETUP_USERNAME and DXN_SETUP_SECRET from env
- [x] Error variant for missing env var; map to exit code
- [x] Wire main: after validate, call collect_identity; hold (username, secret) for Phase C/F

## Phase B-SA: SA file (after identity, before vault)

- [x] Add argon2, base64, uuid (and rand, chrono) to Cargo.toml for SA file
- [x] Create sa_file.rs: write_sa_file(paths, identity, kdf_profile)
- [x] Compute Argon2id password hash with random salt; write id, username, password_hash, password_salt, created_at
- [x] Write .sa-identity.json under dxn_core_dir; set file permissions 0o600
- [x] Wire main: after collect_identity, call write_sa_file; map errors to exit 15

## Phase C: KDF and vault key

- [x] Add `src/kdf.rs` with profile-based Argon2id params (start with `standard`)
- [x] Implement `derive_vault_key(secret, profile)` returning `{ key_bytes, salt_b64, kdf_params }`
- [x] Implement `key_fingerprint = HMAC-SHA256(derived_key, "DXN_VAULT")`
- [x] Add data structs for lock metadata: `kdf` + `key_fingerprint`
- [x] Wire main: after identity/SA file, call KDF derive and hold outputs for Phase D/F

## Phase D: Keystore seed staging (no securestore in dxn-setup)

`dxn-setup` only prepares **ingest inputs** for `dxn-core`. Vault creation and `SecretsManager` usage are **out of scope** for this crate.

- [x] Add `src/keystore_seed_staging.rs`: given validated `--secrets-file`, write seed material into `dxn-core/.dxn-keystore-seed/` (`0700` on Unix), replace directory if present
- [x] Format: normalized `secrets.seed` (`key=value` per line) + `manifest.json` (`format_version`, `entry_count`, `entries_sha256` of seed bytes, `entries_file`)
- [x] Parse/normalize lines (`path.to.key=value`; skip blanks and `#` comments); validate keys (ASCII alnum + `._-`); reject duplicates and empty files
- [x] Wire `main`: run after Phase E; canonical staging path is printed on success
- [x] **Do not** add `securestore` / OpenSSL to `dxn-setup` for this phase
- [ ] Document operator + `dxn-core` contract in `docs/`: core removes staging dir **after** vault ingest + persistence succeeds; failed ingest should not leave stale seeds world-readable

## Phase D2: Keystore ingest + vault (dxn-core — implement there)

Work tracked in **dxn-core**, listed here for a single end-to-end picture:

- [ ] On first init, read staging path from config and/or `.dxn-setup-lock.json`
- [ ] `SecretsManager::new` / load path per `keystore.md`; create vault + key files at configured final locations
- [ ] Read staged seeds, `set` each entry, `save`, **then** delete the staging directory entirely (or secure wipe policy as decided)
- [ ] If staging path missing or empty after successful prior init, treat as no-op (idempotent second boot)

## Phase E: Config generation

- [x] Patch **only** `<project>/dxn-core/config.json` (no alternate path; avoids writing the repo-root manifest by mistake)
- [x] Update only **`settings`** (`keystoreSeedPath`, `admin`) and **`vault`** (deep-merge KDF + fingerprint; preserve `vault.path` when set, default `./vault.db`); do not add **`auth`** or other top-level blocks
- [x] Validate patched config; write back to the same file path; checksum via `GeneratedConfig` for Phase F
- [x] Wire `main`: **Phase E after C, before D**

## Phase F: Lock file and success

- [x] Add `src/lock.rs` — `instance_id`, `project_root`, `created_at`, `created_by`, `config_checksum` (`sha256:` + hex), `keystore_seed_path`, `vault` (`path`, `kdf`, `key_fingerprint`)
- [x] Write `dxn-core/.dxn-setup-lock.json` last; `0600` on Unix
- [x] Success message references lock + dxn-core ingest / staging removal
- [x] Integration test: full run writes `config.json`, seed dir, `.sa-identity.json`, `.dxn-setup-lock.json`; no `keystore.json` in dxn-core from setup
- [x] Integration test: rerun exits **10** when lock exists (`exit_10_when_lock_file_exists`)
