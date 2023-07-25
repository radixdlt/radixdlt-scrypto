use radix_engine::blueprints::resource::FungibleResourceManagerError;
use radix_engine::errors::ApplicationError;
use radix_engine::errors::RuntimeError;
use radix_engine::errors::SystemModuleError;
use radix_engine::system::system_modules::costing::CostingError;
use radix_engine::system::system_modules::costing::FeeReserveError;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_worktop_resource_leak() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, XRD, 1)
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
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut expressions = Vec::<ManifestExpression>::new();
    for _ in 0..20000 {
        expressions.push(ManifestExpression::EntireAuthZone);
    }
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
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
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Act
    let mut expressions = Vec::<ManifestExpression>::new();
    for _ in 0..5000 {
        expressions.push(ManifestExpression::EntireWorktop);
    }
    let manifest = ManifestBuilder::new()
        .lock_standard_test_fee(account)
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
