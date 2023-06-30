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
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;
use utils::ContextualDisplay;

#[test]
fn test_manifest_with_non_existent_resource() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let non_existent_resource = resource_address(EntityType::GlobalFungibleResourceManager, 222);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .take_all_from_worktop(non_existent_resource, |builder, bucket_id| {
            builder.call_method(account, "try_deposit_or_abort", manifest_args!(bucket_id))
        })
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account(account, RADIX_TOKEN)
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .create_proof_from_auth_zone(RADIX_TOKEN, |builder, proof_id| {
            builder.push_to_auth_zone(proof_id)
        })
        .call_method(
            account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
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
fn test_transaction_can_end_with_proofs_remaining_in_auth_zone() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!("1"))
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!("1"))
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!("1"))
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!("1"))
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .add_instruction(InstructionV1::CallFunction {
            package_address: PACKAGE_PACKAGE.into(),
            blueprint_name: PACKAGE_BLUEPRINT.to_string(),
            function_name: PACKAGE_PUBLISH_WASM_ADVANCED_IDENT.to_string(),
            args: to_manifest_value_and_unwrap!(&PackagePublishWasmAdvancedManifestInput {
                code: ManifestBlobRef([0; 32]),
                setup: PackageDefinition {
                    blueprints: btreemap!(),
                },
                metadata: metadata_init!(),
                owner_role: OwnerRole::None,
                package_address: None,
            }),
        })
        .0
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/proof");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .create_proof_from_account_of_amount(account, RADIX_TOKEN, dec!("1"))
        .call_function(
            package_address,
            "Receiver",
            "assert_first_proof",
            manifest_args!(ManifestExpression::EntireAuthZone, dec!("1"), RADIX_TOKEN),
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
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
        .call_method(test_runner.faucet_component(), "free", manifest_args!())
        .call_method(
            account,
            "try_deposit_batch_or_abort",
            manifest_args!(ManifestExpression::EntireWorktop),
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
    receipt.expect_commit_failure();
}
