# dxn-setup — operator handoff

This document describes first-time provisioning with **[dxn-setup](https://github.com/systems-titanware/dxn-setup)**. It is intended to align with the **dxn-setup** repository README; keep one canonical copy (either here or in that repo) and link the other.

## Repository and binary

- **Source:** [github.com/systems-titanware/dxn-setup](https://github.com/systems-titanware/dxn-setup)
- **Build:** from the clone root, `cargo build --release`. The binary is `target/release/dxn-setup` (add it to `PATH` or invoke by full path).

## What provisioning does

`dxn-setup` runs **once** per DXN project tree. It validates paths, collects identity (username + shared secret), derives KDF metadata, writes the SA identity file, patches **`dxn-core/config.json`**, stages keystore seed material under **`dxn-core/.dxn-keystore-seed/`**, and writes **`dxn-core/.dxn-setup-lock.json`** last.

After success, **`dxn-core` must not be started without** `config.json` and `.dxn-setup-lock.json` beside it (see `examples/z.documents/dxn-core-init-flow.md`).

Re-running setup when a lock file already exists exits with code **10** (already provisioned).

## First run (interactive)

Typical flow:

1. Clone or download the DXN **application** repo (this monorepo or your fork) and ensure `dxn-core/config.json` exists as a template.
2. Prepare a **secrets** file (e.g. `secrets.seed`) with `key=value` lines for keys you want staged for later ingest into the SecureStore vault (see `dxn-setup-todo.md` Phase D).
3. From the **dxn-setup** build directory, run `dxn-setup` with your project’s `--project-root`, paths to template/settings/secrets as supported by the CLI, and KDF profile if applicable. Follow prompts for username and shared secret.

Exact flags are defined in the **dxn-setup** crate (`main` / CLI module); use `dxn-setup --help` after building.

## Non-interactive (CI / automation)

Set the shared identity via environment variables (see the dxn-setup crate for exact names; the implementation uses):

| Variable | Purpose |
|----------|---------|
| `DXN_SETUP_USERNAME` | Admin username when not prompting |
| `DXN_SETUP_SECRET` | Shared secret when not prompting |

If a required variable is missing in non-interactive mode, the tool exits with a documented error code (see below).

## Environment variables (runtime — dxn-core)

| Variable | Purpose |
|----------|---------|
| `DXN_CORE_SECRET` | **Required** when starting `dxn-core`: must match the provisioning shared secret. Used to verify the KDF fingerprint in `.dxn-setup-lock.json` before the server boots. You can put it in **`dxn-core/.env`** (loaded automatically; same folder as `config.json`), or export it in the shell / systemd `EnvironmentFile` / orchestrator secrets. |
| `DXN_ALLOW_CONFIG_CHECKSUM_DRIFT` | Set to `1` to allow `config.json` to differ from the checksum stored in `.dxn-core-state.json` (warns and updates the stored checksum). Default behavior is **fail** on drift so accidental config edits are caught. |

If `DXN_CORE_SECRET` is missing or wrong, startup fails with a clear error (fingerprint mismatch).

After a successful boot, **`dxn-core/.dxn-core-state.json`** records the last `config_checksum` and `instance_id`. Delete it only when you intentionally re-provision or accept resetting init metadata.

## Exit codes

| Code | Meaning |
|------|--------|
| `0` | Success — lock, config patch, seeds, and SA file written as designed |
| `10` | Already provisioned — `.dxn-setup-lock.json` already exists |
| `11`–`15` | Validation or provisioning failures (paths, config, identity, SA file, KDF, etc.) |

The authoritative mapping of error variants to codes lives in **dxn-setup** `src/error.rs` (names may vary by version). Prefer checking the version you built.

## Artifacts to expect

| Path | Role |
|------|------|
| `dxn-core/config.json` | Patched `settings`, `keystore`, `vault` (KDF + fingerprint only) |
| `dxn-core/.dxn-setup-lock.json` | Instance id, checksum, `keystore_seed_path`, `keystore` block (paths + KDF + fingerprint) |
| `dxn-core/.dxn-keystore-seed/` | Staged plaintext seeds for **dxn-core** first-run ingest (remove after successful ingest) |
| `dxn-core/.sa-identity.json` | SA identity for JWT login (restricted permissions on Unix) |
| `dxn-core/.dxn-core-state.json` | Written by **dxn-core** after init: checksum binding, instance id, keystore ingest flag (do not edit by hand) |

## See also

- `dxn-setup-todo.md` (this monorepo) — internal task checklist for the setup pipeline
- `examples/z.documents/dxn-core-init-flow.md` — dxn-core boot and vault story
- `examples/z.documents/keystore.md` — SecureStore layout (`keystore.json` + `keystore.key`)
