/**
 * TypeScript type definitions mirroring all VerifiRWA Soroban contract types.
 */

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

/** Lifecycle status of a tokenised real-world asset. */
export enum AssetStatus {
  Active = "Active",
  Settled = "Settled",
  Defaulted = "Defaulted",
  Frozen = "Frozen",
}

/** Status of a yield distribution round. */
export enum RoundStatus {
  Pending = "Pending",
  Active = "Active",
  Completed = "Completed",
}

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/** Full on-chain record for a tokenised invoice or trade receivable. */
export interface AssetMetadata {
  /** Unique identifier for this asset, e.g. "INV-2024-001". */
  assetId: string;
  /** Face value in USDC stroops (7 decimal places). */
  faceValue: bigint;
  /** Unix timestamp after which the debtor is expected to pay. */
  maturityTimestamp: bigint;
  /** Stellar address of the originator that minted this asset. */
  originator: string;
  /** Hashed or encoded identifier of the debtor company. */
  debtor: string;
  /** Asset class: "INVOICE" | "RECEIVABLE" | "TRADE_CREDIT". */
  assetType: string;
  /** Current lifecycle status. */
  status: AssetStatus;
  /** Total token supply; equal to faceValue. */
  tokenSupply: bigint;
  /** Ledger timestamp when this asset was minted. */
  createdAt: bigint;
  /** IPFS CID or hash of the off-chain verification document. */
  ipfsDocHash: string;
}

/** Compliance profile for an individual investor/holder. */
export interface HolderProfile {
  /** The holder's Stellar address. */
  address: string;
  /** The jurisdiction the holder is classified under (e.g., "US"). */
  jurisdiction: string;
  /** Whether the holder has passed KYC verification. */
  kycVerified: boolean;
  /** Whether the holder is whitelisted to participate in transfers. */
  whitelisted: boolean;
  /** Whether the holder has been frozen. */
  frozen: boolean;
}

/** Transfer rules for a given jurisdiction. */
export interface ComplianceRule {
  /** The jurisdiction this rule governs. */
  jurisdiction: string;
  /** Max single-transfer amount in USDC stroops; 0 = no limit. */
  maxTransferAmount: bigint;
  /** Whether KYC is required for transfers into this jurisdiction. */
  requiresKyc: boolean;
  /** Whether this rule is currently active. */
  enabled: boolean;
}

/** A single yield distribution round for one asset. */
export interface DistributionRound {
  /** The asset this round distributes yield for. */
  assetId: string;
  /** Total USDC deposited for this round (in stroops). */
  totalUsdc: bigint;
  /** Total token supply at the time of distribution. */
  totalTokenSupply: bigint;
  /** Ledger timestamp when the round was created. */
  distributedAt: bigint;
  /** Number of holders that have claimed so far. */
  claimedCount: number;
  /** Current status of this round. */
  status: RoundStatus;
}

/** A single oracle update pushed by an authorized oracle node. */
export interface OracleUpdate {
  /** The asset this update refers to. */
  assetId: string;
  /** Current appraised value in USDC stroops. */
  verifiedValue: bigint;
  /** Debtor credit score on a 0-1000 scale. */
  debtorCreditScore: number;
  /** Health flag: "HEALTHY" | "AT_RISK" | "DEFAULT_IMMINENT". */
  statusFlag: string;
  /** Ledger timestamp when the oracle generated this update. */
  updateTimestamp: bigint;
  /** Identifier of the submitting oracle node. */
  oracleId: string;
}

/** Network configuration for connecting to Stellar. */
export interface NetworkConfig {
  /** Deployed contract address for rwa_registry. */
  registryContractId: string;
  /** Deployed contract address for compliance_engine. */
  complianceContractId: string;
  /** Deployed contract address for yield_distributor. */
  yieldContractId: string;
  /** Deployed contract address for oracle_receiver. */
  oracleContractId: string;
  /** Stellar network passphrase. */
  networkPassphrase: string;
  /** Soroban RPC URL. */
  rpcUrl: string;
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/** Structured error thrown by all VerifiRWA SDK clients. */
export class VerifiRwaError extends Error {
  /** Short machine-readable error code matching the Soroban panic message. */
  public readonly code: string;
  /** Optional additional context from the RPC response. */
  public readonly details?: unknown;

  constructor(code: string, message: string, details?: unknown) {
    super(message);
    this.name = "VerifiRwaError";
    this.code = code;
    this.details = details;
  }
}
