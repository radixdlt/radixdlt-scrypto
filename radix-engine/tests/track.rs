use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::NetworkDefinition;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn self_transfer_txn(account: ComponentAddress, amount: Decimal) -> TransactionManifest {
    ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), account)
        .withdraw_from_account_by_amount(amount, RADIX_TOKEN, account)
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build()
}

#[test]
fn batched_executions_should_result_in_the_same_result() {
    // Arrange
    // These runners should mirror each other
    let mut store0 = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner0 = TestRunner::new(true, &mut store0);
    let (public_key, _, account) = test_runner0.new_account();
    let mut manifests = Vec::<(TransactionManifest, Vec<PublicKey>)>::new();
    for amount in 0..10 {
        let manifest = self_transfer_txn(account, Decimal::from(amount));
        manifests.push((manifest, vec![public_key.into()]));
    }

    let mut store1 = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner1 = TestRunner::new(true, &mut store1);
    let _ = test_runner1.new_account();
    let mut store2 = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner2 = TestRunner::new(true, &mut store2);
    let _ = test_runner2.new_account();
    let mut store3 = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner3 = TestRunner::new(true, &mut store3);
    let _ = test_runner3.new_account();
    let mut store4 = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner4 = TestRunner::new(true, &mut store4);
    let _ = test_runner4.new_account();

    // Act

    // Test Runner 0: One by One
    for (manifest, signers) in &manifests {
        let receipt = test_runner0.execute_manifest(manifest.clone(), signers.clone());
        receipt.expect_commit_success();
    }

    // Test Runner 1: Batch
    test_runner1.execute_batch(manifests.clone());

    // Test Runner 2: Multi-batch, Single-commit
    let (batch0, batch1) = manifests.split_at(5);
    let node_id0 = test_runner2.create_child_node(0);
    test_runner2.execute_batch_on_node(node_id0, batch0.to_vec());
    let node_id1 = test_runner2.create_child_node(node_id0);
    test_runner2.execute_batch_on_node(node_id1, batch1.to_vec());
    test_runner2.merge_node(node_id1);

    // Test Runner 3: Multi-batch, Multi-commit
    let (batch0, batch1) = manifests.split_at(5);
    let node_id0 = test_runner3.create_child_node(0);
    test_runner3.execute_batch_on_node(node_id0, batch0.to_vec());
    let node_id1 = test_runner3.create_child_node(node_id0);
    test_runner3.execute_batch_on_node(node_id1, batch1.to_vec());
    test_runner3.merge_node(node_id0);
    test_runner3.merge_node(node_id1);

    // Test Runner 3: Multi-batch, Fork, Single-commit
    let (batch0, batch1) = manifests.split_at(5);
    let node_id0 = test_runner4.create_child_node(0);
    test_runner4.execute_batch_on_node(node_id0, batch0.to_vec());
    let node_id1 = test_runner4.create_child_node(node_id0);
    test_runner4.execute_batch_on_node(node_id1, batch1.to_vec());
    let fork_id = test_runner4.create_child_node(node_id0);
    test_runner4.execute_batch_on_node(fork_id, manifests.clone());
    let fork_child_id = test_runner4.create_child_node(fork_id);
    test_runner4.execute_batch_on_node(fork_child_id, manifests.clone());
    test_runner4.merge_node(node_id1);

    // Assert
    assert_eq!(store0, store1);
    assert_eq!(store1, store2);
    assert_eq!(store2, store3);
    assert_eq!(store3, store4);
}
