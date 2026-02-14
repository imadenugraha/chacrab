# 🦀 ChaCrab Agent Context

This document serves as the main instruction guide for AI Agents in developing and maintaining ChaCrab, a CLI-based secret manager with Zero-Knowledge principles.

---

## 🎯 Core Principles (Strict)

- **Zero-Knowledge First**: Never propose solutions that send Master Password or Plaintext to the server. Encryption MUST occur on the client side (`src/crypto/`).

- **Memory Safety**: Maximize usage of Rust's ownership features. Avoid `unsafe` blocks except when absolutely necessary for OS Keyring interactions.

- **No Disk Persistence**: Do not write derivation keys to `.txt` files or logs. Use `keyring-rs` for session management.

- **Nonce Uniqueness**: Every encryption must use a new unique nonce (12-byte).

---

## 🛠 Tech Stack Reference

- **Language**: Rust (Latest Stable)
- **CLI**: clap v4 (Derive API)
- **Crypto**: chacha20poly1305, argon2
- **Database**: sqlx (PostgreSQL/Supabase)
- **Storage**: keyring crate for OS-level session key management

---

## 📂 Architecture Map

When asked to add features, refer to the following structure:

- **Crypto Logic**: Add to `src/crypto/`. Do not mix encryption logic inside `commands/`.

- **Database Queries**: Use `sqlx::query!` macro inside `src/storage/supabase.rs`.

- **UI/UX**: Use `dialoguer` for interactive input in `src/ui/`.

- **Schema**: Update `migrations/` if there are changes to the `credentials` or `secret_notes` table structure.

---

## 📋 Task Guidelines

### 1. Adding New CLI Commands

- Define sub-commands in `src/main.rs` using clap.
- Create a new module in `src/commands/`.
- Ensure functions return `anyhow::Result<()>` for clean error handling.

### 2. Error Handling

- Use the `anyhow` crate for CLI applications.
- Provide informative error messages to terminal users (e.g., "Supabase connection failed, check .env").

### 3. Data Security

- When displaying passwords, use "copy to clipboard" feature or hide input using `dialoguer::Password`.

---

## ⚠️ Common Pitfalls to Avoid

- **Hardcoding Keys**: Never put static Salt or Keys in the code.

- **SQL Injection**: Always use parameterized queries from sqlx.

- **Dependencies**: Don't add new crates without strong justification to keep the binary lightweight.
