use radix_engine::errors::{KernelError, RuntimeError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::api::LockFlags;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use transaction::data::manifest_args;

fn test_event_store_locking_from_scrypto(lock_flags: LockFlags) -> TransactionReceipt {
    let mut test_runner = TestRunner::builder().without_trace().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/events");

    let receipt = {
        let args = manifest_args!(lock_flags.bits());
        let manifest = ManifestBuilder::new()
            .call_function(
                package_address,
                "EventStoreVisibility",
                "lock_event_store",
                args,
            )
            .build();
        test_runner.execute_manifest_ignoring_fee(manifest, vec![])
    };
    receipt
}

/// Tests that Scrypto code can't lock the event store. This is to ensure that Scrypto code can't
/// add arbitrary events by manually locking the substate and adding events into it.
#[test]
fn locking_event_store_mutably_from_scrypto_fails() {
    // Act
    let receipt = test_event_store_locking_from_scrypto(LockFlags::MUTABLE);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::KernelError(KernelError::InvalidSubstateAccess {
                node_id: RENodeId::EventStore,
                offset: SubstateOffset::EventStore(EventStoreOffset::EventStore),
                flags: LockFlags::MUTABLE,
                ..
            })
        )
    });
}

/// Tests that Scrypto code can't lock the event store immutably. This is to ensure that Scrypto
/// code can't see the events that happened in the transaction.
#[test]
fn locking_event_store_immutably_from_scrypto_fails() {
    // Act
    let receipt = test_event_store_locking_from_scrypto(LockFlags::read_only());

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::KernelError(KernelError::InvalidSubstateAccess {
                node_id: RENodeId::EventStore,
                offset: SubstateOffset::EventStore(EventStoreOffset::EventStore),
                ..
            })
        )
    });
}
