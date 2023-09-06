use radix_engine::blueprints::transaction_processor::*;
use radix_engine::errors::*;
use radix_engine::transaction::ExecutionConfig;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_manifest_with_non_existent_resource() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let non_existent_resource = resource_address(EntityType::GlobalFungibleResourceManager, 222);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .take_all_from_worktop(non_existent_resource, "non_existent")
        .try_deposit_or_abort(account, None, "non_existent")
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(RuntimeError::KernelError(
                KernelError::InvalidReference(..)
            ))
        )
    });
}

#[test]
fn test_call_method_with_all_resources_doesnt_drop_auth_zone_proofs() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_auth_zone_of_all(XRD, "proof1")
        .push_to_auth_zone("proof1")
        .try_deposit_entire_worktop_or_abort(account, None)
        .create_proof_from_auth_zone_of_all(XRD, "proof2")
        .push_to_auth_zone("proof2")
        .try_deposit_entire_worktop_or_abort(account, None)
        .create_proof_from_auth_zone_of_all(XRD, "proof3")
        .push_to_auth_zone("proof3")
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_transaction_can_end_with_proofs_remaining_in_auth_zone() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_existent_blob_hash() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500)
        .call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            PACKAGE_PUBLISH_WASM_ADVANCED_IDENT,
            PackagePublishWasmAdvancedManifestInput {
                code: ManifestBlobRef([0; 32]),
                definition: PackageDefinition {
                    blueprints: indexmap!(),
                },
                metadata: metadata_init!(),
                owner_role: OwnerRole::None,
                package_address: None,
            },
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::BlobNotFound(_)
            ))
        )
    });
}

#[test]
fn test_entire_auth_zone() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .call_function(
            package_address,
            "Receiver",
            "assert_first_proof",
            manifest_args!(ManifestExpression::EntireAuthZone, dec!(1), XRD),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_faucet_drain_attempt_should_fail() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .get_free_xrd_from_faucet()
        .get_free_xrd_from_faucet()
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    println!(
        "{}",
        receipt.display(&AddressBech32Encoder::for_simulator())
    );

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn transaction_processor_produces_expected_error_for_undecodable_instructions() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();

    let invalid_encoded_instructions = [0xde, 0xad, 0xbe, 0xef];
    let references = Default::default();
    let blobs = Default::default();

    let executable = Executable::new(
        &invalid_encoded_instructions,
        &references,
        &blobs,
        ExecutionContext {
            intent_hash: TransactionIntentHash::NotToCheck {
                intent_hash: Hash([0; 32]),
            },
            epoch_range: Default::default(),
            pre_allocated_addresses: Default::default(),
            payload_size: 4,
            num_of_signature_validations: 0,
            auth_zone_params: Default::default(),
            costing_parameters: Default::default(),
        },
    );

    // Act
    let receipt = test_runner.execute_transaction(
        executable,
        Default::default(),
        ExecutionConfig::for_notarized_transaction(NetworkDefinition::simulator()),
    );

    // Assert
    receipt.expect_specific_rejection(|error| {
        matches!(
            error,
            RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(RuntimeError::ApplicationError(
                ApplicationError::InputDecodeError(..)
            ))
        )
    })
}
