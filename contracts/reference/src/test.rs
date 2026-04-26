#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Address, Env, String, Symbol};

fn setup_test() -> (Env, ReferenceContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(ReferenceContract, ());
    let client = ReferenceContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    (env, client, admin)
}

// ── init / increment ──────────────────────────────────────────────────────────

#[test]
fn test_init_and_increment() {
    let (_env, client, admin) = setup_test();
    client.init(&admin);

    assert_eq!(client.increment(), 1);
    assert_eq!(client.increment(), 2);
    assert_eq!(client.increment(), 3);
}

#[test]
fn test_increment_without_init_starts_at_zero() {
    // Counter defaults to 0 via unwrap_or(0), so increment works without init.
    let (_env, client, _admin) = setup_test();
    assert_eq!(client.increment(), 1);
}

// ── process_data ──────────────────────────────────────────────────────────────

#[test]
fn test_process_data() {
    let (env, client, _admin) = setup_test();

    let keys = vec![&env, Symbol::new(&env, "A"), Symbol::new(&env, "B")];
    let result = client.process_data(&keys);

    assert_eq!(result.get(Symbol::new(&env, "A")).unwrap(), 0);
    assert_eq!(result.get(Symbol::new(&env, "B")).unwrap(), 1);
}

#[test]
fn test_process_data_empty_vec_returns_empty_map() {
    let (env, client, _admin) = setup_test();
    let result = client.process_data(&vec![&env]);
    assert_eq!(result.len(), 0);
}

#[test]
fn test_process_data_single_key() {
    let (env, client, _admin) = setup_test();
    let keys = vec![&env, Symbol::new(&env, "only")];
    let result = client.process_data(&keys);
    assert_eq!(result.get(Symbol::new(&env, "only")).unwrap(), 0);
}

// ── set_profile / get_profile ─────────────────────────────────────────────────

#[test]
fn test_set_and_get_profile() {
    let (env, client, admin) = setup_test();
    client.init(&admin);

    let user = Address::generate(&env);
    let profile = UserProfile {
        name: String::from_str(&env, "Alice"),
        age: 25,
        is_active: true,
        tags: vec![&env, Symbol::new(&env, "dev")],
    };

    client.set_profile(&user, &profile);

    let retrieved = client.get_profile(&user).unwrap();
    assert_eq!(retrieved.name, profile.name);
    assert_eq!(retrieved.age, profile.age);
    assert_eq!(retrieved.is_active, profile.is_active);
    assert_eq!(retrieved.tags.len(), 1);
}

#[test]
fn test_get_profile_unknown_address_returns_none() {
    let (env, client, _admin) = setup_test();
    let stranger = Address::generate(&env);
    assert!(client.get_profile(&stranger).is_none());
}

#[test]
fn test_set_profile_minimum_valid_age() {
    // age == 18 is the boundary — must succeed.
    let (env, client, admin) = setup_test();
    client.init(&admin);

    let user = Address::generate(&env);
    let profile = UserProfile {
        name: String::from_str(&env, "Min"),
        age: 18,
        is_active: false,
        tags: vec![&env],
    };

    client.set_profile(&user, &profile);
    assert_eq!(client.get_profile(&user).unwrap().age, 18);
}

#[test]
fn test_set_profile_overwrites_existing() {
    let (env, client, admin) = setup_test();
    client.init(&admin);

    let user = Address::generate(&env);
    let v1 = UserProfile {
        name: String::from_str(&env, "Old"),
        age: 20,
        is_active: true,
        tags: vec![&env],
    };
    let v2 = UserProfile {
        name: String::from_str(&env, "New"),
        age: 30,
        is_active: false,
        tags: vec![&env],
    };

    client.set_profile(&user, &v1);
    client.set_profile(&user, &v2);

    let retrieved = client.get_profile(&user).unwrap();
    assert_eq!(retrieved.name, String::from_str(&env, "New"));
    assert_eq!(retrieved.age, 30);
    assert!(!retrieved.is_active);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_set_profile_invalid_age() {
    let (env, client, admin) = setup_test();
    client.init(&admin);

    let user = Address::generate(&env);
    let profile = UserProfile {
        name: String::from_str(&env, "Bob"),
        age: 17,
        is_active: true,
        tags: vec![&env],
    };

    client.set_profile(&user, &profile);
}

#[test]
#[should_panic(expected = "HostError: Error(Contract, #3)")]
fn test_set_profile_age_zero_fails() {
    let (env, client, admin) = setup_test();
    client.init(&admin);

    let user = Address::generate(&env);
    let profile = UserProfile {
        name: String::from_str(&env, "Zero"),
        age: 0,
        is_active: true,
        tags: vec![&env],
    };

    client.set_profile(&user, &profile);
}

// ── fail_with_error ───────────────────────────────────────────────────────────

#[test]
#[should_panic(expected = "HostError: Error(Contract, #1)")]
fn test_fail_with_error() {
    let (_env, client, _admin) = setup_test();
    client.fail_with_error();
}
