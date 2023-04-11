use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::system::kernel_modules::execution_trace::ResourceChange;
use radix_engine::types::*;
use radix_engine_interface::blueprints::account::{
    AccountSecurifyInput, ACCOUNT_DEPOSIT_BATCH_IDENT, ACCOUNT_SECURIFY_IDENT,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::model::Instruction;

#[test]
fn can_securify_virtual_account() {
    securify_account(true, true, true);
}

#[test]
fn cannot_securify_virtual_account_without_key() {
    securify_account(true, false, false);
}

#[test]
fn cannot_securify_allocated_account() {
    securify_account(false, true, false);
}

fn securify_account(is_virtual: bool, use_key: bool, expect_success: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _, account) = test_runner.new_account(is_virtual);

    let (_, _, storing_account) = test_runner.new_account(true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_method(
            account,
            ACCOUNT_SECURIFY_IDENT,
            to_manifest_value(&AccountSecurifyInput {}),
        )
        .call_method(
            storing_account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let initial_proofs = if use_key {
        vec![NonFungibleGlobalId::from_public_key(&key)]
    } else {
        vec![]
    };
    let receipt = test_runner.execute_manifest(manifest, initial_proofs);

    // Assert
    if expect_success {
        receipt.expect_commit_success();
    } else {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. }))
            )
        });
    }
}

#[test]
fn can_withdraw_from_my_allocated_account() {
    can_withdraw_from_my_account_internal(|test_runner| {
        let (public_key, _, account) = test_runner.new_account(false);
        (public_key, account)
    });
}

#[test]
fn can_withdraw_from_my_virtual_account() {
    can_withdraw_from_my_account_internal(|test_runner| {
        let (public_key, _, account) = test_runner.new_account(true);
        (public_key, account)
    });
}

fn can_withdraw_from_my_account_internal<F>(new_account: F)
where
    F: FnOnce(&mut TestRunner) -> (EcdsaSecp256k1PublicKey, ComponentAddress),
{
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, account) = new_account(&mut test_runner);
    let (_, _, other_account) = test_runner.new_account(true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10.into(), RADIX_TOKEN, 1.into())
        .call_method(
            other_account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let other_account_balance: Decimal = test_runner
        .account_balance(other_account, RADIX_TOKEN)
        .unwrap();
    let transfer_amount = other_account_balance - 10000 /* initial balance */;

    assert_resource_changes_for_transfer(
        &receipt
            .execution_trace
            .resource_changes
            .iter()
            .flat_map(|(_, rc)| rc)
            .cloned()
            .collect(),
        RADIX_TOKEN,
        other_account,
        transfer_amount,
    );
}

fn can_withdraw_non_fungible_from_my_account_internal(use_virtual: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_account(use_virtual);
    let (_, _, other_account) = test_runner.new_account(use_virtual);
    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_and_withdraw(account, 10.into(), resource_address, 1.into())
        .call_method(
            other_account,
            ACCOUNT_DEPOSIT_BATCH_IDENT,
            manifest_args!(ManifestExpression::EntireWorktop),
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
        .withdraw_from_account(other_account, RADIX_TOKEN, 1.into())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
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
        .lock_fee_and_withdraw(account, 10u32.into(), RADIX_TOKEN, 1.into())
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder
                .add_instruction(Instruction::CallMethod {
                    component_address: account,
                    method_name: "deposit".to_string(),
                    args: manifest_args!(bucket_id),
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
    let result = receipt.expect_commit_success();

    let vault_id = test_runner
        .get_component_vaults(account, RADIX_TOKEN)
        .first()
        .cloned()
        .unwrap();
    assert_eq!(
        receipt.execution_trace.resource_changes,
        indexmap!(
            0 => vec![ResourceChange {
                node_id: account.into(),
                vault_id,
                resource_address: RADIX_TOKEN,
                amount: - result.fee_summary.total_execution_cost_xrd - dec!("1")
            }],
            2 => vec![ResourceChange {
                node_id: account.into(),
                vault_id,
                resource_address: RADIX_TOKEN,
                amount: dec!("1")
            }],
        )
    );
}

#[test]
fn account_to_bucket_to_allocated_account() {
    account_to_bucket_to_account_internal(false);
}

#[test]
fn account_to_bucket_to_virtual_account() {
    account_to_bucket_to_account_internal(true);
}

fn assert_resource_changes_for_transfer(
    resource_changes: &Vec<ResourceChange>,
    resource_address: ResourceAddress,
    target_account: ComponentAddress,
    transfer_amount: Decimal,
) {
    println!("transfer: {:?}", transfer_amount);
    println!("{:?}", resource_changes);
    assert_eq!(2, resource_changes.len()); // Two transfers (withdraw + fee, deposit)
    assert!(resource_changes
        .iter()
        .any(|r| r.resource_address == resource_address
            && r.node_id == target_account.into()
            && r.amount == Decimal::from(transfer_amount)));
}
