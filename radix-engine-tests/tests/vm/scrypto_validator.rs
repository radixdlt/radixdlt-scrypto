use radix_common::prelude::*;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

#[test]
fn can_call_accepts_delegated_stake_in_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("validator"));
    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);

    // Act
    let receipt = ledger.execute_manifest(
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
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("validator"));
    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                account,
                VALIDATOR_OWNER_BADGE,
                [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
            )
            .withdraw_from_account(account, XRD, 10)
            .take_all_from_worktop(XRD, "xrd")
            .stake_validator_as_owner(validator_address, "xrd")
            .deposit_entire_worktop(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
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

#[test]
fn can_call_total_stake_unit_supply_in_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("validator"));
    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                account,
                VALIDATOR_OWNER_BADGE,
                [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
            )
            .withdraw_from_account(account, XRD, 10)
            .take_all_from_worktop(XRD, "xrd")
            .stake_validator_as_owner(validator_address, "xrd")
            .deposit_entire_worktop(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ValidatorAccess",
                "total_stake_unit_supply",
                manifest_args!(validator_address),
            )
            .build(),
        vec![],
    );

    // Assert
    let result = receipt.expect_commit_success();
    let stake_unit_supply: Decimal = result.output(1);
    assert_eq!(stake_unit_supply, Decimal::from(10u32));
}

#[test]
fn can_call_validator_get_redemption_value_in_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (pub_key, _, account) = ledger.new_allocated_account();
    let package_address = ledger.publish_package_simple(PackageLoader::get("validator"));
    let validator_address = ledger.new_validator_with_pub_key(pub_key, account);
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungibles(
                account,
                VALIDATOR_OWNER_BADGE,
                [NonFungibleLocalId::bytes(validator_address.as_node_id().0).unwrap()],
            )
            .withdraw_from_account(account, XRD, 10)
            .take_all_from_worktop(XRD, "xrd")
            .stake_validator_as_owner(validator_address, "xrd")
            .deposit_entire_worktop(account)
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&pub_key)],
    );
    receipt.expect_commit_success();

    // Act
    let amount: Decimal = 5.into();
    let receipt = ledger.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                "ValidatorAccess",
                "get_redemption_value",
                manifest_args!(validator_address, amount),
            )
            .build(),
        vec![],
    );

    // Assert
    let result = receipt.expect_commit_success();
    let redemption_value: Decimal = result.output(1);
    assert_eq!(redemption_value, Decimal::from(5u32));
}
