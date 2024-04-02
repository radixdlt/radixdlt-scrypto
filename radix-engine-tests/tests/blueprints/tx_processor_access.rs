use radix_common::prelude::*;
use radix_engine::errors::{RuntimeError, SystemModuleError};
use radix_engine::system::system_modules::auth::AuthError;
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use radix_engine_tests::common::*;
use scrypto::prelude::FromPublicKey;
use scrypto_test::prelude::*;

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let instructions: Vec<InstructionV1> = Vec::new();
    let manifest_encoded_instructions = manifest_encode(&instructions).unwrap();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            TRANSACTION_PROCESSOR_PACKAGE,
            TRANSACTION_PROCESSOR_BLUEPRINT,
            TRANSACTION_PROCESSOR_RUN_IDENT,
            ManifestTransactionProcessorRunInput {
                manifest_encoded_instructions,
                global_address_reservations: vec![],
                references: vec![],
                blobs: index_map_new(),
            },
        )
        .build();
    let result = ledger.execute_manifest(manifest, vec![]);

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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("tx_processor_access"));

    // Act
    let manifest_encoded_instructions: Vec<u8> = vec![0u8];
    let references: Vec<Reference> = vec![];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ExecuteManifest",
            "execute_manifest",
            manifest_args!(manifest_encoded_instructions, references),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_failure();
}

#[test]
fn should_not_be_able_to_steal_money_through_tx_processor_call() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account0) = ledger.new_account(true);
    let (_, _, account1) = ledger.new_account(true);
    let package_address = ledger.publish_package_simple(PackageLoader::get("tx_processor_access"));
    let initial_balance = ledger.get_component_balance(account0, XRD);
    let instructions = ManifestBuilder::new()
        .withdraw_from_account(account0, XRD, 10)
        .try_deposit_entire_worktop_or_abort(account1, None)
        .build()
        .instructions;
    let manifest_encoded_instructions = manifest_encode(&instructions).unwrap();
    let references: Vec<ComponentAddress> = vec![account0, account1];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "ExecuteManifest",
            "execute_manifest",
            manifest_args!(manifest_encoded_instructions, references),
        )
        .build();
    ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );

    // Assert
    let final_balance = ledger.get_component_balance(account0, XRD);
    assert_eq!(initial_balance, final_balance);
}
