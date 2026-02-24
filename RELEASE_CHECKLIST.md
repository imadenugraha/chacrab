# Release Checklist

Use this checklist before creating a release tag.

## Pre-Release

- [ ] Confirm target version is updated in [Cargo.toml](Cargo.toml).
- [ ] Update [CHANGELOG.md](CHANGELOG.md) (`Unreleased` → version/date section).
- [ ] Verify docs are aligned: [README.md](README.md), [ARCHITECTURE.md](ARCHITECTURE.md), [TODO.md](TODO.md).
- [ ] Confirm no plaintext secrets are introduced in storage paths.

## Required Quality Gates

- [ ] `cargo check`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`
- [ ] CI workflow in [.github/workflows/ci.yml](.github/workflows/ci.yml) is green on main.

## Security Validation

- [ ] Validate auth flow: `init`, `login`, `logout`.
- [ ] Validate vault flow: `add-password`, `list`, `show`, `delete`.
- [ ] Validate encrypted backup flow: `backup-export`, `backup-import`.
- [ ] Validate sync flow with remote backend env vars:
  - `CHACRAB_SYNC_BACKEND`
  - `CHACRAB_SYNC_DATABASE_URL`
- [ ] Verify DB rows contain encrypted blob + nonce only (no plaintext password/notes).

## Release

- [ ] Create annotated git tag `vX.Y.Z`.
- [ ] Publish release notes from [CHANGELOG.md](CHANGELOG.md).
- [ ] Attach build artifacts if distributing binaries.

## Post-Release

- [ ] Create new `Unreleased` section entries in [CHANGELOG.md](CHANGELOG.md).
- [ ] Announce release and include upgrade notes if behavior changed.
