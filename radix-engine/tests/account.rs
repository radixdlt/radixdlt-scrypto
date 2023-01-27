use radix_engine::model::ResourceChange;
use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::*;

fn can_withdraw_from_my_account_internal(use_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(use_virtual);
    let (_, _, other_account) = test_runner.new_account(use_virtual);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10.into(), RADIX_TOKEN)
        .call_method(
            other_account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .call_method(other_account, "balance", args!(RADIX_TOKEN))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let other_account_balance: Decimal = receipt.output(2);
    let transfer_amount = other_account_balance - 1000 /* initial balance */;
    let other_account_id: ComponentId = test_runner.deref_component(other_account).unwrap().into();

    assert_resource_changes_for_transfer(
        &receipt.expect_commit().resource_changes,
        RADIX_TOKEN,
        other_account_id,
        transfer_amount,
    );
}

#[test]
fn can_withdraw_from_my_allocated_account() {
    can_withdraw_from_my_account_internal(false);
}

#[test]
fn can_withdraw_from_my_virtual_account() {
    can_withdraw_from_my_account_internal(true);
}

fn can_withdraw_non_fungible_from_my_account_internal(use_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(use_virtual);
    let (_, _, other_account) = test_runner.new_account(use_virtual);
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10.into(), resource_address)
        .call_method(
            other_account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_withdraw_non_fungible_from_my_allocated_account() {
    can_withdraw_non_fungible_from_my_account_internal(false)
}

#[test]
fn can_withdraw_non_fungible_from_my_virtual_account() {
    can_withdraw_non_fungible_from_my_account_internal(true)
}

fn cannot_withdraw_from_other_account_internal(is_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(is_virtual);
    let (_, _, other_account) = test_runner.new_account(is_virtual);
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 10u32.into())
        .withdraw_from_account(other_account, RADIX_TOKEN)
        .call_method(
            account,
            "deposit_batch",
            args!(ManifestExpression::EntireWorktop),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error);
}

#[test]
fn cannot_withdraw_from_other_allocated_account() {
    cannot_withdraw_from_other_account_internal(false);
}

#[test]
fn cannot_withdraw_from_other_virtual_account() {
    cannot_withdraw_from_other_account_internal(true);
}

fn account_to_bucket_to_account_internal(use_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(use_virtual);
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10u32.into(), RADIX_TOKEN)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .add_instruction(BasicInstruction::CallMethod {
                    component_address: account,
                    method_name: "deposit".to_string(),
                    args: args!(bucket_id),
                })
                .0
        })
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
    assert_eq!(1, receipt.expect_commit().resource_changes.len()); // Just the fee payment
}

#[test]
fn account_to_bucket_to_allocated_account() {
    account_to_bucket_to_account_internal(false);
}

#[test]
fn account_to_bucket_to_virtual_account() {
    account_to_bucket_to_account_internal(true);
}

fn test_account_balance_internal(use_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account1) = test_runner.new_account(use_virtual);
    let (_, _, account2) = test_runner.new_account(use_virtual);
    let manifest = ManifestBuilder::new()
        .lock_fee(account1, 10.into())
        .call_method(account2, "balance", args!(RADIX_TOKEN))
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let outputs = receipt.expect_commit_success();

    // Assert
    assert_eq!(1, receipt.expect_commit().resource_changes.len()); // Just the fee payment
    assert_eq!(
        outputs[1].as_vec(),
        IndexedScryptoValue::from_typed(&Decimal::from(1000)).into_vec()
    );
}

#[test]
fn test_allocated_account_balance() {
    test_account_balance_internal(false)
}

#[test]
fn test_virtual_account_balance() {
    test_account_balance_internal(true)
}

fn assert_resource_changes_for_transfer(
    resource_changes: &Vec<ResourceChange>,
    resource_address: ResourceAddress,
    target_account: ComponentId,
    transfer_amount: Decimal,
) {
    println!("transfer: {:?}", transfer_amount);
    println!("{:?}", resource_changes);
    assert_eq!(2, resource_changes.len()); // Two transfers (withdraw + fee, deposit)
    assert!(resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.component_id == target_account
            && r.amount == Decimal::from(transfer_amount)));
}
