//! Full end-to-end lifecycle integration test for VerifiRWA.
//!
//! Tests the complete flow: invoice minted → investors acquire tokens →
//! oracle update → asset settled → all holders claim proportional USDC yield.
//! Also tests the DEFAULT_IMMINENT freeze scenario.

#![cfg(test)]

use compliance_engine::{
    ComplianceEngineContract, ComplianceEngineContractClient, ComplianceRule,
};
use oracle_receiver::{OracleReceiverContract, OracleReceiverContractClient, OracleUpdate};
use rwa_registry::{AssetMetadata, AssetStatus, RwaRegistryContract, RwaRegistryContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env, Symbol,
};
use yield_distributor::{RoundStatus, YieldDistributorContract, YieldDistributorContractClient};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct VerifiRwaEnv<'a> {
    env: Env,
    admin: Address,
    usdc: Address,
    registry: RwaRegistryContractClient<'a>,
    compliance: ComplianceEngineContractClient<'a>,
    yield_dist: YieldDistributorContractClient<'a>,
    oracle: OracleReceiverContractClient<'a>,
}

fn deploy_all() -> VerifiRwaEnv<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);

    // Deploy USDC Stellar Asset Contract
    let usdc_issuer = Address::generate(&env);
    let usdc = env
        .register_stellar_asset_contract_v2(usdc_issuer.clone())
        .address();

    // Deploy all four contracts
    let oracle_id = env.register_contract(None, OracleReceiverContract);
    let compliance_id = env.register_contract(None, ComplianceEngineContract);
    let yield_id = env.register_contract(None, YieldDistributorContract);
    let registry_id = env.register_contract(None, RwaRegistryContract);

    let oracle = OracleReceiverContractClient::new(&env, &oracle_id);
    let compliance = ComplianceEngineContractClient::new(&env, &compliance_id);
    let yield_dist = YieldDistributorContractClient::new(&env, &yield_id);
    let registry = RwaRegistryContractClient::new(&env, &registry_id);

    // Initialize in dependency order: oracle → compliance → yield → registry
    oracle.initialize(&admin, &registry_id, &86_400u64);
    compliance.initialize(&admin, &registry_id);
    yield_dist.initialize(&admin, &registry_id, &usdc);
    registry.initialize(&admin, &compliance_id, &yield_id, &oracle_id);

    VerifiRwaEnv {
        env,
        admin,
        usdc,
        registry,
        compliance,
        yield_dist,
        oracle,
    }
}

fn register_us_jurisdiction(t: &VerifiRwaEnv) {
    t.compliance.set_jurisdiction_rule(
        &t.admin,
        &ComplianceRule {
            jurisdiction: Symbol::new(&t.env, "US"),
            max_transfer_amount: 0,
            requires_kyc: false,
            enabled: true,
        },
    );
}

fn whitelist(t: &VerifiRwaEnv, holder: &Address) {
    t.compliance
        .register_holder(&t.admin, holder, &Symbol::new(&t.env, "US"), &true);
    t.compliance.whitelist_holder(&t.admin, holder);
}

// ---------------------------------------------------------------------------
// HAPPY PATH — Invoice Created, Sold, Settled, Yield Claimed
// ---------------------------------------------------------------------------

