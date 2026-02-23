# Chacrab Architecture

## Design Goals

- Security-first implementation
- Zero-knowledge credential handling
- Client-side encryption only
- Offline-first local operation with optional encrypted sync
- Modular clean architecture with backend abstraction

## Layered Modules

## 1) CLI Layer (`src/cli`)

- `parser.rs`: `clap` command + argument definitions
- `commands.rs`: command handlers and orchestration
- `display.rs`: styled message primitives and header/session indicators
- `prompts.rs`: secure interactive prompt abstractions
- `session.rs`: session state + inactivity timeout metadata
- `table.rs`: formatted table output for list view

Responsibilities:
- Parse user input
- Call auth/vault/sync services
- Render user-facing output without leaking secrets
- Provide secure interaction modes (`--json`, `--quiet`, `--no-color`)
- Enforce session timeout for sensitive commands

## 2) Auth Layer (`src/auth`)

- `login.rs`: registration, login, logout flow
- `keyring.rs`: session key persistence in OS keyring

Responsibilities:
- Registration: create salt + verifier from master password
- Login: derive key again and verify credentials
- Session: store/retrieve/clear keyring material

## 3) Core Domain Layer (`src/core`)

- `crypto.rs`: Argon2id, AEAD encrypt/decrypt, nonce/salt generation
- `models.rs`: vault/auth domain models and encrypted payload schema
- `vault.rs`: service that performs encrypt/decrypt + repository interactions
- `backup.rs`: encrypted backup export/import envelope + integrity verification
- `errors.rs`: centralized error types with safe user-facing mapping

Responsibilities:
- Cryptographic correctness
- Domain modeling
- Sensitive buffer hygiene (zeroize)

## 4) Storage Layer (`src/storage`)

- `trait.rs`: async repository abstraction
- `sqlite.rs`: concrete encrypted-at-rest persistence
- `postgres.rs`: PostgreSQL implementation
- `mongo.rs`: MongoDB implementation
- `app.rs`: runtime backend selector + delegation wrapper

Responsibilities:
- Persist auth metadata (`salt`, `verifier`, Argon2 params)
- Persist vault items (`encrypted_data`, `nonce`, metadata)
- Never persist plaintext secret payloads
- Maintain schema metadata/version marker per backend

## 5) Sync Layer (`src/sync`)

- `sync_engine.rs`: bidirectional sync structure and conflict policy

Responsibilities:
- Compare local/remote encrypted blobs
- Resolve conflicts using `updated_at`
- Ensure sync path handles encrypted blobs only

## Data Model

`VaultItem`:
- `id: UUID`
- `type: Password | Note`
- `title: String` (plaintext metadata)
- `username: Option<String>`
- `url: Option<String>`
- `encrypted_data: Vec<u8>`
- `nonce: [u8; 12]`
- `created_at`
- `updated_at`

Encrypted payload JSON (before encryption):

```json
{
  "password": "...",
  "notes": "...",
  "custom_fields": {}
}
```

## Crypto Decisions

- KDF: Argon2id (`m=65536`, `t=3`, `p=1`)
- Key size: 256-bit (32 bytes)
- AEAD: ChaCha20-Poly1305
- Nonce: random 96-bit per encryption
- Password verifier: Argon2 encoded hash string (with params/salt)

## Security Invariants

1. No plaintext secret persistence.
2. No derived key persistence in database.
3. Session key stored in OS keyring only.
4. Fail closed on unavailable keyring.
5. Zeroize transient key/plaintext buffers.
6. Centralized non-leaky error handling.

## Current Limitations

- `sync` is a structural scaffold, not yet tied to remote transport/API.
- Clipboard clear/reveal timers are best-effort and depend on terminal/OS behavior.

## Backup Format

- Backup payload contains encrypted item records only (no plaintext secrets).
- Envelope fields include nonce, ciphertext, exported timestamp, and SHA-256 checksum.
- Import validates checksum before decryption and then upserts records.

## Runtime Flow

### Registration

1. User enters master password.
2. Generate random salt.
3. Derive 32-byte key with Argon2id.
4. Produce verifier from derived key.
5. Store only salt + verifier + KDF params.

### Login

1. Read auth metadata from storage.
2. Re-derive key from entered password + stored salt.
3. Verify against stored verifier.
4. Store derived key in OS keyring.

### Add/Show Secret

- Add: serialize payload -> encrypt -> store ciphertext/nonce/metadata.
- Show: retrieve session key from keyring -> decrypt -> deserialize payload.
