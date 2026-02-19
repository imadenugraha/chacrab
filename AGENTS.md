# ChaCrab Agent Context

This document serves as the main instruction guide for AI Agents in developing and maintaining ChaCrab, a CLI-based secret manager with Zero-Knowledge principles.

---

## Build, Test, and Lint Commands

```bash
# Build the project
cargo build

# Build release binary
cargo build --release

# Run all tests
cargo test

# Run a single test by name
cargo test test_name

# Run a single test with pattern matching
cargo test test_full_workflow

# Run tests in a specific file
cargo test --test integration_tests

# Run unit tests only (within src/)
cargo test --lib

# Run with output visible
cargo test -- --nocapture

# Check code without building
cargo check

# Run clippy linter
cargo clippy -- -D warnings

# Format code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check

# Run the binary locally
cargo run -- init
cargo run -- --help
```

---

## Core Principles (Strict)

- **Zero-Knowledge First**: Never propose solutions that send Master Password or Plaintext to the server. Encryption MUST occur on the client side (`src/crypto/`).

- **Memory Safety**: Maximize usage of Rust's ownership features. Avoid `unsafe` blocks except when absolutely necessary for OS Keyring interactions.

- **No Disk Persistence**: Do not write derivation keys to `.txt` files or logs. Use `keyring-rs` for session management.

- **Nonce Uniqueness**: Every encryption must use a new unique nonce (12-byte).

---

## Tech Stack Reference

- **Language**: Rust (Edition 2021)
- **CLI**: clap v4 (Derive API)
- **Crypto**: chacha20poly1305, argon2
- **Database**: sqlx (SQLite and PostgreSQL support)
- **Session**: keyring crate for OS-level session key management
- **Error Handling**: anyhow (CLI), thiserror (library errors)
- **Testing**: assert_cmd, predicates, tempfile, tokio-test

---

## Architecture Map

```
src/
├── main.rs           # CLI entry point, clap command definitions
├── lib.rs            # Library crate root (enables unit tests)
├── commands/         # CLI command implementations
│   ├── mod.rs        # Module exports and shared utilities
│   ├── add.rs        # Add credential
│   ├── get.rs        # Retrieve credential
│   ├── list.rs       # List all credentials
│   └── ...           # Other commands
├── crypto/           # Cryptographic operations (NEVER mix with commands)
│   ├── mod.rs        # Re-exports derive_key, encrypt_data, decrypt_data
│   ├── derive.rs     # Argon2id key derivation
│   └── encrypt.rs    # ChaCha20-Poly1305 encryption/decryption
├── models/           # Data structures
│   ├── credential.rs # Credential, DecryptedCredential structs
│   └── user_config.rs
├── storage/          # Database and session management
│   ├── mod.rs        # Re-exports Database and keyring functions
│   ├── db.rs         # SQLite/PostgreSQL connection handling
│   ├── keyring.rs    # OS keyring session storage
│   └── queries.rs    # All database queries
└── ui/               # User interface utilities
    ├── mod.rs
    └── password_validator.rs

tests/
└── integration_tests.rs  # End-to-end CLI tests

migrations/
├── sqlite/           # SQLite schema migrations
└── postgres/         # PostgreSQL schema migrations
```

---

## Code Style Guidelines

### Imports

Group imports in this order with blank lines between:

```rust
use anyhow::{Context, Result};
use dialoguer::{Input, Password};

use crate::crypto::encrypt_data;
use crate::storage::{get_session_key, Database};
use crate::models::Credential;
```

- External crates first (alphabetically within each crate)
- `use crate::` for internal modules
- Use brace grouping: `use anyhow::{Context, Result};`

### Formatting

- Run `cargo fmt` before committing
- Max line length: standard rustfmt default (100 chars)
- No trailing whitespace
- Use 4 spaces for indentation (no tabs)

### Types and Naming

- **Functions/Variables**: `snake_case` (e.g., `get_session_key`, `enc_password`)
- **Types/Structs/Enums**: `PascalCase` (e.g., `DecryptedCredential`, `DatabasePool`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `VERIFICATION_SENTINEL`)
- **Module-private items**: Use `pub(crate)` visibility
- **Key types**: Use `[u8; 32]` for encryption keys, not `Vec<u8>`

### Error Handling

```rust
// Use anyhow::Result for command functions
pub async fn add_credential(db: &Database) -> Result<()> {
    let key = get_session_key()
        .context("Not logged in. Please run 'chacrab login' first.")?;
    
    let result = insert_credential(pool, ...).await;
    
    if let Err(e) = result {
        let error_msg = format!("{:#}", e);
        if error_msg.contains("UNIQUE constraint") {
            anyhow::bail!("A credential with label '{}' already exists.", label);
        }
        return Err(e).context("Failed to save credential");
    }
    
    Ok(())
}
```

- All command functions return `anyhow::Result<()>`
- Use `.context()` to add helpful error messages
- Use `anyhow::bail!` for early returns with error
- Provide actionable messages: "Not logged in. Please run 'chacrab login' first."

### Documentation

```rust
/// Encrypts plaintext using ChaCha20-Poly1305.
///
/// Returns (ciphertext_base64, nonce_base64).
pub fn encrypt_data(key: &[u8; 32], plaintext: &str) -> Result<(String, String)>
```

- Use `///` doc comments for public functions
- Keep descriptions concise (1-2 lines)
- Document return types and important behaviors

### Module Organization

```rust
// mod.rs pattern
pub mod add;
pub mod delete;

pub use add::add_credential;
pub use delete::delete_credential;

pub(crate) const VERIFICATION_SENTINEL: &str = "CHACRAB_VALID_SESSION";
```

- Re-export public API in `mod.rs`
- Use `pub(crate)` for internal helpers

### Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = get_test_key();
        let plaintext = "test data";
        
        let (ciphertext, nonce) = encrypt_data(&key, plaintext).unwrap();
        let decrypted = decrypt_data(&key, &ciphertext, &nonce).unwrap();
        
        assert_eq!(plaintext, decrypted);
    }
}
```

- Place unit tests in `#[cfg(test)] mod tests` at file bottom
- Use `#[tokio::test]` for async tests
- Integration tests go in `tests/` directory

---

## Adding New CLI Commands

1. Define subcommand in `src/main.rs` using clap's `#[derive(Subcommand)]`
2. Create new module in `src/commands/`
3. Add `pub mod command_name;` and `pub use` in `src/commands/mod.rs`
4. Implement function returning `anyhow::Result<()>`
5. Add integration test in `tests/integration_tests.rs`

---

## Database Queries

- Use parameterized queries (never string interpolation)
- Support both SQLite and PostgreSQL with match pattern:

```rust
match pool {
    DatabasePool::Sqlite(p) => {
        sqlx::query("SELECT * FROM table WHERE field = ?")
            .bind(value)
            .fetch_optional(p)
            .await?
    }
    DatabasePool::Postgres(p) => {
        sqlx::query("SELECT * FROM table WHERE field = $1")
            .bind(value)
            .fetch_optional(p)
            .await?
    }
}
```

- SQLite uses `?` placeholders
- PostgreSQL uses `$1, $2` placeholders

---

## Common Pitfalls to Avoid

- **Hardcoding Keys**: Never put static Salt or Keys in the code
- **SQL Injection**: Always use parameterized queries from sqlx
- **Logging Secrets**: Never log passwords, keys, or decrypted data
- **Dependencies**: Don't add new crates without strong justification
- **Mixed Concerns**: Keep crypto logic in `src/crypto/`, not in commands
- **Blocking in Async**: Use `tokio::test` for async test functions
