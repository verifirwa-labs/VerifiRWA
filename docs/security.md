# Security Model

## Overview

VerifiRWA is a permissioned protocol. Every state-changing operation requires explicit authorization, and all monetary arithmetic is overflow-safe. This document describes the security properties of each contract and the known trust assumptions.

---

## Authentication

### `env.require_auth()`

Every function that modifies state calls `env.require_auth()` on the relevant signer before any storage reads or writes. This is Soroban's native auth primitive — it verifies the transaction was signed by that address and aborts the entire invocation if not.

### Admin pattern

Each contract stores a single `Admin` address in instance storage during `initialize`. Admin-only functions load this address and compare it against the caller before proceeding:

```rust
let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
caller.require_auth();
if caller != admin {
    panic!("unauthorized");
}
```

The admin address can be a multisig (via Stellar's native multisig) or a governance contract — no changes to the protocol contracts are required.

### Cross-contract caller verification

When `rwa_registry` calls `compliance_engine.check_transfer`, the compliance contract trusts calls from the stored registry address. When `oracle_receiver` calls `rwa_registry.freeze_asset`, the registry verifies the caller matches the stored oracle address. These stored addresses are set at `initialize` time and are immutable without an upgrade.

---

## Arithmetic Safety

All monetary calculations use Rust's checked arithmetic methods:

| Operation | Method used |
|-----------|-------------|
| Addition | `checked_add(...).expect("arithmetic_overflow")` |
| Subtraction | `checked_sub(...).expect("underflow")` |
| Multiplication | `checked_mul(...).expect("arithmetic_overflow")` |
| Division | `checked_div(...).expect("div_zero")` |

Plain `+`, `-`, `*`, `/` operators are never used on monetary values. In `release` mode the workspace Cargo profile sets `overflow-checks = true` as an additional safety net.

---

## Freeze Propagation

When an asset is frozen via any path, the state update propagates to both contracts:

```
freeze_asset() called on rwa_registry
  → asset.status = Frozen (stored in rwa_registry)
  → cross-contract call → compliance_engine.freeze_asset(asset_id)
      → AssetFrozen(asset_id) = true (stored in compliance_engine)
```

This means `check_transfer` will return `false` for the asset regardless of holder whitelist status, because the compliance check happens before the holder check.

---

## Oracle Trust Model

### Current trust assumption

A single admin-authorized keypair can push oracle updates. This is a centralized trust model appropriate for testnet and early mainnet. The oracle's key should be treated as a hot key and rotated regularly using `revoke_oracle` / `authorize_oracle`.

### Staleness protection

Even if an oracle key is compromised, the damage is bounded:

- The staleness TTL (default 86400s / 24h) means stale data is automatically rejected by `get_latest_update`
- A future-dated timestamp is rejected at `push_update` time
- A stale-on-arrival timestamp (older than TTL at push time) is rejected at `push_update` time

### Upgrade path to decentralized oracles

Replace the single oracle key with [Reflector Protocol](https://reflector.network) by:

1. Calling `authorize_oracle(reflector_contract_address)`
2. Calling `revoke_oracle(old_keypair_address)`

No contract upgrade is needed. Reflector's contract will push updates using the same `push_update` interface.

---

## Emergency Controls

### Global pause

`compliance_engine.set_global_pause(true)` immediately blocks all transfers across all assets. It is admin-only and takes effect on the very next `check_transfer` call with no propagation delay.

### Asset-level freeze

`freeze_asset` on either `rwa_registry` or `compliance_engine` blocks transfers for a single asset without affecting others.

### Upgrade gate

All four contracts implement `upgrade(new_wasm_hash)` gated behind `admin.require_auth()`. Contract state (instance and persistent storage) is preserved across upgrades — only the WASM bytecode changes.

---

## Known Limitations and Trust Assumptions

| Assumption | Detail |
|------------|--------|
| Admin key security | If the admin key is compromised, all contracts can be upgraded or paused. Use a multisig admin in production. |
| Oracle centralization | A single oracle node is a single point of failure until Reflector integration is live. |
| Off-chain document integrity | `ipfs_doc_hash` is stored on-chain but the protocol does not verify IPFS content. Invoice authenticity depends on the originator's off-chain KYC process. |
| USDC contract trust | Settlement is denominated in the Circle-issued USDC SAC at the configured address. If the USDC contract address is misconfigured at `initialize` time, yield distribution will fail. |
| No reentrancy guard needed | Soroban's execution model is non-reentrant by design — a contract cannot call back into itself through a cross-contract call in the same invocation stack. |

---

## Reporting Vulnerabilities

Please do not open public GitHub issues for security vulnerabilities. Contact the maintainers directly via the organization's security contact before disclosure.
