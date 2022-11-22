use radix_engine::engine::{ApplicationError, KernelError, RuntimeError};
use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::model::PackageError;
use radix_engine::types::*;
use radix_engine::wasm::*;
use radix_engine_interface::core::NetworkDefinition;
use radix_engine_interface::data::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

/*
#[test]
fn can_set_package_metadata() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let code = wat2wasm(include_str!("wasm/basic_package.wat"));
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .publish_package(code, HashMap::new())
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let package_address = receipt.expect_commit().entity_changes.new_package_addresses[0];

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(FAUCET_COMPONENT, 10.into())
        .call_native_method(
            RENodeId::Global(GlobalAddress::Package(package_address)),
            "set",
            scrypto_encode(&MetadataSetInvocation {
                receiver: RENodeId::Global(GlobalAddress::Package(package_address)),
                key: "name".to_string(),
                value: "best package ever!".to_string(),
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
 */
