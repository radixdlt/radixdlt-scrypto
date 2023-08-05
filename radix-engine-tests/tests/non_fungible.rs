use radix_engine::blueprints::resource::NonFungibleResourceManagerError;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError};
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use radix_engine_interface::blueprints::transaction_processor::InstructionOutput;
use scrypto::NonFungibleData;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn can_mint_non_fungible_with_global() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleWithGlobalTest",
            "create_non_fungible_with_global",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_mutable() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_burn_non_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_burnable_non_fungible",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
    let vault_id = test_runner.get_component_vaults(account, resource_address)[0];
    let first_id = test_runner
        .inspect_non_fungible_vault(vault_id)
        .unwrap()
        .1
        .next();

    let non_fungible_global_id = NonFungibleGlobalId::new(resource_address, first_id.unwrap());

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, 1)
        .burn_non_fungible_from_worktop(non_fungible_global_id.clone())
        .call_function(
            package,
            "NonFungibleTest",
            "verify_does_not_exist",
            manifest_args!(non_fungible_global_id),
        )
        .try_deposit_batch_or_abort(account, None)
        .assert_worktop_contains(resource_address, 0)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_take_non_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_non_fungible_and_put_bucket",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_take_non_fungibles() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_non_fungibles_and_put_bucket",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_update_non_fungible_when_mutable() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible",
            manifest_args!("available".to_string(), true),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn cannot_update_non_fungible_when_not_mutable() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible",
            manifest_args!("tastes_great".to_string(), false),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::UnknownMutableFieldName(..)
            ))
        )
    });
}

#[test]
fn cannot_update_non_fungible_when_does_not_exist() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible",
            manifest_args!("does_not_exist".to_string(), false),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::UnknownMutableFieldName(..)
            ))
        )
    });
}

#[test]
fn can_call_non_fungible_data_reference() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    test_runner.set_metadata(
        account.into(),
        "test_key",
        "test_value",
        NonFungibleGlobalId::from_public_key(&public_key),
    );
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_reference",
            manifest_args!(account),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let resource_address = receipt.expect_commit_success().new_resource_addresses()[1];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "call_non_fungible_reference",
            manifest_args!(resource_address),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let result = receipt.expect_commit_success().outcome.expect_success();
    assert_eq!(
        result[1],
        InstructionOutput::CallReturn(scrypto_encode("test_value").unwrap())
    );
}

#[test]
fn cannot_have_non_fungible_data_ownership() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_non_fungible_with_ownership",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemError(SystemError::InvalidKeyValueStoreOwnership)
        )
    });
}

#[test]
fn can_update_and_get_non_fungible() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn can_update_and_get_non_fungible_reference() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible_reference",
            manifest_args!(account),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn can_check_if_contains_non_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "contains_non_fungible_vault",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();

    // Act
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_non_fungible_part_1() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "non_fungible_exists",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_bucket",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_non_fungible_part_2() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_vault",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_local_ids_bucket",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_local_ids_vault",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_singleton_non_fungible() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "singleton_non_fungible",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

// This test was introduced in Oct 2022 to protect a regression whereby resources locked
// by a proof in a vault was accidentally committed/persisted, and locked in future transactions
#[test]
fn test_mint_update_and_withdraw() {
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // create non-fungible
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    let badge_resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
    let nft_resource_address = receipt.expect_commit(true).new_resource_addresses()[1];

    // update data (the NFT is referenced within a Proof)
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, badge_resource_address, 1)
        .create_proof_from_account_of_non_fungibles(
            account,
            nft_resource_address,
            &btreeset!(NonFungibleLocalId::integer(0)),
        )
        .take_all_from_worktop(badge_resource_address, "badge")
        .pop_from_auth_zone("proof")
        .call_function_with_name_lookup(
            package_address,
            "NonFungibleTest",
            "update_nft",
            |lookup| (lookup.bucket("badge"), lookup.proof("proof")),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    let mut nfid_list = BTreeSet::new();
    nfid_list.insert(NonFungibleLocalId::integer(0)); // ID from NonFungibleTest::create_non_fungible_mutable

    // transfer
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, nft_resource_address, 1)
        .assert_worktop_contains_any(nft_resource_address)
        .assert_worktop_contains(nft_resource_address, 1)
        .assert_worktop_contains_non_fungibles(nft_resource_address, &nfid_list)
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn create_non_fungible_with_id_type_different_than_in_initial_supply() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_wrong_non_fungible_local_id_type",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_failure();
}

#[test]
fn create_bytes_non_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_bytes_non_fungible",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_string_non_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_string_non_fungible",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn create_ruid_non_fungible() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_ruid_non_fungible",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_get_total_supply() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "get_total_supply",
            manifest_args!(),
        )
        .build();

    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_mint_ruid_non_fungible_in_scrypto() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_ruid_non_fungible_and_mint",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[derive(ManifestSbor, ScryptoSbor, NonFungibleData)]
pub struct Sandwich {
    pub name: String,
    #[mutable]
    pub available: bool,
    pub tastes_great: bool,
    #[mutable]
    pub reference: Option<ComponentAddress>,
    #[mutable]
    pub own: Option<()>,
}

#[test]
fn can_mint_ruid_non_fungible_with_reference_in_manifest() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_mintable_ruid_non_fungible",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_ruid_non_fungible(
            resource_address,
            vec![Sandwich {
                name: "test".to_string(),
                available: false,
                tastes_great: true,
                reference: Some(account),
                own: None,
            }],
        )
        .assert_worktop_contains_any(resource_address)
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_mint_ruid_non_fungible_in_manifest() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_mintable_ruid_non_fungible",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_ruid_non_fungible(
            resource_address,
            vec![Sandwich {
                name: "test".to_string(),
                available: false,
                tastes_great: true,
                reference: None,
                own: None,
            }],
        )
        .assert_worktop_contains_any(resource_address)
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cant_burn_non_fungible_with_wrong_non_fungible_local_id_type() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let package = test_runner.compile_and_publish("./tests/blueprints/non_fungible");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "NonFungibleTest",
            "create_burnable_non_fungible",
            manifest_args!(),
        )
        .try_deposit_batch_or_abort(account, None)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    let resource_address = receipt.expect_commit(true).new_resource_addresses()[0];
    let non_fungible_global_id =
        NonFungibleGlobalId::new(resource_address, NonFungibleLocalId::ruid([0x11; 32]));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, resource_address, 1)
        .burn_non_fungible_from_worktop(non_fungible_global_id.clone())
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_failure();
}
