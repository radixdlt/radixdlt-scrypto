use radix_common::prelude::*;
use radix_substate_store_impls::state_tree_support::*;
use radix_transaction_scenarios::scenarios::*;
use scrypto_test::prelude::*;

#[test]
fn substate_store_matches_state_tree_after_each_scenario() {
    let db = StateTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());
    DefaultTransactionScenarioExecutor::new(db, NetworkDefinition::simulator())
        .on_transaction_executed(|_, _, _, db| {
            db.validate_state_tree_matches_substate_store().unwrap()
        })
        .execute()
        .expect("Must succeed!");
}
