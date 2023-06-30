use radix_engine::{
    errors::{CallFrameError, KernelError, RuntimeError},
    kernel::call_frame::{CloseSubstateError, MoveModuleError},
    types::*,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_create_global_node_with_local_ref() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reference");

    // Call function
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_function(
                package_address,
                "ReferenceTest",
                "create_global_node_with_local_ref",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::MoveModuleError(x),
        )) => {
            matches!(x, MoveModuleError::NonGlobalRefNotAllowed(_))
        }
        _ => false,
    });
}

#[test]
fn test_add_local_ref_to_stored_substate() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reference");

    // Instantiate component
    let component_address = {
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "ReferenceTest".into(),
                "new",
                manifest_args!(),
            )
            .build();

        let receipt = test_runner.execute_manifest_ignoring_fee(
            manifest,
            [NonFungibleGlobalId::from_public_key(&public_key)],
        );
        receipt.expect_commit_success();

        receipt.expect_commit(true).new_component_addresses()[0]
    };

    // Call method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 500u32.into())
            .call_method(
                component_address,
                "add_local_ref_to_stored_substate",
                manifest_args!(),
            )
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::CloseSubstateError(x),
        )) => {
            matches!(x, CloseSubstateError::NonGlobalRefNotAllowed(_))
        }
        _ => false,
    });
}