/// Full lifecycle: originator mints → investors buy → oracle confirms HEALTHY
/// → admin settles with 2% premium → all three parties claim correct USDC.
#[test]
fn test_happy_path_full_lifecycle() {
    let t = deploy_all();

    // -----------------------------------------------------------------------
    // Setup participants
    // -----------------------------------------------------------------------
    let originator = Address::generate(&t.env);
    let investor_a = Address::generate(&t.env);
    let investor_b = Address::generate(&t.env);

    // Admin adds originator
    t.registry.add_originator(&t.admin, &originator);
    register_us_jurisdiction(&t);

    // Step 1 & 2: register and whitelist originator, investors
    whitelist(&t, &originator);
    whitelist(&t, &investor_a);
    whitelist(&t, &investor_b);

    // -----------------------------------------------------------------------
    // Step 3: Originator mints INV-2024-001
    // face_value = 100_000 USDC = 100_000 * 10^7 = 1_000_000_000_000 stroops
    // -----------------------------------------------------------------------
    let face_value: i128 = 100_000_0000000; // 100,000 USDC in stroops (7 dec places)
    let now = t.env.ledger().timestamp();

    let meta = AssetMetadata {
        asset_id: Symbol::new(&t.env, "INV2024001"),
        face_value,
        maturity_timestamp: now + 86_400 * 30,
        originator: originator.clone(),
        debtor: Symbol::new(&t.env, "ACME_CORP"),
        asset_type: Symbol::new(&t.env, "INVOICE"),
        status: AssetStatus::Active,
        token_supply: face_value,
        created_at: 0,
        ipfs_doc_hash: Symbol::new(&t.env, "QmXyz123"),
    };

    let asset_id = t.registry.mint_asset(&originator, &meta);

    // Step 4: verify storage
    let stored = t.registry.get_asset(&asset_id);
    assert_eq!(stored.status, AssetStatus::Active);
    assert_eq!(stored.face_value, face_value);

    let orig_bal = t.registry.get_holder_balance(&asset_id, &originator);
    assert_eq!(orig_bal, face_value);

    // -----------------------------------------------------------------------
    // Steps 5–6: transfer tokens to investors
    // originator → investor_a: 40% (40_000 USDC tokens)
    // originator → investor_b: 30% (30_000 USDC tokens)
    // originator keeps: 30% (30_000 USDC tokens)
    // -----------------------------------------------------------------------
    let forty_pct: i128 = face_value * 40 / 100;
    let thirty_pct: i128 = face_value * 30 / 100;

    t.registry
        .transfer_tokens(&originator, &investor_a, &asset_id, &forty_pct);
    t.registry
        .transfer_tokens(&originator, &investor_b, &asset_id, &thirty_pct);

    // Step 7: verify balances
    let bal_a = t.registry.get_holder_balance(&asset_id, &investor_a);
    let bal_b = t.registry.get_holder_balance(&asset_id, &investor_b);
    let bal_orig = t.registry.get_holder_balance(&asset_id, &originator);

    assert_eq!(bal_a, forty_pct);
    assert_eq!(bal_b, thirty_pct);
    assert_eq!(bal_orig, face_value - forty_pct - thirty_pct);

    // -----------------------------------------------------------------------
    // Step 8: Oracle pushes HEALTHY update
    // -----------------------------------------------------------------------
    let oracle_node = Address::generate(&t.env);
    t.oracle.authorize_oracle(&t.admin, &oracle_node);

    let oracle_update = OracleUpdate {
        asset_id: asset_id.clone(),
        verified_value: face_value,
        debtor_credit_score: 820,
        status_flag: Symbol::new(&t.env, "HEALTHY"),
        update_timestamp: t.env.ledger().timestamp(),
        oracle_id: Symbol::new(&t.env, "oracle01"),
    };
    t.oracle.push_update(&oracle_node, &oracle_update);

    let latest = t.oracle.get_latest_update(&asset_id);
    assert_eq!(latest.debtor_credit_score, 820);

    // -----------------------------------------------------------------------
    // Step 9: Admin settles with 102% — fund yield_distributor first
    // settlement = 102_000 USDC = face_value * 1.02
    // -----------------------------------------------------------------------
    let settlement: i128 = face_value * 102 / 100;

    StellarAssetClient::new(&t.env, &t.usdc)
        .mint(&t.yield_dist.address, &settlement);

    t.registry
        .settle_asset(&t.admin, &asset_id, &settlement);

    // Step 10: verify distribution round
    let round = t.yield_dist.get_distribution_round(&asset_id);
    assert_eq!(round.status, RoundStatus::Active);
    assert_eq!(round.total_usdc, settlement);

    // -----------------------------------------------------------------------
    // Steps 11–13: All three holders claim yield
    // investor_a: 40% of 102_000 = 40_800
    // investor_b: 30% of 102_000 = 30_600
    // originator: 30% of 102_000 = 30_600
    // -----------------------------------------------------------------------
    let usdc_client = soroban_sdk::token::Client::new(&t.env, &t.usdc);

    let before_a = usdc_client.balance(&investor_a);
    t.yield_dist.claim_yield(&investor_a, &asset_id);
    let after_a = usdc_client.balance(&investor_a);
    let expected_a = bal_a
        .checked_mul(settlement)
        .unwrap()
        .checked_div(face_value)
        .unwrap();
    assert_eq!(after_a - before_a, expected_a);

    let before_b = usdc_client.balance(&investor_b);
    t.yield_dist.claim_yield(&investor_b, &asset_id);
    let after_b = usdc_client.balance(&investor_b);
    let expected_b = bal_b
        .checked_mul(settlement)
        .unwrap()
        .checked_div(face_value)
        .unwrap();
    assert_eq!(after_b - before_b, expected_b);

    let before_orig = usdc_client.balance(&originator);
    t.yield_dist.claim_yield(&originator, &asset_id);
    let after_orig = usdc_client.balance(&originator);
    let expected_orig = bal_orig
        .checked_mul(settlement)
        .unwrap()
        .checked_div(face_value)
        .unwrap();
    assert_eq!(after_orig - before_orig, expected_orig);

    // Step 14: investor_a tries to claim again — must panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        t.yield_dist.claim_yield(&investor_a, &asset_id);
    }));
    assert!(result.is_err(), "second claim should panic with already_claimed");
}

