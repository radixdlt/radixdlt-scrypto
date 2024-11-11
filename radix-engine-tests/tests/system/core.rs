use radix_engine::errors::{CallFrameError, KernelError};
use radix_engine::errors::{RuntimeError, SystemError};
use radix_engine::kernel::call_frame::{
    CloseSubstateError, CreateNodeError, ProcessSubstateError, TakeNodeError,
};
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::FromPublicKey;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;
use scrypto_test::prelude::{OpenSubstateError, ProcessSubstateKeyError};

#[derive(ScryptoSbor, PartialEq, Eq, Debug)]
struct Compo {
    message: String,
}

#[test]
fn inspect_component_state() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "Compo", "new", manifest_args!())
            .build(),
        vec![],
    );

    // Assert
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];
    let state: Compo = ledger.component_state(component_address);
    assert_eq!(
        state,
        Compo {
            message: "Hi".to_owned()
        }
    );
}

#[test]
fn test_globalize_with_unflushed_invalid_own() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "GlobalizeUnflushed",
            "globalize_with_unflushed_invalid_own",
            manifest_args![],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_globalize_with_unflushed_kv_store_self_own() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "GlobalizeUnflushed",
            "globalize_with_unflushed_kv_store_self_own",
            manifest_args![],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_globalize_with_unflushed_another_transient_own() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "GlobalizeUnflushed",
            "globalize_with_unflushed_another_transient_own",
            manifest_args![],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CreateNodeError(CreateNodeError::ProcessSubstateError(
                    ProcessSubstateError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
                ))
            ))
        )
    });
}

#[test]
fn test_globalize_with_unflushed_another_own() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "GlobalizeUnflushed",
            "globalize_with_unflushed_another_own",
            manifest_args![],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CreateNodeError(CreateNodeError::ProcessSubstateError(
                    ProcessSubstateError::TakeNodeError(TakeNodeError::SubstateBorrowed(..))
                ))
            ))
        )
    });
}

#[test]
fn test_globalize_with_unflushed_another_own_v2() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "GlobalizeUnflushed",
            "globalize_with_unflushed_another_own_v2",
            manifest_args![],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::CloseSubstateError(CloseSubstateError::SubstateBorrowed(..))
            ))
        )
    });
}

#[test]
fn test_call() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "MoveTest", "move_bucket", manifest_args![])
        .call_function(package_address, "MoveTest", "move_proof", manifest_args![])
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn cant_globalize_in_another_package() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address1 = ledger.publish_package_simple(PackageLoader::get("core"));
    let package_address2 = ledger.publish_package_simple(PackageLoader::get("core"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address1,
            "GlobalizeTest",
            "globalize_in_package",
            manifest_args![package_address2],
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidGlobalizeAccess(..))
        )
    });
}

fn call_function_and_assert_error(blueprint: &str, function: &str, expected_error: &str) {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, blueprint, function, manifest_args![])
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_specific_failure(|e| e.to_string(NO_NETWORK).contains(expected_error));
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
    call_function_and_assert_error("GlobalizeTest", "store_bucket", "CannotPersistPinnedNode")
}

#[test]
fn cant_store_proof() {
    call_function_and_assert_error("GlobalizeTest", "store_proof", "CannotPersistPinnedNode")
}

#[test]
fn cant_store_metadata() {
    call_function_and_assert_error("GlobalizeTest", "store_metadata", "CannotPersistPinnedNode")
}

#[test]
fn cant_store_royalty() {
    call_function_and_assert_error("GlobalizeTest", "store_royalty", "CannotPersistPinnedNode")
}

#[test]
fn cant_store_role_assignment() {
    call_function_and_assert_error(
        "GlobalizeTest",
        "store_role_assignment",
        "CannotPersistPinnedNode",
    )
}

#[test]
fn test_globalize_with_very_deep_own() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "RecursiveTest",
            "create_own_at_depth",
            manifest_args![10000u32],
        )
        .build();
    let result = ledger.execute_manifest(manifest, vec![]);
    result.expect_commit_failure();
}

#[test]
fn test_insert_not_visible_global_refs_in_substate_key() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("core"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "SubstateKeyTest",
            "insert_not_visible_global_refs_in_substate_key",
            manifest_args![],
        )
        .try_deposit_entire_worktop_or_abort(account, None)
        .build();
    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::OpenSubstateError(OpenSubstateError::ProcessSubstateKeyError(
                    ProcessSubstateKeyError::NodeNotVisible(_)
                ))
            ))
        )
    })
}
