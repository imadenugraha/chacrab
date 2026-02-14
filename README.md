# 🦀 ChaCrab - Zero-Knowledge Password Manager

A secure, local-first password manager built with Rust that uses Zero-Knowledge encryption to protect your credentials.

## 🔐 Security Model

ChaCrab implements **Zero-Knowledge Architecture**:
- **Client-side encryption**: All passwords are encrypted locally before storage
- **Argon2id key derivation**: Your master password is never stored, only used to derive encryption keys
- **ChaCha20-Poly1305 encryption**: Modern, authenticated encryption for all credentials
- **OS-level session management**: Derived keys are stored in your system keyring, not on disk

### What's Encrypted
- ✅ Usernames
- ✅ Passwords
- ✅ All credential data

### What's Not Encrypted (Safe to be plaintext)
- Label names (e.g., "GitHub", "Gmail")
- URLs
- Metadata (creation dates)

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/imadenugraha/chacrab.git
cd chacrab

# Build and install
cargo install --path .
```

### Prerequisites

- Rust 1.70+ (2021 edition)
- SQLite (default) or PostgreSQL

## 🚀 Quick Start

### 1. Initialize Your Vault

First time setup - creates a new vault and sets your master password:

```bash
chacrab init
```

⚠️ **Important**: Your master password is the ONLY way to decrypt your data. There is no recovery mechanism if you forget it.

### 2. Add a Credential

```bash
chacrab add
```

Or specify fields directly:

```bash
chacrab add --label "GitHub" --username "user@example.com" --url "https://github.com"
```

### 3. List Credentials

```bash
chacrab list
# or
chacrab ls
```

### 4. Retrieve a Credential

```bash
chacrab get --label "GitHub"
```

You'll be prompted to either:
- Copy the password to clipboard (secure)
- Display it in terminal

### 5. Delete a Credential

```bash
chacrab delete --label "GitHub"
# or
chacrab rm --label "GitHub"
```

### 6. Logout

Clear your session (removes encryption key from system keyring):

```bash
chacrab logout
```

## 🗄️ Database Options

### SQLite (Default)

Zero configuration, single-file database:

```bash
# Uses sqlite://chacrab.db by default
chacrab init
```

### PostgreSQL

For power users:

``bash

*Set environment variable*
export DATABASE_URL="postgres://username:password@localhost/chacrab"

*Or use command flag*
chacrab --database "postgres://username:password@localhost/chacrab" init
```

## 📁 Configuration

ChaCrab looks for configuration in the following order:

1. Command-line `--database` flag
2. `DATABASE_URL` environment variable
3. `.env` file in current directory
4. Default: `sqlite://chacrab.db`

### Example `.env` file

```bash
DATABASE_URL=sqlite://chacrab.db
```

## 🔒 Security Features

### Key Derivation (Argon2id)
- **Memory**: 64 MB
- **Iterations**: 3
- **Parallelism**: 4 threads
- **Output**: 256-bit key

These parameters balance security and performance, providing strong protection against brute-force attacks.

### Encryption (ChaCha20-Poly1305)
- **Algorithm**: ChaCha20 stream cipher with Poly1305 MAC
- **Nonce**: 96-bit unique nonce per encryption
- **Authentication**: Built-in authentication prevents tampering

### Session Management
- Encryption keys stored in OS keyring:
  - **macOS**: Keychain Access
  - **Linux**: Secret Service (libsecret) or KWallet
  - **Windows**: Credential Manager
- Keys cleared on logout
- No keys written to disk

## 📋 Command Reference

| Command | Alias | Description |
|---------|-------|-------------|
| `chacrab init` | - | Initialize new vault |
| `chacrab login` | - | Unlock vault with master password |
| `chacrab logout` | - | Lock vault and clear session |
| `chacrab add` | - | Add new credential |
| `chacrab get` | - | Retrieve and decrypt credential |
| `chacrab list` | `ls` | List all credentials (labels only) |
| `chacrab delete` | `rm` | Delete credential |

## 💾 Backup Strategy

### What to Backup

1. **Database file**: `chacrab.db` (if using SQLite)
2. **Master password**: Store securely, preferably memorized

### How to Backup

```bash
# SQLite backup
cp chacrab.db chacrab.db.backup

# Or use any file backup solution
rsync chacrab.db /path/to/backup/
```

### Restore

```bash
# Simply copy the database file back
cp chacrab.db.backup chacrab.db

# Then login with your master password
chacrab login
```

## ⚠️ Limitations

- **Single-user mode**: One master password per database file
- **No cloud sync**: Intentionally local-only for security (use file sync tools if needed)
- **No password recovery**: If you forget your master password, your data is unrecoverable
- **No mobile apps**: CLI-only tool for desktop use

## 🛠️ Development

### Running Tests

```bash
cargo test
```

### Building

```bash
cargo build --release
```

### Running from Source

```bash
cargo run -- init
cargo run -- add
cargo run -- list
```

## 🤝 Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Security Issues

If you discover a security vulnerability, please email security@example.com instead of using the issue tracker.

## 📄 License

MIT

## 🙏 Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- Uses [ChaCha20-Poly1305](https://en.wikipedia.org/wiki/ChaCha20-Poly1305) encryption
- Uses [Argon2](https://github.com/P-H-C/phc-winner-argon2) key derivation

## ❓ FAQ

### Can I use ChaCrab with multiple devices?

Yes, but you need to manually sync the database file (e.g., via Dropbox, Syncthing, etc.). ChaCrab doesn't include built-in sync to maintain security and simplicity.

### What happens if I lose my database file?

Without the database file, you cannot access your credentials. Regular backups are essential.

### Can I change my master password?

Not in MVP version. This feature requires re-encrypting all credentials and will be added in a future release.

### Is ChaCrab audited?

ChaCrab uses well-audited cryptographic libraries (`chacha20poly1305`, `argon2`) but the application itself has not undergone a formal security audit. Use at your own risk.

## 📚 Further Reading

- [Architecture Documentation](docs/architecture.md)
- [Security Deep Dive](docs/SECURITY.md)
- [Agent Development Guide](agents.md)
- [Future Improvements](docs/TODO.md)
