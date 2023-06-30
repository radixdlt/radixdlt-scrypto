use radix_engine::blueprints::resource::FungibleResourceManagerError;
use radix_engine::errors::ApplicationError;
use radix_engine::errors::RuntimeError;
use radix_engine::errors::SystemModuleError;
use radix_engine::system::system_modules::costing::CostingError;
use radix_engine::system::system_modules::costing::FeeReserveError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_worktop_resource_leak() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(account, RADIX_TOKEN, 1.into())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::DropNonEmptyBucket
            ))
        )
    });
}

#[test]
fn test_many_current_auth_zone_call() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut expressions = Vec::<ManifestExpression>::new();
    for _ in 0..5000 {
        expressions.push(ManifestExpression::EntireAuthZone);
    }
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .call_method(account, "no_such_method", manifest_args!(expressions))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                CostingError::FeeReserveError(FeeReserveError::LimitExceeded { .. })
            ))
        )
    });
}

#[test]
fn test_many_worktop_call() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut expressions = Vec::<ManifestExpression>::new();
    for _ in 0..5000 {
        expressions.push(ManifestExpression::EntireWorktop);
    }
    let manifest = ManifestBuilder::new()
        .lock_fee(account, 500u32.into())
        .call_method(account, "no_such_method", manifest_args!(expressions))
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::CostingError(
                CostingError::FeeReserveError(FeeReserveError::LimitExceeded { .. })
            ))
        )
    });
}
