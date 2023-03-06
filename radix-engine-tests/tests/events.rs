#![allow(dead_code)]

use radix_engine::{
    errors::{ApplicationError, RuntimeError},
    system::kernel_modules::events::EventError,
    types::*,
};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

// TODO: In the future, the ClientAPI should only be able to add events to the event store. It
// should not be able to have full control over it.

#[test]
fn scrypto_cant_emit_unregistered_event() {
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
    receipt.expect_specific_failure(|runtime_error| {
        matches!(
            runtime_error,
            RuntimeError::ApplicationError(ApplicationError::EventError(
                EventError::SchemaNotFoundError { .. }
            ))
        )
    });
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
