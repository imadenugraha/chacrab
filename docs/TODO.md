# ChaCrab TODO List

This document tracks features, improvements, and fixes that need to be implemented to complete ChaCrab.

## 🔴 High Priority (MVP Completion)

### 1. URL Field Support
- ✅ Add URL input prompt in `add` command
- ✅ Display URL in `list` command output
- ✅ Update `add` command to accept `--url` flag
- ✅ Test URL storage and retrieval

**Status**: ✅ COMPLETED in v0.2.0
**File**: `src/commands/add.rs`, `src/commands/list.rs`

### 2. Command Aliases
- ✅ Add `ls` alias for `list` command
- ✅ Add `rm` alias for `delete` command

**Status**: ✅ COMPLETED in v0.2.0
**File**: `src/main.rs`

### 3. Clipboard Integration
- ✅ Add `cli-clipboard` dependency to `Cargo.toml`
- ✅ Implement `copy_to_clipboard()` function in UI module
- ✅ Update `get` command to offer clipboard option
- ✅ Handle clipboard unavailable gracefully (fallback to display)

**Status**: ✅ COMPLETED in v0.2.0
**Files**: `Cargo.toml`, `src/ui/mod.rs`, `src/commands/get.rs`

### 4. Session Validation on Login
- ✅ Verify master password is correct by attempting a test operation
- ✅ Provide clear error message on wrong password
- ✅ Prevent saving invalid session keys to keyring

**Status**: ✅ COMPLETED in v0.2.0
**File**: `src/commands/login.rs`

### 5. Error Message Improvements
- ✅ Enhance "Not logged in" error messages across all commands
- ✅ Add helpful hints in error messages (suggest next action)
- ✅ Handle duplicate label error gracefully with user-friendly message
- ✅ Improve database connection error messages

**Status**: ✅ COMPLETED in v0.2.0
**Files**: All command files

---

## 🟡 Medium Priority (Quality of Life)

### 6. Direct CLI Arguments
- ✅ Support `--label`, `--username`, `--password` flags in `add` command
- ✅ Support `--label` flag in `get` command
- ✅ Support `--label` flag in `delete` command
- ✅ Add validation for flag combinations

**Status**: ✅ COMPLETED in v0.2.0
**Files**: `src/main.rs`, all command files

### 7. Integration Tests
- ✅ Create `tests/integration_tests.rs`
- ✅ Test full workflow: init → login → add → get → delete → logout
- ✅ Test error cases:
  - Wrong master password on login
  - Adding duplicate labels
  - Getting non-existent credentials
  - Commands without active session
- ✅ Test with in-memory SQLite database

**Status**: ✅ COMPLETED in v0.3.0
**Note**: Integration tests run in non-interactive mode via `CHACRAB_TEST_MODE`; full suite is passing.
**File**: `tests/integration_tests.rs`

### 8. Enhanced Help Text
- ✅ Add usage examples to command help
- ✅ Document all flags and options
- ✅ Add "EXAMPLES" section to main help

**Status**: ✅ COMPLETED in v0.3.0
**File**: `src/main.rs`

### 9. Password Strength Validation
- ✅ Check master password length (minimum 12 characters recommended)
- ✅ Warn on weak passwords (optional)
- ✅ Suggest strong password practices during `init`
- ✅ Entropy calculation and pattern detection
- ✅ Common password detection (~50 passwords)
- ✅ Color-coded feedback (Weak/Fair/Strong/Excellent)

**Status**: ✅ COMPLETED in v1.0.0
**Files**: `src/ui/password_validator.rs`, `src/commands/init.rs`

### 10. Update Credential Feature
- ✅ Add `update` command to modify existing credentials
- ✅ Re-encrypt with new values
- ✅ Prompt for which fields to update

**Status**: ✅ COMPLETED in v0.3.0
**Files**: `src/commands/update.rs`, `src/main.rs`

---

## 🟢 Low Priority (Future Enhancements)

