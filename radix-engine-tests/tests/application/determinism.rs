use radix_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto::resource::DIVISIBILITY_MAXIMUM;
use scrypto_test::prelude::*;

#[test]
fn test_simple_deterministic_execution() {
    // Arrange
    let mut ledger0 = LedgerSimulatorBuilder::new().with_state_hashing().build();
    let mut ledger1 = LedgerSimulatorBuilder::new().with_state_hashing().build();

    // Act
    let (public_key0, _, account0) = ledger0.new_allocated_account();
    let (public_key1, _, account1) = ledger1.new_allocated_account();

    // Assert
    assert_eq!(public_key0, public_key1);
    assert_eq!(account0, account1);
    assert_eq!(ledger0.get_state_hash(), ledger1.get_state_hash());
    assert_eq!(ledger0.substate_db(), ledger1.substate_db());
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
    let mut ledger = LedgerSimulatorBuilder::new().with_state_hashing().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let resource_address =
        ledger.create_fungible_resource(100.into(), DIVISIBILITY_MAXIMUM, account);
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

    // Act
    let mut builder = ManifestBuilder::new();
    builder = builder.lock_fee_from_faucet();
    let mut proof_ids: Vec<_> = vec![];
    for _ in 0..20 {
        let proof_name = builder.generate_proof_name("proof");
        builder = builder
            .create_proof_from_account_of_amount(account, resource_address, 1)
            .pop_from_auth_zone(&proof_name);

        proof_ids.push(builder.proof(proof_name));
    }
    let manifest = builder
        .call_function(
            package_address,
            "VaultProof",
            "receive_proofs",
            manifest_args!(proof_ids),
        )
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();

    ledger.get_state_hash()
}
