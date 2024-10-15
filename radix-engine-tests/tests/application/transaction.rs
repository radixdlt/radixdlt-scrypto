use radix_common::prelude::*;
use radix_engine::blueprints::transaction_processor::*;
use radix_engine::errors::*;
use radix_engine::transaction::*;
use radix_engine_interface::blueprints::package::*;
use radix_engine_interface::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn test_manifest_with_non_existent_resource() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let non_existent_resource = resource_address(EntityType::GlobalFungibleResourceManager, 222);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .take_all_from_worktop(non_existent_resource, "non_existent")
        .try_deposit_or_abort(account, None, "non_existent")
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_rejection(|e| {
        matches!(
            e,
            RejectionReason::BootloadingError(BootloadingError::ReferencedNodeDoesNotExist(..))
        )
    });
}

#[test]
fn test_call_method_with_all_resources_doesnt_drop_auth_zone_proofs() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

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
    let receipt = ledger.execute_manifest(
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .create_proof_from_account_of_amount(account, XRD, dec!(1))
        .build();
    let receipt = ledger.execute_manifest(
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

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
        .build_no_validate();
    let receipt = ledger.execute_manifest(
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("proof"));

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
    let receipt = ledger.execute_manifest(
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
        .get_free_xrd_from_faucet()
        .get_free_xrd_from_faucet()
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
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
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let invalid_encoded_instructions = vec![0xde, 0xad, 0xbe, 0xef];
    let references = Default::default();
    let blobs: IndexMap<Hash, Vec<u8>> = Default::default();

    let executable = ExecutableTransaction::new_v1(
        invalid_encoded_instructions,
        Default::default(),
        references,
        blobs,
        ExecutionContext {
            unique_hash: Hash([0; 32]),
            intent_hash_nullifications: vec![],
            epoch_range: Default::default(),
            pre_allocated_addresses: Default::default(),
            payload_size: 4,
            num_of_signature_validations: 0,
            costing_parameters: Default::default(),
            disable_limits_and_costing_modules: false,
            proposer_timestamp_range: None,
        },
    );

    // Act
    let receipt = ledger.execute_transaction(
        executable,
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

#[test]
fn creating_proof_and_then_dropping_it_should_not_keep_bucket_locked() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(true);

    ManifestBuilder::new()
        .withdraw_from_account(account, XRD, 73)
        .take_from_worktop(XRD, 73, "XRD")
        .create_proof_from_bucket_of_amount("XRD", 73, "XRDProof")
        .drop_all_proofs()
        .try_deposit_or_abort(account, None, "XRD")
        .build(); // This asserts the manifest is valid
}

#[test]
fn creating_proof_and_then_dropping_it_should_not_keep_bucket_locked2() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(true);

    ManifestBuilder::new()
        .withdraw_from_account(account, XRD, 73)
        .take_from_worktop(XRD, 73, "XRD")
        .create_proof_from_bucket_of_amount("XRD", 73, "XRDProof")
        .drop_named_proofs()
        .try_deposit_or_abort(account, None, "XRD")
        .build(); // This asserts the manifest is valid
}

#[test]
fn test_create_proof_from_bucket_of_amount() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(true);

    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, XRD, 73)
        .take_from_worktop(XRD, 73, "XRD")
        .create_proof_from_bucket_of_amount("XRD", 73, "XRDProof")
        .drop_all_proofs()
        .try_deposit_or_abort(account, None, "XRD")
        .build();

    // Act
    let receipt = ledger.preview_manifest(
        manifest,
        Default::default(),
        Default::default(),
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    // Assert
    let execution_trace = receipt
        .expect_commit_success()
        .execution_trace
        .as_ref()
        .unwrap();

    assert_eq!(
        execution_trace
            .execution_traces
            .get(3)
            .unwrap()
            .output
            .proofs
            .values()
            .next()
            .unwrap()
            .clone(),
        ProofSnapshot::Fungible {
            resource_address: XRD,
            total_locked: 73.into()
        }
    );
}

#[test]
fn test_create_proof_from_bucket_of_non_fungibles() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(true);
    let nft = ledger.create_non_fungible_resource(account);

    let manifest = ManifestBuilder::new()
        .withdraw_from_account(account, nft, 3)
        .take_from_worktop(nft, 3, "NFT")
        .create_proof_from_bucket_of_non_fungibles(
            "NFT",
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ],
            "NFTProof",
        )
        .drop_all_proofs()
        .try_deposit_or_abort(account, None, "NFT")
        .build();

    // Act
    let receipt = ledger.preview_manifest(
        manifest,
        Default::default(),
        Default::default(),
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    // Assert
    let execution_trace = receipt
        .expect_commit_success()
        .execution_trace
        .as_ref()
        .unwrap();

    assert_eq!(
        execution_trace
            .execution_traces
            .get(3)
            .unwrap()
            .output
            .proofs
            .values()
            .next()
            .unwrap()
            .clone(),
        ProofSnapshot::NonFungible {
            resource_address: nft,
            total_locked: indexset![
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
                NonFungibleLocalId::integer(3),
            ]
        }
    );
}

#[test]
fn test_drop_auth_zone_regular_proofs() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(true);

    let manifest = ManifestBuilder::new()
        .create_proof_from_account_of_amount(account, XRD, 73)
        .drop_auth_zone_regular_proofs()
        .pop_from_auth_zone("proof")
        .build();

    // Act
    let receipt = ledger.preview_manifest(
        manifest,
        Default::default(),
        Default::default(),
        PreviewFlags {
            use_free_credit: true,
            assume_all_signature_proofs: true,
            skip_epoch_check: true,
            disable_auth: false,
        },
    );

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::TransactionProcessorError(
                TransactionProcessorError::AuthZoneIsEmpty
            ))
        )
    })
}