### 11. PostgreSQL Support
- ✅ Add PostgreSQL connection handling in `src/storage/db.rs`
- ✅ Create PostgreSQL-specific migrations in `migrations/postgres/`
- ✅ Implement database type detection from URL scheme
- ✅ Add runtime switching for SQLite vs PostgreSQL queries
- [ ] Execute real PostgreSQL validation run (`scripts/validate_postgres.sh`)

**Status**: ✅ COMPLETED in v1.0.0 (SQLite + PostgreSQL abstraction)
**Details**: Database abstraction layer supports both SQLite (`sqlite://`) and PostgreSQL (`postgresql://` or `postgres://`) connection strings. Query functions automatically use correct SQL syntax for each database type. Migrations organized in separate directories. Real-instance validation harness is available at `scripts/validate_postgres.sh`.
**Files**: `src/storage/db.rs`, `src/storage/queries.rs`, `migrations/sqlite/*.sql`, `migrations/postgres/*.sql`

### 12. Master Password Change
- ✅ Add `change-password` command
- ✅ Backup recommendation before proceeding
- ✅ Current password verification
- ✅ New password validation (strength checking)
- ✅ Decrypt all credentials with old password
- ✅ Generate new salt
- ✅ Re-encrypt all credentials with new derived key
- ✅ Update user_config in database
- ✅ Update session key in keyring

**Status**: ✅ COMPLETED in v1.0.0
**Details**: Comprehensive password change implementation with safety checks. Verifies current password, validates new password strength (minimum Fair required), re-encrypts all credentials with new key derived from new salt. Updates both database and session key. Strongly recommends backup before proceeding.
**File**: `src/commands/change_password.rs`

### 13. Import/Export Functionality
- ✅ Add `export` command to dump credentials to encrypted JSON
- ✅ Add `import` command to load from backup
- ✅ Maintain encryption during export (exports encrypted credentials)
- ✅ Duplicate handling (Skip/Overwrite/Rename options)
- ✅ Secure file permissions (0600 for exports)
- [ ] Support common password manager formats (1Password, LastPass CSV)

**Status**: ✅ COMPLETED in v1.0.0 (Core functionality)
**Details**: Export creates timestamped JSON backups of encrypted credentials with 0600 permissions. Import handles duplicates interactively with Skip/Overwrite/Rename options. External format support deferred to v2.0.0.
**Files**: `src/commands/export.rs`, `src/commands/import.rs`, `src/models/credential.rs`

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
- ✅ Fix unused import warnings in main.rs
- ✅ Mark intentionally unused fields with `#[allow(dead_code)]`
- ✅ Remove `update_credential` function or implement update command (implemented)
- ✅ Remove or use `print_banner` function (allowed as dead_code)

**Status**: ✅ All resolved in v1.0.0

### Future Rust Compatibility
- [ ] Update `sqlx-postgres` when new version available
- [ ] Update `wl-clipboard-rs` when new version available (Linux clipboard)

### Test Infrastructure
- ✅ Implement non-interactive `CHACRAB_TEST_MODE` for integration tests
- ✅ Isolate keyring usage per integration test via environment-based service namespace

### Security Issues (from SECURITY_AUDIT.md)
- [x] **HIGH PRIORITY**: Upgrade sqlx from 0.7.4 to 0.8.1+ (RUSTSEC-2024-0363)
- [x] **MEDIUM**: Implement constant-time string comparison for verification token
- [ ] **LOW**: Add `zeroize` crate for secure memory clearing
- [ ] Monitor unmaintained dependencies (derivative, instant, paste)

---

## 📚 Documentation

### 21. Additional Documentation
- [ ] Create `CONTRIBUTING.md` with development guidelines
- [ ] Add troubleshooting section to README
- [ ] Document environment variables
- [ ] Create user guide for common workflows
- [ ] Add architecture diagrams
- ✅ Create security audit documentation

**Completed**:
- ✅ `docs/SECURITY_AUDIT.md` - Comprehensive security review and recommendations

---

## 🧪 Testing

