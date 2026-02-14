# ChaCrab Testing Strategy

## Overview

ChaCrab uses a hybrid testing approach with unit tests and integration tests. This document explains the testing architecture, current coverage, and known limitations.

## Test Types

### Unit Tests (37 tests - ✅ ALL PASSING)

Unit tests are located within module files using `#[cfg(test)]` blocks. These tests focus on testing individual functions and logic in isolation.

**Test Distribution:**
- **Crypto Module** (9 tests):
  - `crypto/derive.rs`: 4 tests (key derivation, salt handling, determinism)
  - `crypto/encrypt.rs`: 5 tests (encryption/decryption, tamper detection, unicode support)

- **Storage Module** (4 tests):
  - `storage/db.rs`: 1 test (in-memory database initialization)
  - `storage/queries.rs`: 2 tests (credential + user_config operations)
  - `storage/keyring.rs`: 1 test (keyring roundtrip)

- **UI Module** (12 tests):
  - `ui/password_validator.rs`: 12 tests (strength levels, entropy, common passwords, suggestions)

- **Command Module** (9 tests - NEW):
  - `commands/export.rs`: 3 tests (JSON serialization, multiple credentials, file creation)
  - `commands/import.rs`: 3 tests (JSON parsing, duplicate detection, multiple imports)
  - `commands/change_password.rs`: 3 tests (re-encryption logic, user config updates, credential updates)

### Integration Tests (17 tests - ✅ ALL PASSING)

Integration tests are located in `tests/integration_tests.rs` and use `assert_cmd` to test the CLI as a whole. Interactive command paths are handled via `CHACRAB_TEST_MODE` to keep tests deterministic.

**Status:**
- ✅ 17 PASSING: Full critical-path coverage including interactive command flows in test mode

**How Integration Tests Work in Non-TTY:**
`dialoguer` requires a real TTY for interactive prompts. ChaCrab uses `CHACRAB_TEST_MODE=1` and command-specific environment variables to provide non-interactive input paths during test execution.

**Test-mode covered commands:**
- `init`, `login`, `add`, `delete`, `export`, `import`, `change-password`

## Running Tests

### Run All Unit Tests
```bash
cargo test --lib
```

### Run Specific Module Tests
```bash
cargo test --lib crypto::
cargo test --lib commands::export
```

### Run Integration Tests
```bash
cargo test --test integration_tests
```

### Run All Tests (hybrid)
```bash
cargo test
```

## Test Coverage

**Current Coverage:** ~50-60% (estimate based on unit tests)
**Target Coverage:** 80%

**Coverage by Module:**
- ✅ Crypto: ~90% (excellent)
- ✅ Password Validation: ~95% (excellent)
- 🟡 Storage: ~60% (good, PostgreSQL paths need more coverage)
- 🟡 Commands: ~40% (unit tests added, but missing end-to-end testing)
- ✅ Interactive UI paths: Tested via `CHACRAB_TEST_MODE`

## Known Limitations

### 1. Interactive Prompt Testing

**Current Approach:** Commands using `dialoguer` are tested through explicit non-interactive test-mode branches enabled by `CHACRAB_TEST_MODE=1`.

**Important:** Test mode is for automated testing only; default runtime behavior remains fully interactive.

### 2. PostgreSQL Integration Testing

**Status:** PostgreSQL validation harness is implemented and can be run against a real instance.

**Run Local Validation (real PostgreSQL):**

```bash
export CHACRAB_POSTGRES_TEST_URL="postgres://user:pass@localhost:5432/chacrab_test"
export CHACRAB_TEST_MASTER_PASSWORD="testpass123"
./scripts/validate_postgres.sh
```

**Optional Integration Test (env-gated):**

```bash
CHACRAB_POSTGRES_TEST_URL="postgres://user:pass@localhost:5432/chacrab_test" \
CHACRAB_TEST_MASTER_PASSWORD="testpass123" \
cargo test --test integration_tests test_postgres_real_instance_workflow
```

**Validation checklist covered by script:**
- Initialize or reuse vault on PostgreSQL
- Login/session checks via keyring
- Add/list/update/delete credential lifecycle
- Export/import encrypted backup round-trip
- Logout cleanup

### 3. OS Keyring Testing

**Status:** Keyring tests use real OS keyring (Success Keyring on Linux, Keychain on macOS, Credential Manager on Windows).

**Limitation:** Tests may interfere with actual ChaCrab sessions if service name collides.

**Mitigation:** Tests use unique service names to avoid conflicts.

## Adding New Tests

### Adding Unit Tests

1. Create `#[cfg(test)]` module at end of file:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feature() {
        // Test code
    }
}
```

2. For async tests, use `#[tokio::test]`:
```rust
#[tokio::test]
async fn test_async_feature() {
    // Async test code
}
```

3. Use in-memory SQLite for database tests:
```rust
let db = init_db("sqlite::memory:").await.unwrap();
let pool = db.pool();
```

### Adding Integration Tests

Integration tests can include interactive command flows when they use `CHACRAB_TEST_MODE=1` and provide the required env vars.

Recommended focus:
- Critical CLI workflows (init/login/add/get/delete/export/import/change-password)
- Help/version and command discovery flows
- Error-path assertions for authentication and duplicates

## Future Improvements

### Short-term (v1.0.0)
- [x] Add unit tests for command modules
- [x] Document test-mode environment variable approach
- [x] Add PostgreSQL integration test setup guide

### Medium-term (v1.1.0)
- [x] Implement `CHACRAB_TEST_MODE` environment variable
- [x] Update integration tests to use test mode
- [ ] Increase command module coverage to 80%

### Long-term (v2.0.0)
- [ ] Set up CI/CD with PostgreSQL service
- [ ] Add coverage measurement (cargo-tarpaulin or cargo-llvm-cov)
- [ ] Implement property-based testing (proptest) for crypto
- [ ] Add fuzzing for parser (cargo-fuzz)

## CI/CD Considerations

**Current State:** No CI/CD configured

**Recommended Setup:**
1. GitHub Actions workflow
2. Test matrix: Linux, macOS, Windows
3. PostgreSQL service for integration tests
4. Code coverage reporting (Codecov)
5. Security audit (cargo-audit)

**Test Stages:**
```yaml
stages:
  - Fast Tests: Unit tests only (~7 seconds)
  - Full Tests: Unit + Integration (~20 seconds)
  - Coverage Report: Measure and upload
  - Security Audit: cargo-audit, cargo-deny
```

## Debugging Failed Tests

### TTY Error in Integration Tests
```
Error: Failed to read master password
Caused by:
    0: IO error: not a terminal
    1: not a terminal
```

**Cause:** Test is trying to use `dialoguer` prompts in non-TTY environment.

**Solution:** Ensure test command includes `CHACRAB_TEST_MODE=1` and required input env vars.

### Database Lock Errors
```
Error: database is locked
```

**Cause:** Multiple tests trying to use same database file.

**Solution:** Use in-memory databases (`sqlite::memory:`) or unique temp files per test.

### Keyring Access Errors
```
Error: Failed to access keyring
```

**Cause:** OS keyring service unavailable or permission denied.

**Solution:** Run tests on system with keyring support, or mock keyring access.

## Test Maintenance

- **Run tests before every commit**: `cargo test`
- **Update tests when changing APIs**: Keep tests synchronized with implementation
- **Document test failures**: Add known issues to this file
- **Review test coverage regularly**: Aim for 80% coverage target

---

**Last Updated:** February 14, 2026  
**Test Count:** 37 unit tests (all passing), 17 integration tests (all passing)  
**Coverage:** ~50-60% (estimated; formal coverage tooling pending)
