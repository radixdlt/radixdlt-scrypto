use radix_engine::engine::ScryptoInterpreter;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::state_manager::StagedSubstateStoreManager;
use radix_engine::transaction::{
    execute_and_commit_transaction, ExecutionConfig, FeeReserveConfig, TransactionReceipt,
};
use radix_engine::types::*;
use radix_engine::wasm::WasmEngine;
use radix_engine_constants::DEFAULT_COST_UNIT_LIMIT;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn self_transfer_txn(account: ComponentAddress, amount: Decimal) -> TransactionManifest {
    ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(account, 10.into())
        .withdraw_from_account_by_amount(account, amount, RADIX_TOKEN)
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
    let mut test_runner0 = TestRunner::new(true);
    let (public_key, _, account) = test_runner0.new_allocated_account();
    let mut manifests = Vec::<(TransactionManifest, Vec<NonFungibleAddress>, u64)>::new();
    for i in 0u64..10u64 {
        let manifest = self_transfer_txn(account, Decimal::from(i));
        manifests.push((
            manifest,
            vec![NonFungibleAddress::from_public_key(&public_key)],
            i,
        ));
    }

    let mut test_runner1 = TestRunner::new(true);
    let _ = test_runner1.new_allocated_account();
    let mut test_runner2 = TestRunner::new(true);
    let _ = test_runner2.new_allocated_account();
    let mut test_runner3 = TestRunner::new(true);
    let _ = test_runner3.new_allocated_account();
    let mut test_runner4 = TestRunner::new(true);
    let _ = test_runner4.new_allocated_account();

    // Act

    // Test Runner 0: One by One
    for each in &manifests {
        let (mut manager, scrypto_interpreter) = test_runner0.new_staged_substate_store_manager();
        execute_and_commit_manifests(&mut manager, scrypto_interpreter, 0, vec![each.clone()]);
    }

    // Test Runner 1: Batch
    let (mut manager, scrypto_interpreter) = test_runner1.new_staged_substate_store_manager();
    execute_and_commit_manifests(&mut manager, scrypto_interpreter, 0, manifests.clone());

    // Test Runner 2: Multi-batch, Single-commit
    let (batch0, batch1) = manifests.split_at(5);
    let (mut manager, scrypto_interpreter) = test_runner2.new_staged_substate_store_manager();
    let node_id0 = manager.new_child_node(0);
    execute_and_commit_manifests(&mut manager, scrypto_interpreter, node_id0, batch0.to_vec());
    let node_id1 = manager.new_child_node(node_id0);
    execute_and_commit_manifests(&mut manager, scrypto_interpreter, node_id1, batch1.to_vec());
    manager.merge_to_parent(node_id1);

    // Test Runner 3: Multi-batch, Multi-commit
    let (batch0, batch1) = manifests.split_at(5);
    let (mut manager, scrypto_interpreter) = test_runner3.new_staged_substate_store_manager();
    let node_id0 = manager.new_child_node(0);
    execute_and_commit_manifests(&mut manager, scrypto_interpreter, node_id0, batch0.to_vec());
    let node_id1 = manager.new_child_node(node_id0);
    execute_and_commit_manifests(&mut manager, scrypto_interpreter, node_id1, batch1.to_vec());
    manager.merge_to_parent(node_id0);
    manager.merge_to_parent(node_id1);

    // Test Runner 3: Multi-batch, Fork, Single-commit
    let (batch0, batch1) = manifests.split_at(5);
    let (mut manager, scrypto_interpreter) = test_runner4.new_staged_substate_store_manager();
    let node_id0 = manager.new_child_node(0);
    execute_and_commit_manifests(&mut manager, scrypto_interpreter, node_id0, batch0.to_vec());
    let node_id1 = manager.new_child_node(node_id0);
    execute_and_commit_manifests(&mut manager, scrypto_interpreter, node_id1, batch1.to_vec());
    let fork_id = manager.new_child_node(node_id0);
    execute_and_commit_manifests(
        &mut manager,
        scrypto_interpreter,
        fork_id,
        manifests.clone(),
    );
    let fork_child_id = manager.new_child_node(fork_id);
    execute_and_commit_manifests(
        &mut manager,
        scrypto_interpreter,
        fork_child_id,
        manifests.clone(),
    );
    manager.merge_to_parent(node_id1);

    // Assert
    assert_eq!(test_runner0.substate_store(), test_runner1.substate_store());
    assert_eq!(test_runner1.substate_store(), test_runner2.substate_store());
    assert_eq!(test_runner2.substate_store(), test_runner3.substate_store());
    assert_eq!(test_runner3.substate_store(), test_runner4.substate_store());
}

pub fn execute_and_commit_manifests<W: WasmEngine>(
    manager: &mut StagedSubstateStoreManager<TypedInMemorySubstateStore>,
    scrypto_interpreter: &ScryptoInterpreter<W>,
    node_id: u64,
    manifests: Vec<(TransactionManifest, Vec<NonFungibleAddress>, u64)>,
) -> Vec<TransactionReceipt> {
    if node_id == 0 {
        let staged = manager.get_root_store();
        manifests
            .into_iter()
            .map(|(manifest, initial_proofs, nonce)| {
                let transaction = TestTransaction::new(manifest, nonce, DEFAULT_COST_UNIT_LIMIT);
                let executable = transaction.get_executable(initial_proofs);
                execute_and_commit_transaction(
                    staged,
                    scrypto_interpreter,
                    &FeeReserveConfig::default(),
                    &ExecutionConfig::default(),
                    &executable,
                )
            })
            .collect()
    } else {
        let mut staged = manager.get_output_store(node_id);
        manifests
            .into_iter()
            .map(|(manifest, initial_proofs, nonce)| {
                let transaction = TestTransaction::new(manifest, nonce, DEFAULT_COST_UNIT_LIMIT);
                let executable = transaction.get_executable(initial_proofs);
                execute_and_commit_transaction(
                    &mut staged,
                    scrypto_interpreter,
                    &FeeReserveConfig::default(),
                    &ExecutionConfig::default(),
                    &executable,
                )
            })
            .collect()
    }
}
