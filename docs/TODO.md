# ChaCrab TODO List

This document tracks features, improvements, and fixes that need to be implemented to complete ChaCrab.

## 🔴 High Priority (MVP Completion)

### 1. URL Field Support
- [ ] Add URL input prompt in `add` command
- [ ] Display URL in `list` command output
- [ ] Update `add` command to accept `--url` flag
- [ ] Test URL storage and retrieval

**File**: `src/commands/add.rs`, `src/commands/list.rs`

### 2. Command Aliases
- [ ] Add `ls` alias for `list` command
- [ ] Add `rm` alias for `delete` command

**File**: `src/main.rs`

### 3. Clipboard Integration
- [ ] Add `cli-clipboard` dependency to `Cargo.toml`
- [ ] Implement `copy_to_clipboard()` function in UI module
- [ ] Update `get` command to offer clipboard option
- [ ] Handle clipboard unavailable gracefully (fallback to display)

**Files**: `Cargo.toml`, `src/ui/mod.rs`, `src/commands/get.rs`

### 4. Session Validation on Login
- [ ] Verify master password is correct by attempting a test operation
- [ ] Provide clear error message on wrong password
- [ ] Prevent saving invalid session keys to keyring

**File**: `src/commands/login.rs`

### 5. Error Message Improvements
- [ ] Enhance "Not logged in" error messages across all commands
- [ ] Add helpful hints in error messages (suggest next action)
- [ ] Handle duplicate label error gracefully with user-friendly message
- [ ] Improve database connection error messages

**Files**: All command files

---

## 🟡 Medium Priority (Quality of Life)

### 6. Direct CLI Arguments
- [ ] Support `--label`, `--username`, `--password` flags in `add` command
- [ ] Support `--label` flag in `get` command
- [ ] Support `--label` flag in `delete` command
- [ ] Add validation for flag combinations

**Files**: `src/main.rs`, all command files

### 7. Integration Tests
- [ ] Create `tests/integration_tests.rs`
- [ ] Test full workflow: init → login → add → get → delete → logout
- [ ] Test error cases:
  - Wrong master password on login
  - Adding duplicate labels
  - Getting non-existent credentials
  - Commands without active session
- [ ] Test with in-memory SQLite database

**File**: `tests/integration_tests.rs`

### 8. Enhanced Help Text
- [ ] Add usage examples to command help
- [ ] Document all flags and options
- [ ] Add "EXAMPLES" section to main help

**File**: `src/main.rs`

### 9. Password Strength Validation
- [ ] Check master password length (minimum 12 characters recommended)
- [ ] Warn on weak passwords (optional)
- [ ] Suggest strong password practices during `init`

**File**: `src/commands/init.rs`

### 10. Update Credential Feature
- [ ] Add `update` command to modify existing credentials
- [ ] Re-encrypt with new values
- [ ] Prompt for which fields to update

**Files**: `src/commands/update.rs`, `src/main.rs`

---

## 🟢 Low Priority (Future Enhancements)

### 11. PostgreSQL Support
- [ ] Add PostgreSQL connection handling in `src/storage/db.rs`
- [ ] Create PostgreSQL-specific migrations in `migrations-postgres/`
- [ ] Implement database type detection from URL scheme
- [ ] Add runtime switching for SQLite vs PostgreSQL queries
- [ ] Test with real PostgreSQL instance

**Files**: `src/storage/db.rs`, `migrations-postgres/*.sql`

### 12. Master Password Change
- [ ] Add `change-password` command
- [ ] Decrypt all credentials with old password
- [ ] Generate new salt
- [ ] Re-encrypt all credentials with new derived key
- [ ] Update user_config in database

**File**: `src/commands/change_password.rs`

### 13. Import/Export Functionality
- [ ] Add `export` command to dump credentials to encrypted JSON
- [ ] Add `import` command to load from backup
- [ ] Support common password manager formats (1Password, LastPass CSV)
- [ ] Maintain encryption during export

**Files**: `src/commands/export.rs`, `src/commands/import.rs`

### 14. Search and Filter
- [ ] Add search functionality to `list` command
- [ ] Filter by label substring
- [ ] Filter by URL domain
- [ ] Sort options (alphabetical, date created, date modified)

**File**: `src/commands/list.rs`

### 15. Tags/Categories Support
- [ ] Add tags field to credentials table
- [ ] Allow tagging credentials during add
- [ ] Filter by tags in list command
- [ ] Support multiple tags per credential

**Files**: Migration, models, commands

### 16. Audit/History Feature
- [ ] Track access history (when credentials were retrieved)
- [ ] Track modification history
- [ ] Add `audit` command to view history
- [ ] Identify stale credentials (not accessed in X months)

