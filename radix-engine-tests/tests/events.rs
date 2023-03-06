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
            "EventsBlueprint",
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

        let expected_events = vec![(
            EventTypeIdentifier(
                Emitter::Function(
                    RENodeId::GlobalPackage(package_address),
                    NodeModuleId::SELF,
                    "EventsBlueprint".into(),
                ),
                schema_hash::<CustomEvent>(),
            ),
            scrypto_encode(&CustomEvent { number: 12 }).unwrap(),
        )];

        assert_eq!(expected_events, application_events)
    }
}

#[derive(ScryptoEncode, Describe)]
struct CustomEvent {
    number: u64,
}

fn schema_hash<T: ScryptoDescribe>() -> Hash {
    let (local_type_index, schema) =
        generate_full_schema_from_single_type::<T, ScryptoCustomTypeExtension>();
    scrypto_encode(&(local_type_index, schema))
        .map(hash)
        .expect("Schema can't be encoded!")
}
