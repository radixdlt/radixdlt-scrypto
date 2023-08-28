use radix_engine::{
    errors::{RuntimeError, SystemModuleError},
    system::system_modules::limits::TransactionLimitsError,
    transaction::{CostingParameters, ExecutionConfig},
    types::*,
};
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_wasm_buffers_read() {
    let (code, definition) = Compile::compile("tests/blueprints/wasm_buffers");
    let code_len = code.len();
    let definition_len = scrypto_encode(&definition).unwrap().len();

    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address =
        test_runner.publish_package(code, definition, BTreeMap::new(), OwnerRole::None);
    let component_address = test_runner
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(package_address, "WasmBuffersTest", "new", manifest_args!())
                .build(),
            vec![],
        )
        .expect_commit_success()
        .new_component_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            component_address,
            "read_memory",
            manifest_args!(0usize, 1024usize),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}
