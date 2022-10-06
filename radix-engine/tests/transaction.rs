use radix_engine::engine::KernelError;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto::core::Blob;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

#[test]
fn test_manifest_with_non_existent_resource() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let non_existent_resource = ResourceAddress::Normal([0u8; 26]);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), account)
        .take_from_worktop(non_existent_resource, |builder, bucket_id| {
            builder.call_method(
                account,
                "deposit",
                args!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::GlobalAddressNotFound(..))
        )
    });
}

#[test]
fn test_call_method_with_all_resources_doesnt_drop_auth_zone_proofs() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(dec!("10"), account)
        .create_proof_from_account(RADIX_TOKEN, account)
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.displayable(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_transaction_can_end_with_proofs_remaining_in_auth_zone() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(dec!("10"), account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.displayable(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_existent_blob_hash() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(dec!("10"), account)
        .add_instruction(Instruction::PublishPackage {
            code: Blob(Hash([0; 32])),
            abi: Blob(Hash([0; 32])),
        })
        .0
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.displayable(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::KernelError(KernelError::BlobNotFound(_)))
    });
}

#[test]
fn test_entire_auth_zone() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/proof");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(dec!("10"), account)
        .create_proof_from_account_by_amount(dec!("1"), RADIX_TOKEN, account)
        .call_scrypto_function(
            package_address,
            "Receiver",
            "assert_first_proof",
            args!(Expression::entire_auth_zone(), dec!("1"), RADIX_TOKEN),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.displayable(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_faucet_drain_attempt_should_fail() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(dec!("10"), account)
        .call_method(SYS_FAUCET_COMPONENT, "free", args!())
        .call_method(SYS_FAUCET_COMPONENT, "free", args!())
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    println!("{}", receipt.displayable(&Bech32Encoder::for_simulator()));

    // Assert
    receipt.expect_commit_failure();
}
