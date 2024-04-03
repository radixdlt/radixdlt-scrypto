use radix_common::prelude::*;
use radix_substate_store_impls::state_tree_support::*;
use radix_transaction_scenarios::scenarios::*;
use scrypto_test::prelude::*;

#[test]
fn substate_store_matches_state_tree_after_each_scenario() {
    let db = StateTreeUpdatingDatabase::new(InMemorySubstateDatabase::standard());
    DefaultTransactionScenarioExecutor::new(db)
        .on_transaction_execution(|_, _, _, db| {
            let hashes_from_tree = db.list_substate_hashes();
            assert_eq!(
                hashes_from_tree.keys().cloned().collect::<HashSet<_>>(),
                db.list_partition_keys().collect::<HashSet<_>>(),
                "partitions captured in the state tree should match those in the substate store"
            );
            for (db_partition_key, by_db_sort_key) in hashes_from_tree {
                assert_eq!(
                    by_db_sort_key.into_iter().collect::<HashMap<_, _>>(),
                    db.list_entries(&db_partition_key)
                        .map(|(db_sort_key, substate_value)| (db_sort_key, hash(substate_value)))
                        .collect::<HashMap<_, _>>(),
                    "partition's {:?} substates in the state tree should match those in the substate store",
                    db_partition_key,
                )
            }
        })
        .execute()
        .expect("Must succeed!");
}
