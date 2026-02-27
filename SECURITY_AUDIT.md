# Security Audit Tracker

This document tracks security findings, remediation status, and verification evidence for Chacrab.

## How to Use

- Add a new row for each discovered security issue.
- Keep status up to date (`Open`, `In Progress`, `Mitigated`, `Verified`, `Accepted Risk`).
- Link remediation PRs/commits and verification evidence.
- Do not include plaintext secrets, keys, tokens, or sensitive payload samples.

## Severity Scale

- `Critical`: immediate compromise of confidentiality/integrity.
- `High`: significant security impact, likely exploitable.
- `Medium`: meaningful weakness requiring mitigation.
- `Low`: minor issue or hard-to-exploit weakness.
- `Info`: hardening or best-practice recommendation.

## Issue Register

| ID | Date Found | Area | Finding | Severity | Status | Owner | Mitigation Plan | Target Date | Verified Date | References |
|---|---|---|---|---|---|---|---|---|---|---|
| SEC-0001 | 2026-02-27 | Example | Replace this sample row with a real finding. | Info | Open | TBD | Define remediation steps and validation. | TBD |  | Issue/PR links |

## Verification Checklist

When marking an issue as `Verified`, confirm:

- Root cause is addressed (not only symptom-level patching).
- Regression tests or checks were added/updated where appropriate.
- No plaintext secret leakage in logs, errors, storage, or test fixtures.
- Documentation is updated if behavior or controls changed.

## Release Security Sign-off

For each release, summarize security posture:

| Release | Date | Open Critical | Open High | Sign-off | Notes |
|---|---|---:|---:|---|---|
| Unreleased | 2026-02-27 | 0 | 0 | Pending | Initial tracker created. |
