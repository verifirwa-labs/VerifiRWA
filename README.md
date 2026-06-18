# VerifiRWA

**Permissioned invoice and trade receivable tokenization on Stellar Soroban**

[![Build](https://github.com/your-org/verifirwa/actions/workflows/ci.yml/badge.svg)](https://github.com/your-org/verifirwa/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Network: Stellar](https://img.shields.io/badge/Network-Stellar%20Soroban-7d00ff)](https://stellar.org)

---

## The Problem

SMEs globally hold **trillions in unpaid invoices**. They wait 60-90 days for payment or pay 5-8% to traditional factoring companies for early liquidity. Meanwhile, institutional and retail investors have no direct access to these high-yield, short-duration instruments.

## The Solution

VerifiRWA tokenizes invoices and trade receivables on Stellar Soroban:

- **SMEs** get immediate liquidity by selling fractional invoice positions
- **Investors** earn 8-15% APY on short-duration instruments settled in USDC
- **Settlement** is automatic — when the debtor pays, USDC flows directly to token holders

---

## Architecture

Four Soroban smart contracts with one TypeScript SDK:

```
┌──────────────────┐   check_transfer()   ┌──────────────────────┐
│   rwa_registry   │ ───────────────────► │  compliance_engine   │
│   (orchestrator) │                      │  (gatekeeper)        │
└────────┬─────────┘                      └──────────────────────┘
         │                                          ▲
         │ queue_distribution()                     │ freeze_asset()
         ▼                                          │
┌──────────────────┐              ┌─────────────────┴──┐
│ yield_distributor│              │  oracle_receiver    │
│ (USDC settlement)│              │  (off-chain bridge) │
└──────────────────┘              └────────────────────┘
```

| Contract | Purpose |
|----------|---------|
| `rwa_registry` | Mint assets, manage lifecycle, track holder balances |
| `compliance_engine` | Whitelist, KYC, jurisdiction rules, freeze/clawback |
| `yield_distributor` | Proportional USDC yield distribution (pull pattern) |
| `oracle_receiver` | Verified off-chain data with staleness checks |

---

## Testnet Contract Addresses

| Contract | Address |
|----------|---------|
| rwa_registry | TBD |
| compliance_engine | TBD |
| yield_distributor | TBD |
| oracle_receiver | TBD |

---

## Quick Start

```bash
# 1. Clone
git clone https://github.com/your-org/verifirwa
cd verifirwa

# 2. Build
cargo build --target wasm32-unknown-unknown --release --workspace

# 3. Test
cargo test --workspace

# 4. Deploy to testnet
cp .env.example .env   # fill in ADMIN_SECRET_KEY
bash scripts/deploy.sh

# 5. Install TypeScript SDK
cd sdk/typescript && npm install && npm run build
```

---

## TypeScript SDK — Mint an Asset in 10 Lines

```typescript
import { createVerifiRwaClients, AssetStatus } from "@verifirwa/sdk";
import { Keypair } from "@stellar/stellar-sdk";

const clients = createVerifiRwaClients({
  registryContractId: process.env.RWA_REGISTRY_CONTRACT!,
  complianceContractId: process.env.COMPLIANCE_ENGINE_CONTRACT!,
  yieldContractId: process.env.YIELD_DISTRIBUTOR_CONTRACT!,
  oracleContractId: process.env.ORACLE_RECEIVER_CONTRACT!,
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcUrl: "https://soroban-testnet.stellar.org",
});

const keypair = Keypair.fromSecret(process.env.ADMIN_SECRET_KEY!);
const assetId = await clients.registry.mintAsset(keypair, {
  assetId: "INV-2024-001",
  faceValue: 100_000_0000000n,   // 100,000 USDC
  maturityTimestamp: BigInt(Math.floor(Date.now() / 1000) + 86400 * 30),
  originator: keypair.publicKey(),
  debtor: "ACME_CORP",
  assetType: "INVOICE",
  status: AssetStatus.Active,
  tokenSupply: 100_000_0000000n,
  createdAt: 0n,
  ipfsDocHash: "QmXyz123",
});
console.log("Asset minted:", assetId);
```

---

## Security

- `env.require_auth()` on every state-changing function
- All arithmetic uses checked operations (no silent overflows)
- Global emergency pause via `compliance_engine.set_global_pause`
- Oracle data TTL prevents stale data from affecting transfers
- Upgrade pattern gated behind admin auth

See [docs/architecture.md](docs/architecture.md) for the full security model.

---

## Contributing

This project participates in the **[Drips Stellar Wave Program](https://www.drips.network/wave/stellar)**. Issues labeled `Stellar Wave` are eligible for Wave point allocations.

See [CONTRIBUTING.md](CONTRIBUTING.md) for branch naming, PR checklist, and how to pick Wave issues.

---

## License

MIT — see [LICENSE](LICENSE).
