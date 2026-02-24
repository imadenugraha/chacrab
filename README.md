# 🔐 Chacrab

Security-first CLI password manager with zero-knowledge design, client-side encryption, and offline-first operation.

## ✨ Highlights

- 🔑 Argon2id key derivation (`m=65536`, `t=3`, `p=1`)
- 🛡️ ChaCha20-Poly1305 encryption with random 96-bit nonce
- 💾 Encrypted-at-rest vault storage for SQLite, PostgreSQL, and MongoDB
- 🔒 OS keyring-backed session key handling (fail-closed behavior)
- 🧭 Secure CLI UX: hidden prompts, no-echo sensitive input, reveal/copy safeguards
- 📦 Encrypted backup export/import with integrity verification (`SHA-256` checksum)

## 🧠 Security Model

- Master password is never persisted.
- Stored auth bootstrap contains only `salt + verifier + Argon2 parameters`.
- Vault records persist ciphertext + nonce + non-sensitive metadata only.
- Session key is stored in OS keyring and removed on logout.
- Sensitive buffers are zeroized where possible.

## 🚀 Quick Start (Cargo)

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

## 🛠️ Makefile Usage

Use Make targets for faster local workflows:

```bash
# Show all tasks
make help

# Core quality gates
make check
make fmt
make clippy
make test-all

# Common CLI flows
make init
make login
make add-password
make list
make show ID=<id-or-prefix>
make logout

# Backend integration helpers
make docker-up
make test-backend
make docker-down
```

## 📚 Command Reference

- `init` - initialize vault auth metadata
- `login` / `logout` - start or end secure session
- `add-password` / `add-note` - create encrypted entries
- `list` / `show <id-or-prefix>` / `delete <id-or-prefix>` - manage entries
- `backup-export <path>` / `backup-import <path>` - encrypted backup workflows
- `sync` - perform encrypted bidirectional synchronization
- `config` - display current runtime configuration

## ⚙️ Global Options

- `--backend <sqlite|postgres|mongo>`
- `--database-url <url>`
- `--json` (machine-readable output)
- `--quiet` (minimal output)
- `--no-color`
- `--session-timeout-secs <N>`

## 🗄️ Backend Examples

```bash
# SQLite (default)
cargo run --bin chacrab -- --backend sqlite --database-url sqlite://chacrab.db init

# PostgreSQL
cargo run --bin chacrab -- --backend postgres --database-url postgres://chacrab:chacrab@localhost:5433/chacrab init

# MongoDB
cargo run --bin chacrab -- --backend mongo --database-url mongodb://localhost:27018/chacrab init
```

After a successful `init`, Chacrab persists the selected `--backend` and `--database-url` in
`~/.config/chacrab/config.json` (or `CHACRAB_CONFIG_PATH` when set). Later commands reuse this
config unless you explicitly pass new values.

## 📦 Encrypted Backup

```bash
cargo run --bin chacrab -- backup-export ./vault.backup
cargo run --bin chacrab -- backup-import ./vault.backup
```

`backup-export` writes encrypted backup data plus checksum.
`backup-import` verifies checksum before decrypting and upserting records.

## 🔄 Sync

`sync` performs encrypted bidirectional synchronization between the local vault and a remote
backend configured with environment variables:

- `CHACRAB_SYNC_BACKEND` (`sqlite`, `postgres`, or `mongo`)
- `CHACRAB_SYNC_DATABASE_URL` (connection URL for the remote backend)
- `CHACRAB_SYNC_AUTH_TOKEN` (required for non-sqlite sync targets, min 16 chars)
- `CHACRAB_SYNC_REQUIRE_TLS` (`true` by default; set `false` only for local/dev)

Sync conflict handling is deterministic and version-aware:

- higher `sync_version` always wins,
- ties resolve by timestamp, then tombstones (delete-wins),
- stale remote updates are rejected by replay protection,
- CLI shows `⚠️` conflict/replay summaries with short IDs only.

Example:

```bash
CHACRAB_SYNC_BACKEND=mongo \
CHACRAB_SYNC_DATABASE_URL=mongodb://localhost:27018/chacrab_sync?tls=true \
CHACRAB_SYNC_AUTH_TOKEN=replace-with-long-random-token \
cargo run --bin chacrab -- sync
```

## 🧪 Integration Testing (Postgres + Mongo)

```bash
# start test infrastructure
docker compose up -d

# run backend selection integration test
CHACRAB_TEST_POSTGRES_URL=postgres://chacrab:chacrab@localhost:5433/chacrab \
CHACRAB_TEST_MONGO_URL=mongodb://localhost:27018/chacrab \
cargo test --test backend_selection
```

## 🧰 Troubleshooting Tips

- 🔐 **Keyring errors (`No active session` / keyring unavailable)**: ensure your OS keyring service is running and unlocked, then run `chacrab login` again.
- 🗃️ **Backend mismatch after init**: run `chacrab config` to inspect active backend/URL, or pass explicit `--backend` and `--database-url` for one-off commands.
- 🔄 **Sync configuration errors**: ensure `CHACRAB_SYNC_BACKEND`, `CHACRAB_SYNC_DATABASE_URL`, and `CHACRAB_SYNC_AUTH_TOKEN` (non-sqlite) are set, and TLS is enabled unless explicitly disabled for local development.
- 🧪 **Backend tests skipped/failing**: verify Postgres/Mongo are up (`make docker-up`) and URLs match expected test env variables.
- 🧹 **Formatting/lint gate failures**: run `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` before pushing.

## 🏗️ Development Layout

- `src/bin/main.rs` - binary entrypoint
- `src/core/` - crypto, models, errors, vault and backup logic
- `src/storage/` - repository trait + backend implementations + runtime selector
- `src/auth/` - registration/login/logout + keyring session management
- `src/cli/` - parser, commands, prompts, display, session helpers
- `src/sync/` - sync scaffolding

## 📌 Current Scope Notes

- `sync` is enabled for encrypted bidirectional transfer between local and configured remote backend.
- Secret reveal/copy actions are blocked on insecure terminal output (redirected/non-TTY).

## 🏷️ Versioning

- Chacrab follows Semantic Versioning.
- Release notes and change history are maintained in [CHANGELOG.md](CHANGELOG.md).
- Required release gates are enforced in CI via [.github/workflows/ci.yml](.github/workflows/ci.yml).
- Manual release steps are documented in [RELEASE_CHECKLIST.md](RELEASE_CHECKLIST.md).
- Branch merge requirements for stable releases are documented in [BRANCH_PROTECTION.md](BRANCH_PROTECTION.md).

See `ARCHITECTURE.md` for deep design details and `TODO.md` for next priorities.
