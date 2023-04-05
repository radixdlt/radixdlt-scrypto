use radix_engine::{
    errors::{CallFrameError, KernelError, RuntimeError},
    kernel::call_frame::UnlockSubstateError,
    types::*,
};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn verify_no_internal_ref_can_be_stored_in_track() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let package_address = test_runner.compile_and_publish("./tests/blueprints/reference");

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .call_function(package_address, "ReferenceTest", "new", manifest_args!())
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_specific_failure(|e| match e {
        RuntimeError::KernelError(KernelError::CallFrameError(
            CallFrameError::UnlockSubstateError(x),
        )) => {
            matches!(x, UnlockSubstateError::CantStoreLocalReference(_))
        }
        _ => false,
    });
}
