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

#[test]
fn can_call_total_stake_xrd_amount_in_scrypto() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (pub_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/validator");
    let validator_address = test_runner.new_validator_with_pub_key(pub_key, account);
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                account,
                VALIDATOR_OWNER_BADGE,
                &btreeset!(NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()),
            )
            .withdraw_from_account(account, XRD, 10)
            .take_all_from_worktop(XRD, "xrd")
            .stake_validator_as_owner(validator_address, "xrd")
            .deposit_batch(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ValidatorAccess",
                "total_stake_xrd_amount",
                manifest_args!(validator_address),
            )
            .build(),
        vec![],
    );

    // Assert
    let result = receipt.expect_commit_success();
    let stake_amount: Decimal = result.output(1);
    assert_eq!(stake_amount, Decimal::from(10u32));
}
