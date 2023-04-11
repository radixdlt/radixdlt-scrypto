use radix_engine::errors::{KernelError, ModuleError, RejectionError, RuntimeError};
use radix_engine::system::kernel_modules::auth::AuthError;
use radix_engine::types::*;
use scrypto_unit::*;
use std::ops::Sub;
use transaction::builder::ManifestBuilder;

#[test]
fn non_existing_vault_should_cause_error() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_allocated_account();

    let non_existing_address = local_address(EntityType::InternalFungibleVault, 5);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .recall(non_existing_address, Decimal::one())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_rejection(|e| {
        e.eq(&RejectionError::ErrorBeforeFeeLoanRepaid(
            RuntimeError::KernelError(KernelError::NodeNotFound(
                non_existing_address.as_node_id().clone(),
            )),
        ))
    });
}

#[test]
fn cannot_take_on_non_recallable_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_fungible_resource(10u32.into(), 0u8, account);
    let vaults = test_runner.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .recall(LocalAddress::new_unchecked(vault_id.into()), Decimal::one())
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthError(AuthError::Unauthorized { .. },))
        )
    });
}

#[test]
fn can_take_on_recallable_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let recallable_token = test_runner.create_recallable_token(account);
    let vaults = test_runner.get_component_vaults(account, recallable_token);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .recall(LocalAddress::new_unchecked(vault_id.into()), Decimal::one())
        .call_method(
            other_account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    let original_account_amount = test_runner
        .get_component_resources(account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    let mut expected_amount: Decimal = 5u32.into();
    expected_amount = expected_amount.sub(Decimal::one());
    assert_eq!(expected_amount, original_account_amount);

    let other_amount = test_runner
        .get_component_resources(other_account)
        .get(&recallable_token)
        .cloned()
        .unwrap();
    assert_eq!(other_amount, Decimal::one());
}
