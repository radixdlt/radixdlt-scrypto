use radix_engine::ledger::TypedInMemorySubstateStore;
use radix_engine::types::*;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn create_non_fungible_mutable() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package = test_runner.compile_and_publish("./tests/non_fungible");

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            args!(),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_burn_non_fungible() {
    // Arrange
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package = test_runner.compile_and_publish("./tests/non_fungible");
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package,
            "NonFungibleTest",
            "create_burnable_non_fungible",
            args!(),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let resource_address = receipt
        .expect_commit()
        .entity_changes
        .new_resource_addresses[0];
    let non_fungible_address =
        NonFungibleAddress::new(resource_address, NonFungibleId::from_u32(0));
    let mut ids = BTreeSet::new();
    ids.insert(NonFungibleId::from_u32(0));

    // Act
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account(resource_address, account)
        .burn_non_fungible(non_fungible_address.clone())
        .call_scrypto_function(
            package,
            "NonFungibleTest",
            "verify_does_not_exist",
            args!(non_fungible_address),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_fungible() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/non_fungible");

    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            args!(),
        )
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            args!(),
        )
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "non_fungible_exists",
            args!(),
        )
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_bucket",
            args!(),
        )
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_vault",
            args!(),
        )
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_ids_bucket",
            args!(),
        )
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_ids_vault",
            args!(),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_singleton_non_fungible() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/non_fungible");

    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "singleton_non_fungible",
            args!(),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

// This test was introduced in Oct 2022 to protect a regression whereby resources locked
// by a proof in a vault was accidentally committed/persisted, and locked in future transactions
#[test]
fn test_mint_update_and_withdraw() {
    let mut store = TypedInMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.compile_and_publish("./tests/non_fungible");

    // create non-fungible
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_scrypto_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            args!(),
        )
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
    let badge_resource_address = receipt
        .expect_commit()
        .entity_changes
        .new_resource_addresses[0];
    let nft_resource_address = receipt
        .expect_commit()
        .entity_changes
        .new_resource_addresses[1];

    // update data (the NFT is referenced within a Proof)
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .call_function_with_abi(
            package_address,
            "NonFungibleTest",
            "update_nft",
            vec![
                format!("1,{}", badge_resource_address),
                format!("1,{}", nft_resource_address),
            ],
            Some(account),
            &test_runner.export_abi(package_address, "NonFungibleTest"),
        )
        .unwrap()
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // transfer
    let manifest = ManifestBuilder::new(&NetworkDefinition::simulator())
        .lock_fee(10.into(), SYS_FAUCET_COMPONENT)
        .withdraw_from_account(nft_resource_address, account)
        .call_method(
            account,
            "deposit_batch",
            args!(Expression::entire_worktop()),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleAddress::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}
