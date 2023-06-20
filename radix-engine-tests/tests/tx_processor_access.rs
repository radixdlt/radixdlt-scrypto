use radix_engine::blueprints::resource::VaultError;
use radix_engine::errors::{
    ApplicationError, CallFrameError, KernelError, RuntimeError, SystemError,
};
use radix_engine::kernel::call_frame::{CreateNodeError, TakeNodeError, UnlockSubstateError};
use radix_engine::types::*;
use scrypto::prelude::FromPublicKey;
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn calling_transaction_processor_from_scrypto_should_not_panic() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/tx_processor_access");

    // Act
    let manifest_encoded_instructions: Vec<u8> = vec![0u8];
    let references: Vec<Reference> = vec![];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
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
        .build().instructions;
    let manifest_encoded_instructions = manifest_encode(&instructions).unwrap();
    let references: Vec<ComponentAddress> = vec![account0, account1];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .call_function(
            package_address,
            "ExecuteManifest",
            "execute_manifest",
            manifest_args!(manifest_encoded_instructions, references),
        )
        .build();
    test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pub_key)]);

    // Assert
    let final_balance = test_runner.account_balance(account0, XRD).unwrap();
    assert_eq!(initial_balance, final_balance);
}