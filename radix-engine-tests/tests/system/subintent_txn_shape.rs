use radix_common::prelude::{FromPublicKey, NonFungibleGlobalId};
use radix_engine::transaction::ExecutionConfig;
use radix_rust::btreeset;
use radix_transactions::builder::ManifestBuilder;
use radix_transactions::model::{ManifestIntent, TestTransaction};
use scrypto_test::ledger_simulator::LedgerSimulatorBuilder;

#[test]
fn simple_subintent_should_work() {
    test_subintent_txn_shape(vec![vec![1], vec![]]);
}

#[test]
fn multiple_flat_subintents_should_work() {
    test_subintent_txn_shape(vec![vec![1, 2, 3, 4], vec![], vec![], vec![], vec![]]);
}

#[test]
fn multiple_deep_subintents_should_work() {
    test_subintent_txn_shape(vec![vec![1], vec![2], vec![3], vec![4], vec![]]);
}

fn test_subintent_txn_shape(children: Vec<Vec<usize>>) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let mut intents = vec![];

    for (index, intent_children) in children.into_iter().enumerate() {
        let mut builder = ManifestBuilder::new_v2();
        if index == 0 {
            builder = builder.lock_standard_test_fee(account);
        }
        for (index, _) in intent_children.iter().enumerate() {
            builder = builder.yield_to_child(ManifestIntent(index as u32), ());
        }

        let manifest = builder.build();

        let signatures = btreeset![NonFungibleGlobalId::from_public_key(&public_key)];
        intents.push((
            manifest,
            ledger.next_transaction_nonce(),
            intent_children,
            signatures,
        ))
    }

    let receipt = ledger.execute_transaction(
        TestTransaction::new_v2_from_nonce(intents)
            .prepare()
            .expect("expected transaction to be preparable")
            .get_executable(),
        ExecutionConfig::for_test_transaction(),
    );

    // Assert
    receipt.expect_commit_success();
}
