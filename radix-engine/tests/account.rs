#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::is_auth_error;
use crate::test_runner::TestRunner;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use scrypto::values::ScryptoValue;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), account)
        .withdraw_from_account(resource_address, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}

#[test]
fn cannot_withdraw_from_other_account() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, other_account)
        .call_method_with_all_resources(account, "deposit_batch")
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_failure(is_auth_error);
}

#[test]
fn account_to_bucket_to_account() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_address: account,
                    method_name: "deposit".to_string(),
                    arg: to_struct!(scrypto::resource::Bucket(bucket_id)),
                })
                .0
        })
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_account_balance() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), account1)
        .call_method(account2, "balance", to_struct!(RADIX_TOKEN))
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    let outputs = receipt.expect_success();

    // Assert
    assert_eq!(
        outputs[1],
        ScryptoValue::from_typed(&Decimal::from(1000000)).raw
    );
}
