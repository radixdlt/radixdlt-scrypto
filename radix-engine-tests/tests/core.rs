use radix_engine::{
    errors::{RuntimeError, SystemError},
    types::*,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_process_and_transaction() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest1 = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "RuntimeTest", "query", manifest_args![])
        .build();
    let receipt1 = test_runner.execute_manifest(manifest1, vec![]);
    receipt1.expect_commit_success();
}

#[test]
fn test_call() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MoveTest", "move_bucket", manifest_args![])
        .call_function(package_address, "MoveTest", "move_proof", manifest_args![])
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn cant_globalize_in_another_package() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address1 = test_runner.compile_and_publish("./tests/blueprints/core");
    let package_address2 = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address1,
            "GlobalizeTest",
            "globalize_in_package",
            manifest_args![package_address2],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidGlobalizeAccess(..))
        )
    });
}

#[test]
fn cant_drop_in_another_package() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address1 = test_runner.compile_and_publish("./tests/blueprints/core");
    let package_address2 = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address1,
            "DropTest",
            "drop_in_package",
            manifest_args![package_address2],
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidDropAccess(..))
        )
    });
}

fn call_function_and_assert_error(blueprint: &str, function: &str, expected_error: &str) {
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, blueprint, function, manifest_args![])
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| e.to_string().contains(expected_error));
}

#[test]
fn cant_globalize_bucket() {
    call_function_and_assert_error(
        "GlobalizeTest",
        "globalize_bucket",
        "InvalidGlobalizeAccess",
    )
}

#[test]
fn cant_globalize_proof() {
    call_function_and_assert_error("GlobalizeTest", "globalize_proof", "InvalidGlobalizeAccess")
}

#[test]
fn cant_globalize_vault() {
    call_function_and_assert_error("GlobalizeTest", "globalize_vault", "InvalidGlobalizeAccess")
}

#[test]
fn cant_globalize_metadata() {
    call_function_and_assert_error(
        "GlobalizeTest",
        "globalize_metadata",
        "InvalidGlobalizeAccess",
    )
}

#[test]
fn cant_globalize_royalty() {
    call_function_and_assert_error(
        "GlobalizeTest",
        "globalize_royalty",
        "InvalidGlobalizeAccess",
    )
}

#[test]
fn cant_globalize_role_assignment() {
    call_function_and_assert_error(
        "GlobalizeTest",
        "globalize_role_assignment",
        "InvalidGlobalizeAccess",
    )
}

#[test]
fn cant_store_bucket() {
    call_function_and_assert_error("GlobalizeTest", "store_bucket", "CannotPersistStickyNode")
}

#[test]
fn cant_store_proof() {
    call_function_and_assert_error("GlobalizeTest", "store_proof", "CannotPersistStickyNode")
}

#[test]
fn cant_store_metadata() {
    call_function_and_assert_error("GlobalizeTest", "store_metadata", "CannotPersistStickyNode")
}

#[test]
fn cant_store_royalty() {
    call_function_and_assert_error("GlobalizeTest", "store_royalty", "CannotPersistStickyNode")
}

#[test]
fn cant_store_role_assignment() {
    call_function_and_assert_error(
        "GlobalizeTest",
        "store_role_assignment",
        "CannotPersistStickyNode",
    )
}

#[test]
fn test_globalize_with_very_deep_own() {
    let mut test_runner = TestRunnerBuilder::new().without_trace().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/core");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RecursiveTest",
            "create_own_at_depth",
            manifest_args![10000u32],
        )
        .build();
    let result = test_runner.execute_manifest(manifest, vec![]);
    result.expect_commit_failure();
}
