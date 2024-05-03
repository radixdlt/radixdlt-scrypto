use radix_common::prelude::*;
use radix_substate_store_impls::state_tree_support::*;
use radix_transaction_scenarios::executor::*;
use scrypto_test::prelude::*;

#[test]
fn substate_store_matches_state_tree_after_each_scenario() {
    let db = StateTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());
    let network_definition = NetworkDefinition::simulator();
    DefaultTransactionScenarioExecutor::new(db, &network_definition)
        .on_transaction_executed(|_, _, _, db| {
            db.validate_state_tree_matches_substate_store().unwrap()
        })
        .execute_every_protocol_update_and_scenario()
        .expect("Must succeed!");
}
