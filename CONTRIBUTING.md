# Contributing to VerifiRWA

VerifiRWA is an open-source Stellar Soroban protocol targeting the [Drips Stellar Wave Program](https://www.drips.network/wave/stellar). All contributions that move the protocol closer to production readiness are welcome.

---

## Project Overview

VerifiRWA tokenizes invoices and trade receivables on Stellar Soroban. The protocol consists of four Rust/Soroban smart contracts and a TypeScript SDK. See [docs/architecture.md](docs/architecture.md) for the full design.

---

## Picking Issues (Drips Wave Contributors)

All Wave-eligible issues are labeled `Stellar Wave` in the GitHub Issues tab. Point allocations:
- **200 points** — High-complexity contract features
- **150 points** — Medium-complexity features and integration tests
- **100 points** — Documentation and CI/DevOps

Before picking an issue:
1. Check that no one else is already working on it (look for assignee or a comment)
2. Comment on the issue to claim it
3. Reference the issue number in your PR description

---

## Branch Naming

| Type | Pattern | Example |
|------|---------|---------|
| Feature | `feat/*` | `feat/mint-asset-validation` |
| Bug fix | `fix/*` | `fix/stale-oracle-check` |
| Documentation | `docs/*` | `docs/update-architecture` |
| Test | `test/*` | `test/compliance-kyc-gate` |

---

## PR Checklist

Before opening a pull request, verify:

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` is clean
- [ ] `cargo fmt --check` passes (run `cargo fmt` to fix)
- [ ] New public functions have `///` doc comments
- [ ] Integration test updated if behavior changes
- [ ] PR description references the issue number (`Closes #N`)
- [ ] PR title is concise and describes the change (not the ticket)

---

## Issue Template

When filing a new issue, use this format:

```
**Summary**
One sentence describing the bug or feature.

**Context**
Which contract is affected? What is the expected behavior vs actual?

**Definition of Done**
- [ ] Specific acceptance criterion 1
- [ ] Specific acceptance criterion 2
- [ ] Unit tests covering the new behavior

**Labels**
enhancement / bug / documentation + Stellar Wave
```

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). Be respectful, constructive, and collaborative.
