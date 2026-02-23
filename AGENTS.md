# AGENTS.md

## Purpose
This repository contains **Chacrab**, a security-first CLI password manager.

## Architecture
- `src/bin/main.rs`: binary entry point
- `src/core/*`: domain models, crypto, errors, vault service
- `src/storage/*`: repository trait + backend implementations
- `src/auth/*`: registration/login/logout and keyring session key management
- `src/sync/*`: encrypted-blob sync engine scaffolding
- `src/cli/*`: clap parser and command handlers

## Security Invariants
1. Never store plaintext secrets.
2. Derive keys with Argon2id (`m=65536`, `t=3`, `p=1`).
3. Encrypt secrets with ChaCha20-Poly1305 and 96-bit random nonce.
4. Store only salt + verifier for auth bootstrap; do not persist derived key in DB.
5. Keep session key in OS keyring only; fail closed if keyring is unavailable.
6. Zeroize sensitive buffers after use (`zeroize` crate).
7. Do not log secrets, keys, plaintext payloads, or raw crypto errors.

## Agent Rules
- Preserve module boundaries and trait abstractions.
- Prefer explicit error propagation; avoid panics in security-sensitive paths.
- Avoid introducing `unsafe` unless strictly required and justified.
- Keep storage records encrypted at rest (`encrypted_data`, `nonce`, metadata only).

## Validation Checklist
- `cargo check`
- confirm `init`, `login`, `add-password`, `list`, `show`, `logout` flows
- verify DB rows do not contain plaintext password/notes
