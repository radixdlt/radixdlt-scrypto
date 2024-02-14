use radix_engine_common::prelude::*;
use scrypto_test::prelude::*;
use transaction_scenarios::scenario::{NextAction, ScenarioCore};
use transaction_scenarios::scenarios::get_builder_for_every_scenario;

#[test]
fn substate_store_matches_hash_tree_after_each_scenario() {
    let network = NetworkDefinition::simulator();
    let mut test_runner = TestRunnerBuilder::new().with_state_hashing().build();

    let mut next_nonce: u32 = 0;
    for scenario_builder in get_builder_for_every_scenario() {
        let epoch = test_runner.get_current_epoch();
        let mut scenario = scenario_builder(ScenarioCore::new(network.clone(), epoch, next_nonce));
        let mut previous = None;
        loop {
            let next = scenario
                .next(previous.as_ref())
                .map_err(|err| err.into_full(&scenario))
                .unwrap();
            match next {
                NextAction::Transaction(next) => {
                    let receipt = test_runner.execute_notarized_transaction(&next.raw_transaction);
                    // TODO(when figured out): We could run this assert as part of the existing
                    // `post_run_db_check` feature; however, it is tricky to implement (syntactically),
                    // due to the assert only being available for certain `TestDatabase` impl.
                    test_runner.assert_state_hash_tree_matches_substate_store();
                    previous = Some(receipt);
                }
                NextAction::Completed(end_state) => {
                    next_nonce = end_state.next_unused_nonce;
                    break;
                }
            }
        }
    }
}
