use radix_engine::blueprints::transaction_processor::TransactionProcessorError;
use radix_engine::errors::ApplicationError;
use radix_engine::errors::KernelError;
use radix_engine::errors::RejectionError;
use radix_engine::errors::RuntimeError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_interface::metadata_init;
use radix_engine_queries::typed_substate_layout::PackagePublishWasmAdvancedManifestInput;
use radix_engine_queries::typed_substate_layout::PACKAGE_BLUEPRINT;
use radix_engine_queries::typed_substate_layout::PACKAGE_PUBLISH_WASM_ADVANCED_IDENT;
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
            RejectionError::ErrorBeforeFeeLoanRepaid(RuntimeError::KernelError(
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
        .try_deposit_batch_or_abort(account, None)
        .create_proof_from_auth_zone_of_all(XRD, "proof2")
        .push_to_auth_zone("proof2")
        .try_deposit_batch_or_abort(account, None)
        .create_proof_from_auth_zone_of_all(XRD, "proof3")
        .push_to_auth_zone("proof3")
        .try_deposit_batch_or_abort(account, None)
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
                    blueprints: btreemap!(),
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
        .try_deposit_batch_or_abort(account, None)
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
