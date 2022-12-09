use radix_engine::types::*;
use radix_engine_interface::args;
use radix_engine_interface::node::NetworkDefinition;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn can_set_component_metadata() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let package_address = test_runner.compile_and_publish("./tests/blueprints/metadata_component");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_function(package_address, "MetadataComponent", "new", args!())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
