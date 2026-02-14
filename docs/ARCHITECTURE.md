# 🏗️ ChaCrab: Technical Architecture

This document explains the system design, security model, and data schema used in ChaCrab.

---

## 🛡️ Security Model (Zero-Knowledge)

ChaCrab uses the **Zero-Knowledge Architecture** principle. This means the server (Supabase) never knows the user's _Master Password_ or the contents of stored data.

### 1. Key Derivation (Argon2id)

Before encryption is performed, the _Master Password_ is converted into a strong cryptographic key.

- **Input**: Master Password + Unique User Salt
- **Algorithm**: Argon2id (Configuration: Memory 64MB, Iterations 3, Parallelism 4)
- **Output**: 32-byte _Derived Key_

### 2. Authenticated Encryption (ChaCha20-Poly1305)

We use a modern _Stream Cipher_ that is very fast and secure.

- Each data entry uses a unique **Nonce** (12-byte) randomly generated via `OsRng`
- Encrypted data includes: Username, Password, and Secret Note Contents

---
    

## 🔄 Data Flow: Create New Credential

1. **User Input**: User enters label, username, and password via CLI

2. **Encryption**:
   - System retrieves _Derived Key_ from memory/OS Keyring
   - Generates random _Nonce_
   - Performs local encryption on username & password

3. **Transmission**: Encrypted data is sent to Supabase via HTTPS

4. **Storage**: Supabase stores _Ciphertext_ and _Nonce_

---
    

## 🗄️ Database Schema

ChaCrab uses PostgreSQL on the Supabase platform. Below is the main table structure:

### 1. Table: `user_configs`

Stores basic information for the login process and key derivation.

| Column     | Type        | Description                        |
|------------|-------------|------------------------------------|
| user_id    | UUID (PK)   | Relation to auth.users Supabase    |
| salt       | TEXT        | Unique salt per user for Argon2id  |
| updated_at | TIMESTAMPTZ | Last update timestamp              |

### 2. Table: `credentials`

Stores encrypted account data.

| Column       | Type      | Description                          |
|--------------|-----------|--------------------------------------|
| id           | UUID (PK) | Unique entry ID                      |
| user_id      | UUID (FK) | Data owner (with RLS enabled)        |
| label        | TEXT      | Application/service name (Plaintext) |
| url          | TEXT      | Service URL (Plaintext)              |
| enc_username | TEXT      | Encrypted username (Base64)          |
| enc_password | TEXT      | Encrypted password (Base64)          |
| nonce        | TEXT      | Unique nonce for this row (Base64)   |

### 3. Table: `secret_notes`

Stores encrypted long-form text notes.

| Column      | Type      | Description                             |
|-------------|-----------|-----------------------------------------|
| id          | UUID (PK) | Unique entry ID                         |
| title       | TEXT      | Note title (Plaintext)                  |
| enc_content | TEXT      | Encrypted note content (Base64)         |
| nonce       | TEXT      | Unique nonce for this row (Base64)      |
| tags        | TEXT[]    | Label array for grouping (Plaintext)    |

---

## 🔑 Session Management (OS Keyring)

To maintain convenience without sacrificing security, ChaCrab uses **OS-level secure storage**:

- **macOS**: Keychain Access
- **Linux**: Secret Service (libsecret) or KWallet
- **Windows**: Credential Manager

The _Derived Key_ is stored here after successful login and automatically deleted when the user runs the logout command.

---

## 📂 Project Structure

The code structure is organized modularly to facilitate unit testing:

```plaintext
src/
├── main.rs          # CLI argument handler (Clap)
├── crypto/          # Pure Argon2 & ChaCha20 logic
├── storage/         # SQLx implementation and Supabase connection
├── commands/        # Business logic per command (Add, Get, Ls)
├── models/          # Data structures (Structs for DB rows)
└── ui/              # Terminal output formatting & prompts
```
