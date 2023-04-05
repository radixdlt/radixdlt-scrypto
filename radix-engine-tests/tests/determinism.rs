use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::*;
use scrypto::resource::DIVISIBILITY_MAXIMUM;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

#[test]
fn test_simple_deterministic_execution() {
    // Arrange
    let mut test_runner0 = TestRunner::builder().with_state_hashing().build();
    let mut test_runner1 = TestRunner::builder().with_state_hashing().build();

    // Act
    let (public_key0, _, account0) = test_runner0.new_allocated_account();
    let (public_key1, _, account1) = test_runner1.new_allocated_account();

    // Assert
    assert_eq!(public_key0, public_key1);
    assert_eq!(account0, account1);
    assert_eq!(test_runner0.get_state_hash(), test_runner1.get_state_hash());
    assert_eq!(test_runner0.substate_db(), test_runner1.substate_db());
}

#[test]
fn same_executions_result_in_same_final_state_hash() {
    let state_hashes = (0..5)
        .map(|_| create_and_pass_multiple_proofs())
        .collect::<HashSet<Hash>>();
    assert_eq!(
        state_hashes.len(),
        1,
        "non-deterministic final state hash: {:?}",
        state_hashes
    );
}

/// Simulates a complete "test" which creates multiple proofs and passes them to a method.
/// Such operation is supposed to trigger non-determinism bugs in the engine.
/// Returns the root hash of the system's final state.
fn create_and_pass_multiple_proofs() -> Hash {
    // Arrange
    let mut test_runner = TestRunner::builder().with_state_hashing().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let resource_address =
        test_runner.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let mut builder = ManifestBuilder::new();
    builder.lock_fee(test_runner.faucet_component(), 10.into());
    let proof_ids = (0..20)
        .map(|_| {
            builder
                .create_proof_from_account_by_amount(account, resource_address, 1.into())
                .add_instruction(Instruction::PopFromAuthZone)
                .2
                .unwrap()
        })
        .collect::<Vec<ManifestProof>>();
    let manifest = builder
        .call_function(
            package_address,
            "VaultProof",
            "receive_proofs",
            manifest_args!(proof_ids),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();

    test_runner.get_state_hash()
}
