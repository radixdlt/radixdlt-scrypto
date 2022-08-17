use radix_engine::engine::ResourceChange;
use radix_engine::ledger::TypedInMemorySubstateStore;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::values::ScryptoValue;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

#[test]
fn can_withdraw_from_my_account() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let (_, _, other_account) = test_runner.new_account();

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), account)
        .withdraw_from_account(RADIX_TOKEN, account)
        .call_method_with_all_resources(other_account, "deposit_batch")
        .call_method(other_account, "balance", args!(RADIX_TOKEN))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    let outputs = receipt.expect_success();
    let other_account_balance: Decimal = scrypto_decode(&outputs[3]).unwrap();
    let transfer_amount = other_account_balance - 1_000_000 /* initial balance */;
    assert_resource_changes_for_transfer(
        &receipt.resource_changes,
        RADIX_TOKEN,
        account,
        other_account,
        transfer_amount,
    );
}

#[test]
fn can_withdraw_non_fungible_from_my_account() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
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
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
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
                    args: args!(scrypto::resource::Bucket(bucket_id)),
                })
                .0
        })
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    assert!(receipt.resource_changes.is_empty());
    receipt.expect_success();
}

#[test]
fn test_account_balance() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account1) = test_runner.new_account();
    let (_, _, account2) = test_runner.new_account();
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), account1)
        .call_method(account2, "balance", args!(RADIX_TOKEN))
        .build();

    // Act
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    let outputs = receipt.expect_success();

    // Assert
    assert!(receipt.resource_changes.is_empty());
    assert_eq!(
        outputs[1],
        ScryptoValue::from_typed(&Decimal::from(1000000)).raw
    );
}

fn assert_resource_changes_for_transfer(
    resource_changes: &Vec<ResourceChange>,
    resource_address: ResourceAddress,
    source_account: ComponentAddress,
    target_account: ComponentAddress,
    transfer_amount: Decimal,
) {
    assert_eq!(2, resource_changes.len());
    assert!(resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_address == source_account
            && r.amount == -transfer_amount));
    assert!(resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_address == target_account
            && r.amount == Decimal::from(transfer_amount)));
}
