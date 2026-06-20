/**
 * Utility helpers for working with VerifiRWA values.
 *
 * All on-chain monetary values are denominated in USDC stroops
 * (1 USDC = 10_000_000 stroops, 7 decimal places). These helpers
 * convert between stroops and human-readable USDC for display.
 */

const STROOPS_PER_USDC = 10_000_000n;

/**
 * Convert a USDC stroop amount to a human-readable USDC string.
 *
 * @param stroops - Amount in USDC stroops (7 decimal places).
 * @param decimals - Number of decimal places to show (default 2).
 * @returns Formatted string, e.g. "102000.00"
 *
 * @example
 * stroopsToUsdc(102_000_0000000n) // "102000.00"
 * stroopsToUsdc(1_5000000n, 4)    // "1.5000"
 */
export function stroopsToUsdc(stroops: bigint, decimals = 2): string {
  const whole = stroops / STROOPS_PER_USDC;
  const remainder = stroops % STROOPS_PER_USDC;
  const fractional = remainder.toString().padStart(7, "0").slice(0, decimals);
  return `${whole}.${fractional}`;
}

/**
 * Convert a human-readable USDC amount to stroops.
 *
 * Accepts integers or decimals up to 7 places.
 *
 * @param usdc - USDC amount as a string or number, e.g. "102000" or 1.5
 * @returns Stroop amount as bigint.
 *
 * @example
 * usdcToStroops("102000")  // 1020000000000n
 * usdcToStroops("1.5")     // 15000000n
 */
export function usdcToStroops(usdc: string | number): bigint {
  const str = String(usdc);
  const [whole, frac = ""] = str.split(".");
  const fracPadded = frac.slice(0, 7).padEnd(7, "0");
  return BigInt(whole) * STROOPS_PER_USDC + BigInt(fracPadded);
}

/**
 * Format a stroop amount as a display string with currency symbol.
 *
 * @param stroops - Amount in stroops.
 * @param symbol - Currency symbol to prefix (default "USDC").
 * @returns e.g. "USDC 102,000.00"
 *
 * @example
 * formatUsdc(102_000_0000000n) // "USDC 102,000.00"
 */
export function formatUsdc(stroops: bigint, symbol = "USDC"): string {
  const [whole, frac] = stroopsToUsdc(stroops, 2).split(".");
  const formatted = Number(whole).toLocaleString("en-US");
  return `${symbol} ${formatted}.${frac}`;
}

/**
 * Calculate the proportional yield claimable by a holder.
 *
 * Mirrors the on-chain formula: (holderBalance × totalUsdc) / totalSupply
 *
 * @param holderBalance - Holder's token balance in stroops.
 * @param totalUsdc - Total USDC in the distribution round (stroops).
 * @param totalSupply - Total token supply for the asset (stroops).
 * @returns Claimable USDC in stroops.
 * @throws If totalSupply is zero.
 *
 * @example
 * calculateClaimable(40_000_0000000n, 102_000_0000000n, 100_000_0000000n)
 * // 40_800_0000000n  (40% of 102,000 = 40,800)
 */
export function calculateClaimable(
  holderBalance: bigint,
  totalUsdc: bigint,
  totalSupply: bigint
): bigint {
  if (totalSupply === 0n) {
    throw new Error("totalSupply cannot be zero");
  }
  return (holderBalance * totalUsdc) / totalSupply;
}

/**
 * Return whether an asset maturity timestamp has passed.
 *
 * @param maturityTimestamp - Unix timestamp (seconds) from AssetMetadata.
 * @returns True if the asset has matured.
 */
export function isMatured(maturityTimestamp: bigint): boolean {
  return maturityTimestamp <= BigInt(Math.floor(Date.now() / 1000));
}

/**
 * Return the number of days remaining until an asset matures.
 *
 * Returns 0 if already matured.
 *
 * @param maturityTimestamp - Unix timestamp (seconds) from AssetMetadata.
 */
export function daysUntilMaturity(maturityTimestamp: bigint): number {
  const now = BigInt(Math.floor(Date.now() / 1000));
  if (maturityTimestamp <= now) return 0;
  const secondsLeft = maturityTimestamp - now;
  return Math.ceil(Number(secondsLeft) / 86_400);
}

/**
 * Stellar testnet network constants.
 */
export const TESTNET = {
  networkPassphrase: "Test SDF Network ; September 2015",
  rpcUrl: "https://soroban-testnet.stellar.org",
  usdcContractId: "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA",
} as const;

/**
 * Stellar mainnet network constants.
 */
export const MAINNET = {
  networkPassphrase: "Public Global Stellar Network ; September 2015",
  rpcUrl: "https://mainnet.sorobanrpc.com",
  usdcContractId: "CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75",
} as const;
