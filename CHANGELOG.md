# Changelog

All notable changes to this project are documented in this file.

The format is based on Keep a Changelog,
and this project adheres to Semantic Versioning.

## [1.1.0] - 2026-02-24

### Added
- Sync tombstone model and repository API for delete propagation across backends.
- Deterministic sync conflict reporting with short-ID summaries in CLI output.
- Replay-protection counters in sync reports for stale remote update rejection.

### Changed
- Sync engine now resolves item/delete conflicts using version-first deterministic policy.
- `delete` now records sync tombstones with incremented sync version metadata.
- Remote sync adapter now validates backend URL transport constraints and auth token policy.

### Fixed
- Mongo/SQLite/Postgres sync state handling now includes `sync_version` for conflict-safe merges.

## [1.0.0] - 2026-02-24

### Added
- GitHub Actions CI workflow at `.github/workflows/ci.yml` with release gates: `cargo check`, `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
- Project release checklist in `RELEASE_CHECKLIST.md`.
- Branch protection policy for stable releases in `BRANCH_PROTECTION.md`.
- Real sync execution path using `SyncEngine` with remote backend configuration via `CHACRAB_SYNC_BACKEND` and `CHACRAB_SYNC_DATABASE_URL`.
- Sync report counters for uploaded/downloaded items.

### Changed
- Login verification now uses Argon2 parameters stored in auth metadata.
- `sync` command now performs real bidirectional synchronization instead of placeholder counters.

### Fixed
- Backup import now zeroizes decrypted plaintext buffers after deserialization attempt.

### Security
- Added release governance controls for protected branches and required CI status checks.

## [0.1.0] - 2026-02-24

### Added
- Initial CLI password manager commands: `init`, `login`, `logout`, `add-password`, `add-note`, `list`, `show`, `delete`, `backup-export`, `backup-import`, `sync`, `config`.
- Multi-backend storage support with runtime selection: SQLite, PostgreSQL, and MongoDB.
- Security-first cryptography model using Argon2id key derivation and ChaCha20-Poly1305 encryption.
- Keyring-backed session management with fail-closed behavior and timeout enforcement.
- Encrypted backup export/import with checksum integrity verification.
- CLI safeguards for sensitive actions on insecure terminal output.
- Runtime config persistence for selected `--backend` and `--database-url` after successful `init`.

### Security
- Enforced encrypted-at-rest storage model (ciphertext + nonce + metadata only).
- Zeroization of sensitive buffers where applicable.

[Unreleased]: https://github.com/<owner>/chacrab/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/<owner>/chacrab/releases/tag/v1.0.0
[0.1.0]: https://github.com/<owner>/chacrab/releases/tag/v0.1.0
