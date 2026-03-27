#[cfg(test)]
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String, Vec, vec};
use crate::{PredictionMarketClient, MarketStatus, Role, FeeMode, DataKey};

#[test]
fn test_propose_outcome_demo() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(crate::PredictionMarket, ());
    let client = PredictionMarketClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin);
    
    // Assign required roles to admin
    client.assign_role(&admin, &Role::FeeSetter, &admin);
    client.assign_role(&admin, &Role::Pauser, &admin);
    
    let fee_dest = Address::generate(&env);
    client.update_fee(&admin, &0i128, &fee_dest, &FeeMode::Treasury);

    let tokenadmin = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(tokenadmin);
    let token_addr = sac.address();
    
    client.set_token_whitelist(&admin, &token_addr, &true);
    client.update_bet_limits(&admin, &1i128, &0i128);

    let oracle = Address::generate(&env);
    client.assign_role(&admin, &Role::Oracle, &oracle);

    let creator = Address::generate(&env);
    let deadline = env.ledger().timestamp() + 86400;
    let options = vec![
        &env,
        String::from_str(&env, "Yes"),
        String::from_str(&env, "No"),
    ];
    
    // Create market with unique ID
    let market_id = 999u64;
    client.create_market(
        &creator,
        &market_id,
        &String::from_str(&env, "Test market"),
        &options,
        &deadline,
        &token_addr,
        &100_000_000i128,
        &None,
        &None,
    );

    // Check initial status
    let initial_status = client.get_market(&market_id).status;
    assert_eq!(initial_status, MarketStatus::Active);
    
    // Oracle locks market
    client.lock_market(&oracle, &market_id);
    let locked_status = client.get_market(&market_id).status;
    assert_eq!(locked_status, MarketStatus::Locked);
    
    // Oracle proposes outcome
    client.propose_outcome(&oracle, &market_id, &0u32);
    let market = client.get_market(&market_id);
    assert_eq!(market.status, MarketStatus::Proposed);
    assert_eq!(market.proposed_outcome, Some(0u32));
    
    // Check that unlock timestamp is set correctly
    let unlock_timestamp: u64 = env.storage().persistent().get(&DataKey::UnlockTimestamp(market_id)).unwrap();
    let current_time = env.ledger().timestamp();
    let time_diff = unlock_timestamp - current_time;
    assert_eq!(time_diff, 86400); // 24 hours
    
    // Check that proposed outcome is stored
    let proposed_outcome: u32 = env.storage().persistent().get(&DataKey::ProposedOutcomeId(market_id)).unwrap();
    assert_eq!(proposed_outcome, 0u32);
    
    // Test that settlement works after timer expires
    env.ledger().with_mut(|l| l.timestamp += 86400 + 1); // Advance past timer
    client.settle(&oracle, &market_id);
    let settled_market = client.get_market(&market_id);
    assert_eq!(settled_market.status, MarketStatus::Resolved);
}
