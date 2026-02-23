# Chacrab

Security-first CLI password manager with zero-knowledge design, client-side encryption, and offline-first operation.

## Highlights

- Argon2id key derivation (`m=65536`, `t=3`, `p=1`)
- ChaCha20-Poly1305 encryption with random 96-bit nonce
- Encrypted-at-rest vault storage for SQLite, PostgreSQL, and MongoDB
- OS keyring-backed session key handling (fail-closed behavior)
- Secure CLI UX: hidden prompts, no-echo sensitive input, reveal/copy safeguards
- Encrypted backup export/import with integrity verification (`SHA-256` checksum)

## Security Model

- Master password is never persisted.
- Stored auth bootstrap contains only `salt + verifier + Argon2 parameters`.
- Vault records persist ciphertext + nonce + non-sensitive metadata only.
- Session key is stored in OS keyring and removed on logout.
- Sensitive buffers are zeroized where possible.

## Quick Start

```bash
# Build
cargo check

# Initialize and login
cargo run --bin chacrab -- init
cargo run --bin chacrab -- login

# Add and list
cargo run --bin chacrab -- add-password
cargo run --bin chacrab -- add-note
cargo run --bin chacrab -- list

# Show and logout
cargo run --bin chacrab -- show <ID_OR_PREFIX>
cargo run --bin chacrab -- logout
```

## Command Reference

- `init` - initialize vault auth metadata
- `login` / `logout` - start or end secure session
- `add-password` / `add-note` - create encrypted entries
- `list` / `show <id-or-prefix>` / `delete <id-or-prefix>` - manage entries
- `backup-export <path>` / `backup-import <path>` - encrypted backup workflows
- `sync` - sync engine scaffold command
- `config` - display current runtime configuration

## Global Options

- `--backend <sqlite|postgres|mongo>`
- `--database-url <url>`
- `--json` (machine-readable output)
- `--quiet` (minimal output)
- `--no-color`
- `--session-timeout-secs <N>`

## Backend Examples

```bash
# SQLite (default)
cargo run --bin chacrab -- --backend sqlite --database-url sqlite://chacrab.db init

# PostgreSQL
cargo run --bin chacrab -- --backend postgres --database-url postgres://chacrab:chacrab@localhost:5433/chacrab init

# MongoDB
cargo run --bin chacrab -- --backend mongo --database-url mongodb://localhost:27018/chacrab init
```

## Encrypted Backup

```bash
cargo run --bin chacrab -- backup-export ./vault.backup
cargo run --bin chacrab -- backup-import ./vault.backup
```

`backup-export` writes encrypted backup data plus checksum.
`backup-import` verifies checksum before decrypting and upserting records.

## Integration Testing (Postgres + Mongo)

```bash
# start test infrastructure
docker compose up -d

# run backend selection integration test
CHACRAB_TEST_POSTGRES_URL=postgres://chacrab:chacrab@localhost:5433/chacrab \
CHACRAB_TEST_MONGO_URL=mongodb://localhost:27018/chacrab \
cargo test --test backend_selection
```

## Development Layout

- `src/bin/main.rs` - binary entrypoint
- `src/core/` - crypto, models, errors, vault and backup logic
- `src/storage/` - repository trait + backend implementations + runtime selector
- `src/auth/` - registration/login/logout + keyring session management
- `src/cli/` - parser, commands, prompts, display, session helpers
- `src/sync/` - sync scaffolding

## Current Scope Notes

- `sync` is currently structural and not yet wired to a production remote transport.
- Secret reveal/copy actions are blocked on insecure terminal output (redirected/non-TTY).

See `ARCHITECTURE.md` for deep design details and `TODO.md` for next priorities.
