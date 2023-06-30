use radix_engine::blueprints::resource::{NonFungibleResourceManagerError, VaultError};
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::types::*;
use scrypto::prelude::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_burn_frozen_burn_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _priv, account) = test_runner.new_account(true);
    let token_address = test_runner.create_freezeable_token(account);
    let vaults = test_runner.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .freeze_burn(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .burn_in_account(account, token_address, 1.into())
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn cannot_deposit_into_frozen_deposit_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _priv, account) = test_runner.new_account(true);
    let token_address = test_runner.create_freezeable_token(account);
    let vaults = test_runner.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .freeze_deposit(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(account, token_address, 1.into())
        .deposit_batch(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn cannot_withdraw_from_frozen_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _priv, account) = test_runner.new_account(true);
    let token_address = test_runner.create_freezeable_token(account);
    let vaults = test_runner.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .freeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(account, token_address, 1.into())
        .deposit_batch(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn cannot_recall_from_frozen_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _priv, account) = test_runner.new_account(true);
    let token_address = test_runner.create_freezeable_token(account);
    let vaults = test_runner.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let vault_address = InternalAddress::new_or_panic(vault_id.into());
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .freeze_withdraw(vault_address)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .recall(vault_address, 1.into())
        .deposit_batch(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_withdraw_from_unfrozen_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _priv, account) = test_runner.new_account(true);
    let token_address = test_runner.create_freezeable_token(account);
    let vaults = test_runner.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .freeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .unfreeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 500u32.into())
        .withdraw_from_account(account, token_address, 1.into())
        .deposit_batch(account)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_freezy_recall_unfreezy_non_fungible_vault() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (key, _priv, account) = test_runner.new_account(true);
    let resource_address = test_runner.create_freezeable_non_fungible(account);
    let vaults = test_runner.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];
    let internal_address = InternalAddress::new_or_panic(vault_id.into());
    let mut ids = BTreeSet::new();
    ids.insert(NonFungibleLocalId::integer(1));
    ids.insert(NonFungibleLocalId::integer(2));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50u32.into())
        .freeze_withdraw(internal_address)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50u32.into())
        .assert_worktop_contains_non_fungibles(resource_address, &BTreeSet::new())
        .recall_non_fungibles(internal_address, ids.clone())
        .assert_worktop_contains_non_fungibles(resource_address, &ids)
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::NonFungibleResourceManagerError(
                NonFungibleResourceManagerError::DropNonEmptyBucket
            ))
        )
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 50u32.into())
        .unfreeze_withdraw(internal_address)
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_commit_success();
}