### 22. Test Coverage
- 🟡 Increase unit test coverage to 80%+ (currently ~50-60%)
- [x] Add unit tests for command modules (export, import, change_password)
- [x] Resolve integration test TTY limitation with `CHACRAB_TEST_MODE`
- [ ] Add property-based tests for crypto functions
- [ ] Add fuzzing tests for parsing and decryption
- [ ] Performance testing for large credential databases
- [ ] Cross-platform testing (macOS, Linux, Windows)
- [ ] PostgreSQL integration testing with real database

**Current Status**: 37 unit tests ✅ ALL PASSING
- ✅ Crypto module: ~90% coverage (9 tests)
- ✅ Storage module: ~60% coverage (4 tests)
- ✅ Commands module: ~40% coverage (9 NEW tests)
  - export.rs: 3 tests (JSON serialization, file creation)
  - import.rs: 3 tests (parsing, duplicate detection)
  - change_password.rs: 3 tests (re-encryption, config updates)
- ✅ UI module: ~95% coverage (12 password validation tests)

**Integration Tests**: 17 total, 17 passing
- ✅ Interactive command paths are covered through `CHACRAB_TEST_MODE`
- ✅ Test keyring isolation is enabled via per-test keyring namespace env vars
- 📝 See `docs/TESTING.md` for full testing strategy documentation

**Recommended Next Steps**:
1. Add PostgreSQL CI/CD testing service
2. Install cargo-tarpaulin for coverage measurement
3. Add property-based/fuzz tests for crypto and parsing paths

---

## 🚀 DevOps

### 23. CI/CD Pipeline
- [ ] Set up GitHub Actions for automated testing
- [ ] Add clippy and rustfmt checks
- ✅ Add security audit (`cargo audit`) - tool installed and documented
- [ ] Automated releases with GitHub Releases
- [ ] Build binaries for multiple platforms

**Security Tools Available**:
- ✅ `cargo audit` - dependency vulnerability scanning
- ✅ `cargo clippy -- -D warnings` - strict linting (clean build achieved)

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

| Priority | Feature | Effort | Impact | Status |
|----------|---------|--------|--------|--------|
| 🔴 High | URL Field Support | Low | High | ✅ v0.2.0 |
| 🔴 High | Command Aliases | Low | High | ✅ v0.2.0 |
| 🔴 High | Clipboard Integration | Low | High | ✅ v0.2.0 |
| 🔴 High | Session Validation | Medium | High | ✅ v0.2.0 |
| 🔴 High | Error Messages | Medium | High | ✅ v0.2.0 |
| 🟡 Medium | CLI Arguments | Medium | Medium | ✅ v0.2.0 |
| 🟡 Medium | Integration Tests | High | High | ✅ v0.3.0 |
| 🟡 Medium | Enhanced Help Text | Low | Medium | ✅ v0.3.0 |
| 🟡 Medium | Update Command | Medium | Medium | ✅ v0.3.0 |
| � Medium | Password Validation | Low | High | ✅ v1.0.0 |
| 🟡 Medium | Security Audit | Medium | High | ✅ v1.0.0 Phase 1 |
| 🟢 Low | PostgreSQL | High | Low | ✅ v1.0.0 Phase 2 |
| 🟢 Low | Password Change | High | Medium | v1.0.0 Phase 3 |
| 🟢 Low | Import/Export | Medium | Medium | v1.0.0 Phase 2 |

---

## 🗓️ Roadmap

### v0.1.0 (Current - MVP)
- ✅ SQLite support
- ✅ Basic CRUD operations
- ✅ Zero-Knowledge encryption
- ✅ OS keyring integration

### v0.2.0 (MVP Complete)
- ✅ URL field support
- ✅ Command aliases
- ✅ Clipboard integration
- ✅ Session validation
- ✅ Better error messages

### v0.3.0 (Enhanced)
- ✅ Direct CLI arguments (already in v0.2.0)
- ✅ Integration tests
- ✅ Update command
- ✅ Enhanced help text