**Files**: New migration, new commands

### 17. Security Enhancements
- [ ] Implement memory locking (mlock) for sensitive data
- [ ] Add session timeout (auto-logout after X minutes)
- [ ] Add failed login attempt tracking
- [ ] Implement secure memory zeroization on sensitive data

**Files**: Various

### 18. Multiple Vault Support
- [ ] Support multiple database files (vaults)
- [ ] Add vault switching functionality
- [ ] List available vaults
- [ ] Set default vault

**Files**: Configuration, main CLI

### 19. TOTP/2FA Support
- [ ] Store TOTP secrets
- [ ] Generate TOTP codes on demand
- [ ] Display countdown timer
- [ ] Copy codes to clipboard

**Files**: New models, new commands, crypto module

### 20. Password Generation
- [ ] Add `generate` command for secure password generation
- [ ] Configurable length and character sets
- [ ] Generate passphrases (diceware style)
- [ ] Directly use generated password in `add` command

**File**: `src/commands/generate.rs`

---

## 🐛 Known Issues

### Warnings to Address
- [ ] Fix unused import warnings in main.rs
- [ ] Mark intentionally unused fields with `#[allow(dead_code)]`
- [ ] Remove `update_credential` function or implement update command
- [ ] Remove or use `print_banner` function

### Future Rust Compatibility
- [ ] Update `sqlx-postgres` when new version available
- [ ] Update `wl-clipboard-rs` when new version available (Linux clipboard)

---

## 📚 Documentation

### 21. Additional Documentation
- [ ] Create `CONTRIBUTING.md` with development guidelines
- [ ] Add troubleshooting section to README
- [ ] Document environment variables
- [ ] Create user guide for common workflows
- [ ] Add architecture diagrams

---

## 🧪 Testing

### 22. Test Coverage
- [ ] Increase unit test coverage to 80%+
- [ ] Add property-based tests for crypto functions
- [ ] Add fuzzing tests for parsing and decryption
- [ ] Performance testing for large credential databases
- [ ] Cross-platform testing (macOS, Linux, Windows)

---

## 🚀 DevOps

### 23. CI/CD Pipeline
- [ ] Set up GitHub Actions for automated testing
- [ ] Add clippy and rustfmt checks
- [ ] Add security audit (`cargo audit`)
- [ ] Automated releases with GitHub Releases
- [ ] Build binaries for multiple platforms

### 24. Distribution
- [ ] Publish to crates.io
- [ ] Create Homebrew formula (macOS)
- [ ] Create AUR package (Arch Linux)
- [ ] Create .deb package (Debian/Ubuntu)
- [ ] Docker container (optional)

---

## 🎨 User Experience

### 25. UI/UX Improvements
- [ ] Add color-coded output (errors in red, success in green)
- [ ] Add progress indicators for slow operations
- [ ] Improve table formatting in `list` command
- [ ] Add interactive credential selection (fuzzy finder)
- [ ] Support for custom output formats (JSON, CSV)

### 26. Configuration File
- [ ] Support config file (`~/.chacrab/config.toml`)
- [ ] Configurable defaults (database path, clipboard timeout)
- [ ] Per-vault configuration

---

## 📊 Priority Matrix

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| 🔴 High | URL Field Support | Low | High |
| 🔴 High | Command Aliases | Low | High |
| 🔴 High | Clipboard Integration | Low | High |
| 🔴 High | Session Validation | Medium | High |
| 🔴 High | Error Messages | Medium | High |
| 🟡 Medium | CLI Arguments | Medium | Medium |
| 🟡 Medium | Integration Tests | High | High |
| 🟡 Medium | Update Command | Medium | Medium |
| 🟢 Low | PostgreSQL | High | Low |
| 🟢 Low | Password Change | High | Medium |

---

## 🗓️ Roadmap

### v0.1.0 (Current - MVP)
- ✅ SQLite support
- ✅ Basic CRUD operations
- ✅ Zero-Knowledge encryption
- ✅ OS keyring integration

### v0.2.0 (MVP Complete)
- 🔴 URL field support
- 🔴 Command aliases
- 🔴 Clipboard integration
- 🔴 Session validation
- 🔴 Better error messages

### v0.3.0 (Enhanced)
- 🟡 Direct CLI arguments
- 🟡 Integration tests
- 🟡 Update command
- 🟡 Enhanced help text

### v1.0.0 (Stable)
- 🟢 PostgreSQL support
- 🟢 Password change
- 🟢 Import/export
- 🟢 Full test coverage
- 🟢 Security audit

### v2.0.0 (Advanced)
- Tags/categories
- TOTP support
- Password generation
- Multiple vault support
- Audit history

---

**Last Updated**: February 14, 2026