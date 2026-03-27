use crate::{PredictionMarket, PredictionMarketClient, MarketStatus, Market, Role, access::AccessRole};
use soroban_sdk::{testutils::{Address as _, Ledger}, vec, Env, String, Address};

fn setup<'a>(env: &Env) -> (PredictionMarketClient<'a>, Address, Address) {
    let contract_id = env.register_contract(None, PredictionMarket);
    let client = PredictionMarketClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    client.initialize(&admin);
    client.assign_role(&admin, &Role::Oracle, &oracle);
    (client, admin, oracle)
}

#[test]
fn test_lock_market_and_propose_outcome() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, oracle) = setup(&env);
    
    // Deploy market
    let creator = Address::generate(&env);
    client.assign_role(&admin, &Role::Pauser, &creator);
    
    let token = Address::generate(&env);
    client.assign_role(&admin, &Role::FeeSetter, &admin);
    client.set_token_whitelist(&admin, &token, &true);
    
    // Create Market
    client.create_market(
        &creator,
        &1u64,
        &String::from_str(&env, "Test"),
        &vec![&env, String::from_str(&env, "A"), String::from_str(&env, "B")],
        &(env.ledger().timestamp() + 1000),
        &token,
        &1000i128,
        &None,
        &None,
    );
    
    // Oracle locks market
    client.lock_market(&oracle, &1u64);
    
    // Oracle proposes outcome
    client.propose_outcome(&oracle, &1u64, &0u32);
    
    // Time travel past challenge timer
    env.ledger().with_mut(|l| l.timestamp += 86_500);
    
    // Settle market
    client.settle(&oracle, &1u64);
}

#[test]
#[should_panic(expected = "Challenge timer has not expired")]
fn test_settle_fails_before_timer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, oracle) = setup(&env);
    
    let creator = Address::generate(&env);
    client.assign_role(&admin, &Role::Pauser, &creator);
    let token = Address::generate(&env);
    client.assign_role(&admin, &Role::FeeSetter, &admin);
    client.set_token_whitelist(&admin, &token, &true);
    
    client.create_market(
        &creator,
        &2u64,
        &String::from_str(&env, "Test2"),
        &vec![&env, String::from_str(&env, "A"), String::from_str(&env, "B")],
        &(env.ledger().timestamp() + 1000),
        &token,
        &1000i128,
        &None,
        &None,
    );
    
    client.lock_market(&oracle, &2u64);
    client.propose_outcome(&oracle, &2u64, &0u32);
    
    // Fast forward by 10 hours (36_000s) - not enough to settle
    env.ledger().with_mut(|l| l.timestamp += 36_000);
    client.settle(&oracle, &2u64);
}

#[test]
#[should_panic(expected = "AccessDenied: caller does not hold role Oracle")]
fn test_unauthorized_proposer() {
    let env = Env::default();
    env.mock_all_auths();
    let (client, admin, _oracle) = setup(&env);
    
    let creator = Address::generate(&env);
    client.assign_role(&admin, &Role::Pauser, &creator);
    let token = Address::generate(&env);
    client.assign_role(&admin, &Role::FeeSetter, &admin);
    client.set_token_whitelist(&admin, &token, &true);
    
    client.create_market(
        &creator,
        &3u64,
        &String::from_str(&env, "Test3"),
        &vec![&env, String::from_str(&env, "A"), String::from_str(&env, "B")],
        &(env.ledger().timestamp() + 1000),
        &token,
        &1000i128,
        &None,
        &None,
    );
    
    let hacker = Address::generate(&env);
    client.lock_market(&hacker, &3u64);
}
