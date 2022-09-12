use radix_engine::engine::{KernelError, ModuleError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn local_component_should_return_correct_info() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/local_component");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package_address,
            "Secret",
            "check_info_of_local_component",
            args!(package_address, "Secret".to_string()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn local_component_should_be_callable_read_only() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/local_component");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Secret", "read_local_component", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn local_component_should_be_callable_with_write() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/local_component");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(package_address, "Secret", "write_local_component", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn local_component_with_access_rules_should_not_be_callable() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/local_component");
    let (public_key, _, account) = test_runner.new_account();
    let auth_resource_address = test_runner.create_non_fungible_resource(account);
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth_resource_address, auth_id);

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package_address,
            "Secret",
            "try_to_read_local_component_with_auth",
            args!(auth_address),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::AuthorizationError { .. })
        )
    });
}

#[test]
fn local_component_with_access_rules_should_be_callable() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.compile_and_publish("./tests/local_component");
    let (public_key, _, account) = test_runner.new_account();
    let auth_resource_address = test_runner.create_non_fungible_resource(account);
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth_resource_address, auth_id.clone());

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_method(
            account,
            "create_proof_by_ids",
            args!(BTreeSet::from([auth_id.clone()]), auth_resource_address),
        )
        .call_scrypto_function(
            package_address,
            "Secret",
            "try_to_read_local_component_with_auth",
            args!(auth_address),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn recursion_bomb() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/local_recursion");

    // Act
    // Note: currently SEGFAULT occurs if bucket with too much in it is sent. My guess the issue is a native stack overflow.
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_amount(Decimal::from(10), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_scrypto_function(
                package_address,
                "LocalRecursionBomb",
                "recursion_bomb",
                args!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn recursion_bomb_to_failure() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/local_recursion");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_amount(Decimal::from(100), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_scrypto_function(
                package_address,
                "LocalRecursionBomb",
                "recursion_bomb",
                args!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::MaxCallDepthLimitReached)
        )
    });
}

#[test]
fn recursion_bomb_2() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/local_recursion");

    // Act
    // Note: currently SEGFAULT occurs if bucket with too much in it is sent. My guess the issue is a native stack overflow.
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_amount(Decimal::from(10), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_scrypto_function(
                package_address,
                "LocalRecursionBomb2",
                "recursion_bomb",
                args!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn recursion_bomb_2_to_failure() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/local_recursion");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::local_simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account_by_amount(Decimal::from(100), RADIX_TOKEN, account)
        .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
            builder.call_scrypto_function(
                package_address,
                "LocalRecursionBomb2",
                "recursion_bomb",
                args!(scrypto::resource::Bucket(bucket_id)),
            )
        })
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key.into()]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::MaxCallDepthLimitReached)
        )
    });
}
