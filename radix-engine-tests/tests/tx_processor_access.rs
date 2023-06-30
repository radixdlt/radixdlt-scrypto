use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use scrypto::prelude::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::InstructionV1;

#[derive(Debug, Eq, PartialEq, ManifestSbor)]
pub struct ManifestTransactionProcessorRunInput {
    pub manifest_encoded_instructions: Vec<u8>,
    pub global_address_reservations: Vec<()>,
    pub references: Vec<()>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
}

#[test]
fn should_not_be_able_to_call_tx_processor_in_tx_processor() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let instructions: Vec<InstructionV1> = Vec::new();
    let manifest_encoded_instructions = manifest_encode(&instructions).unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            TRANSACTION_PROCESSOR_PACKAGE,
            TRANSACTION_PROCESSOR_BLUEPRINT,
            TRANSACTION_PROCESSOR_RUN_IDENT,
            to_manifest_value_and_unwrap!(&ManifestTransactionProcessorRunInput {
                manifest_encoded_instructions,
                global_address_reservations: vec![],
                references: vec![],
                blobs: index_map_new(),
            }),
        )
        .build();
    let result = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    result.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn calling_transaction_processor_from_scrypto_should_not_panic() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/tx_processor_access");

    // Act
    let manifest_encoded_instructions: Vec<u8> = vec![0u8];
    let references: Vec<Reference> = vec![];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "ExecuteManifest",
            "execute_manifest",
            manifest_args!(manifest_encoded_instructions, references),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure();
}

#[test]
fn should_not_be_able_to_steal_money_through_tx_processor_call() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pub_key, _, account0) = test_runner.new_account(true);
    let (_, _, account1) = test_runner.new_account(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/tx_processor_access");
    let initial_balance = test_runner.account_balance(account0, XRD).unwrap();
    let instructions = ManifestBuilder::new()
        .withdraw_from_account(account0, XRD, 10.into())
        .try_deposit_batch_or_abort(account1)
        .build()
        .instructions;
    let manifest_encoded_instructions = manifest_encode(&instructions).unwrap();
    let references: Vec<ComponentAddress> = vec![account0, account1];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .call_function(
            package_address,
            "ExecuteManifest",
            "execute_manifest",
            manifest_args!(manifest_encoded_instructions, references),
        )
        .build();
    test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    let final_balance = test_runner.account_balance(account0, XRD).unwrap();
    assert_eq!(initial_balance, final_balance);
}