// ---------------------------------------------------------------------------
// FREEZE SCENARIO — DEFAULT_IMMINENT triggers automatic freeze
// ---------------------------------------------------------------------------

/// Oracle pushes DEFAULT_IMMINENT → registry auto-freezes → transfer blocked.
#[test]
fn test_freeze_on_default_imminent() {
    let t = deploy_all();

    let originator = Address::generate(&t.env);
    let investor = Address::generate(&t.env);

    t.registry.add_originator(&t.admin, &originator);
    register_us_jurisdiction(&t);
    whitelist(&t, &originator);
    whitelist(&t, &investor);

    let face_value: i128 = 10_000_0000000;
    let now = t.env.ledger().timestamp();

    let meta = AssetMetadata {
        asset_id: Symbol::new(&t.env, "INVDEFAULT"),
        face_value,
        maturity_timestamp: now + 86_400 * 30,
        originator: originator.clone(),
        debtor: Symbol::new(&t.env, "RISKY_CORP"),
        asset_type: Symbol::new(&t.env, "INVOICE"),
        status: AssetStatus::Active,
        token_supply: face_value,
        created_at: 0,
        ipfs_doc_hash: Symbol::new(&t.env, "QmRisky"),
    };

    let asset_id = t.registry.mint_asset(&originator, &meta);

    // Step 1: Oracle pushes DEFAULT_IMMINENT
    let oracle_node = Address::generate(&t.env);
    t.oracle.authorize_oracle(&t.admin, &oracle_node);

    let bad_update = OracleUpdate {
        asset_id: asset_id.clone(),
        verified_value: face_value / 2,
        debtor_credit_score: 100,
        status_flag: Symbol::new(&t.env, "DEFAULT_IMMINENT"),
        update_timestamp: t.env.ledger().timestamp(),
        oracle_id: Symbol::new(&t.env, "oracle01"),
    };
    t.oracle.push_update(&oracle_node, &bad_update);

    // Step 2: asset status should now be Frozen
    let stored = t.registry.get_asset(&asset_id);
    assert_eq!(stored.status, AssetStatus::Frozen, "asset should be frozen after DEFAULT_IMMINENT");

    // Step 3: attempt transfer — should be blocked
    let amount: i128 = 1_000_0000000;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        t.registry
            .transfer_tokens(&originator, &investor, &asset_id, &amount);
    }));
    assert!(result.is_err(), "transfer on frozen asset should be blocked");
}
