use radix_common::prelude::*;
use radix_engine::errors::{CallFrameError, KernelError, RuntimeError};
use radix_engine::kernel::call_frame::{MovePartitionError, PersistNodeError};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn stored_bucket_in_committed_component_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("stored_values"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "InvalidInitStoredBucket",
            "create",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::MovePartitionError(MovePartitionError::PersistNodeError(
                    PersistNodeError::CannotPersistPinnedNode(..)
                ))
            ))
        )
    });
}

#[test]
fn stored_bucket_in_owned_component_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("stored_values"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "InvalidStoredBucketInOwnedComponent",
            "create_bucket_in_owned_component",
            manifest_args!(),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::KernelError(KernelError::CallFrameError(
                CallFrameError::MovePartitionError(MovePartitionError::PersistNodeError(
                    PersistNodeError::CannotPersistPinnedNode(..)
                ))
            ))
        )
    });
}
