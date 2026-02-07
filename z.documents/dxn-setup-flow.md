# DXN Setup Flow (`dxn-setup`)

This document describes the one-time provisioning workflow for a DXN server instance using the `dxn-setup` CLI.

---

## Overview

`dxn-setup` is a **single-use provisioning tool**. It:

1. Collects a username and shared secret.
2. Derives an encryption key for the vault using Argon2/Balloon KDF.
3. Preloads vault key/value pairs from a secrets file.
4. Generates the final `config.json` for `dxn-core` from a template + settings.
5. Writes a provisioning lock file to prevent re-initialization.

After a successful run, this instance **cannot be provisioned again** by `dxn-setup`.

---

## Preconditions

- Project directory exists: `/path/to/project`
- `dxn-core` directory exists inside project root.
- Template config available: `config.template.json`
- Settings file available: `settings.json` (optional but recommended)
- Secrets file available: `secrets.txt` (key=value pairs)

---

## Step-by-step Flow

### 1. Start

1. User runs:

   ```bash
   dxn-setup \
     --project-root /path/to/project \
     --config-template ./config.template.json \
     --settings-file ./settings.json \
     --secrets-file ./secrets.txt
   ```

2. `dxn-setup` resolves absolute paths and validates:

   - `project-root` exists.
   - `dxn-core` directory exists.
   - Template, settings, and secrets files exist and are readable.

3. If `dxn-core/.dxn-setup-lock.json` exists:

   - Display:  
     > This DXN instance has already been provisioned. To create a new server, use a new project directory.
   - Exit with code `10`.

---

### 2. Collect identity and shared secret

1. Prompt for `Username:` (or read from `DXN_SETUP_USERNAME` when `--non-interactive`).
2. Prompt for `Shared secret:` (no echo, or `DXN_SETUP_SECRET` in non-interactive mode).

No secrets are written to logs or stored in plaintext.

---

### 3. Derive vault key

1. Read KDF profile (default: `standard`).
2. Generate random salt.
3. Use Argon2id (or Argon2 + Balloon) with:

   - memory: profile-dependent
   - iterations: profile-dependent
   - parallelism: profile-dependent

4. Compute derived key.
5. Compute `key_fingerprint = HMAC-SHA256(derived_key, "DXN_VAULT")`.

Only salt, KDF parameters, and `key_fingerprint` are stored on disk.

---

### 4. Preload vault

1. Parse `secrets.txt`:

   - Each non-empty, non-comment line:  
     `path.to.key=value`
   - Example:  
     `profile.email=admin@example.com`

2. Create or open `dxn-core/vault.db`.
3. For each entry:

   - Store encrypted value under key `path.to.key`.

4. Close vault.

---

### 5. Generate final config

1. Load `config.template.json`.
2. Load `settings.json` (if provided).
3. Merge settings into template (e.g. host, ports, base URLs).
4. Optionally validate:

   - required fields present,
   - no conflicting routes or data models.

5. Write merged config to `dxn-core/config.json`.

---

### 6. Write provisioning lock

1. Compute `config_checksum = sha256(config.json)`.
2. Write `dxn-core/.dxn-setup-lock.json`:

   - `instance_id` (UUID)
   - `project_root`
   - `created_at`
   - `created_by` (username)
   - `config_checksum`
   - `vault` metadata (path, KDF params, salt, `key_fingerprint`)

3. Display success message:

   > DXN setup complete. Start the server from the dxn-core directory:
   >
   > ```bash
   > cd dxn-core
   > cargo run
   > ```

---

## Error Handling

- If any step fails (invalid files, vault error, config merge error):
  - Print a descriptive error.
  - Do **not** write `.dxn-setup-lock.json`.
  - Exit with a non-zero code.

---

## Re-running

- If `.dxn-setup-lock.json` exists, `dxn-setup` refuses to run.
- To create another server, the user must:

  1. Create a new project directory.
  2. Copy or clone the codebase.
  3. Run `dxn-setup` in that new directory.

Existing instances are never re-initialized by `dxn-setup`.

