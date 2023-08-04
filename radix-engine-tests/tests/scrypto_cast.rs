use radix_engine::types::*;
use scrypto::prelude::ComponentCastError;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn should_be_able_to_get_address_of_an_address_reservation() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/cast");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "CastTest",
            "cast_to_validator",
            manifest_args!(FAUCET),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);


    // Assert
    let result = receipt.expect_commit_success();
    let output: Result<(), ComponentCastError> = result.output(1);
    assert!(matches!(output, Err(ComponentCastError::CannotCast { .. })));
}
