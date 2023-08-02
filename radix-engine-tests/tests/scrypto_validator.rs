use radix_engine::types::*;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn can_call_accepts_delegated_stake_in_scrypto() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pub_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/validator");
    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ValidatorAccess",
                "accepts_delegated_stake",
                manifest_args!(validator_address),
            )
            .build(),
        vec![],
    );

    // Assert
    let result = receipt.expect_commit_success();
    let accepts_delegated_stake: bool = result.output(1);
    assert_eq!(accepts_delegated_stake, false);
}