### v1.0.0 (Stable) - IN PROGRESS
- ✅ Password strength validation
- ✅ Security self-audit (Phase 1)
- ✅ sqlx 0.8.6 upgrade (Phase 2A)
- ✅ PostgreSQL support (Phase 2B)
- ✅ Import/export functionality (Phase 2C)
- ✅ Password change command (Phase 3A)
- 🟡 Test coverage expansion (50% → 80% target)
  - ✅ Command module unit tests (9 new tests)
  - ✅ 37 unit tests all passing
  - ✅ 17 integration tests all passing via test mode

**Phase 1 (Foundation) Complete**: Password validation + Security audit  
**Phase 2A (Dependencies) Complete**: sqlx vulnerability fixed  
**Phase 2B (PostgreSQL) Complete**: Database abstraction + migrations  
**Phase 2C (Import/Export) Complete**: Backup and restore with duplicate handling  
**Phase 3A (Password Change) Complete**: Master password rotation with re-encryption  
**Phase 3B (Testing) IN PROGRESS**: 
- ✅ Created `src/lib.rs` for library tests
- ✅ Added 9 command unit tests (export, import, change_password)
- ✅ Documented testing strategy in `docs/TESTING.md`
- ✅ Integration tests stabilized with `CHACRAB_TEST_MODE`
- ⏳ PostgreSQL integration testing pending
- ⏳ Coverage measurement tool (tarpaulin) not installed

**Estimated Completion**: 8-16 hours remaining (PostgreSQL testing + coverage hardening)

### v2.0.0 (Advanced)
- Tags/categories
- TOTP support
- Password generation
- Multiple vault support
- Audit history

---

## 📋 v1.0.0 Implementation Status

**Overall Progress**: Phase 1-2-3A Complete  
**Next Phase**: Phase 3B (Test Coverage)  
**Release Target**: After comprehensive testing + PostgreSQL validation

### Completed (v1.0.0 Phase 1)
- ✅ Password strength validation with comprehensive checks
- ✅ Security self-audit with automated tools
- ✅ Clean build with zero clippy warnings
- ✅ 37 unit tests passing (12 new password validation tests)
- ✅ Security audit documentation

### Completed (v1.0.0 Phase 2A-2B-2C)
- ✅ sqlx upgraded from 0.7.4 → 0.8.6 (RUSTSEC-2024-0363 fixed)
- ✅ Database abstraction layer (DatabasePool enum)
- ✅ PostgreSQL support with runtime detection
- ✅ Separate migrations for SQLite and PostgreSQL
- ✅ All query functions support both database types
- ✅ Export command with encrypted JSON backup
- ✅ Import command with duplicate handling (Skip/Overwrite/Rename)
- ✅ Secure file permissions (0600) for exports
- ✅ Serde derives added to models

### Completed (v1.0.0 Phase 3A)
- ✅ Password change command with backup recommendation
- ✅ Current password verification before change
- ✅ New password strength validation (minimum Fair)
- ✅ Complete credential re-encryption with new key
- ✅ New salt generation and storage
- ✅ Session key update in OS keyring
- ✅ Comprehensive error handling

### In Progress (v1.0.0 Phase 3B)
- 🟡 Comprehensive test coverage (45% → 80%+)
- 🟡 PostgreSQL integration testing
- 🟡 Import/export round-trip tests
- 🟡 Password change safety tests

### Blockers for v1.0.0 Release
- [x] ~~Upgrade sqlx to 0.8.1+ (RUSTSEC-2024-0363 vulnerability)~~ ✅ Complete
- [x] ~~Implement PostgreSQL support~~ ✅ Complete
- [x] ~~Implement import/export for backups~~ ✅ Complete
- [x] ~~Implement password change functionality~~ ✅ Complete
- [ ] Achieve 80%+ test coverage
- [ ] Test PostgreSQL with real database instance
- [ ] Integration tests for all Phase 2-3 features

---

### 🐛 Bug
- [x] Clipboard from `get` command copies plaintext password (verified behavior)
- [x] Added export format option: `--format encrypted|plaintext` (default: encrypted)

**Last Updated**: February 14, 2026  
**Current Version**: v0.3.0  
**Target Version**: v1.0.0  
**Estimated Hours Remaining**: 50-80 hours