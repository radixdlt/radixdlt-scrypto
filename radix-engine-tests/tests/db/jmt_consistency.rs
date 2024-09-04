use radix_common::prelude::*;
use radix_substate_store_impls::state_tree_support::*;
use radix_transaction_scenarios::executor::*;
use scrypto_test::prelude::*;

#[test]
fn substate_store_matches_state_tree_after_each_scenario() {
    let db = StateTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());
    struct Hooks;
    impl ScenarioExecutionHooks<StateTreeUpdatingDatabase<InMemorySubstateDatabase>> for Hooks {
        fn on_transaction_executed(
            &mut self,
            event: OnScenarioTransactionExecuted<
                StateTreeUpdatingDatabase<InMemorySubstateDatabase>,
            >,
        ) {
            let OnScenarioTransactionExecuted { database, .. } = event;
            database
                .validate_state_tree_matches_substate_store()
                .unwrap()
        }
    }
    TransactionScenarioExecutor::new(db, NetworkDefinition::simulator())
        .execute_every_protocol_update_and_scenario(&mut Hooks)
        .expect("Must succeed!");
}
