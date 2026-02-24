# TODO

## Recently Completed (UX Redesign)

- [x] Refactor CLI into reusable UX modules: `display`, `prompts`, `session`, `table`.
- [x] Replace secret command args with hidden interactive prompts for `add-password` and `add-note`.
- [x] Add consistent header, session indicator, and structured message styles.
- [x] Add secure reveal/copy flows with timer-based clearing.
- [x] Add insecure terminal detection for sensitive actions.
- [x] Add global automation flags: `--json`, `--quiet`, `--no-color`.
- [x] Add inactivity-based session timeout handling.

## Priority 0 - Security Hardening

- [x] Enforce password strength policy for `init` (length + entropy checks).
- [x] Add integration check: SQLite rows never contain plaintext password/notes.
- [x] Add secret redaction guardrails for all error + debug paths.
- [x] Add explicit keyring diagnostics with safe actionable hints (no internals leaked).
- [x] Review and zeroize any remaining temporary secret buffers in CLI/UI paths.
- [x] Add TTY-only enforcement for sensitive commands (`show` reveal/copy hard block in non-TTY).

## Priority 1 - Core Correctness

- [x] Add unit tests for Argon2 derivation and verification behavior.
- [x] Add unit tests for ChaCha20-Poly1305 encrypt/decrypt roundtrip and nonce uniqueness assumptions.
- [x] Add tests for vault service encryption/decryption and delete behavior.
- [x] Add tests for auth registration/login/logout lifecycle.
- [x] Add validation for malformed/non-12-byte nonce records.
- [x] Add tests for session timeout behavior and auto-logout path.

## Priority 2 - Storage Backends

- [x] Implement full PostgreSQL repository (schema + CRUD + auth metadata).
- [x] Implement full MongoDB repository (collections + CRUD + auth metadata).
- [x] Add backend selection integration tests for SQLite/Postgres/Mongo.
- [x] Define migration/versioning strategy across backends.
- [x] Add encrypted backup/export format with integrity verification.

## Priority 3 - Sync Engine

- [ ] Define remote sync contract/API for encrypted blob transfer.
- [ ] Implement remote adapter with auth and transport hardening.
- [ ] Add deterministic conflict resolution policy for ties and tombstones.
- [ ] Add sync tests for create/update/delete conflicts.
- [ ] Add replay-protection/version checks for remote updates.
- [ ] Add conflict reporting in CLI (`⚠️` summary with short IDs only).

## Priority 4 - UX and Config

- [x] Persist app config (selected backend, DSN, sync endpoint) in a local config file.
- [ ] Improve `config` command to support set/get/reset operations.
- [ ] Add command to rotate master password (re-encrypt all records).
- [ ] Add optional non-interactive flags for automation-safe secret input via stdin.
- [ ] Add clipboard disable toggle in config for hardened environments.

## Priority 5 - Operational

- [x] Add CI pipeline (`cargo check`, `cargo test`, formatting, lint).
- [ ] Add release profile and binary hardening flags.
- [ ] Add reproducible local dev environment docs.
- [ ] Add threat model document with assumptions and non-goals.
- [x] Add security regression checklist for each release.
