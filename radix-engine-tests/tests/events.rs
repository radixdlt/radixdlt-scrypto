use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

// TODO: In the future, the ClientAPI should only be able to add events to the event store. It
// should not be able to have full control over it.

#[test]
fn can_emit_basic_event_from_scrypto() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/events");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "EventStoreVisibility",
            "emit_event",
            manifest_args!(12u64),
        )
        .build();

    // Act
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    {
        receipt.expect_commit_success();
        let mut application_events = receipt.expect_commit().application_events.clone();
        application_events.remove(0); // Removing the first event which is the lock fee against the faucet.

        // FIXME fix test
    }
}

#[derive(ScryptoEncode)]
struct CustomEvent {
    number: u64,
}
