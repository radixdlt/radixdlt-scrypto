use radix_common::prelude::*;
use radix_engine::blueprints::resource::{NonFungibleResourceManagerError, VaultError};
use radix_engine::errors::{ApplicationError, RuntimeError};
use scrypto::prelude::FromPublicKey;
use scrypto_test::prelude::*;

#[test]
fn cannot_burn_frozen_burn_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_token(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_burn(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .burn_in_account(account, token_address, 1)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn cannot_deposit_into_frozen_deposit_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_token(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_deposit(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, token_address, 1)
        .deposit_entire_worktop(account)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn cannot_withdraw_from_frozen_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_token(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, token_address, 1)
        .deposit_entire_worktop(account)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn can_recall_from_frozen_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_token(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let vault_address = InternalAddress::new_or_panic(vault_id.into());
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_withdraw(vault_address)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .recall(vault_address, 1)
        .deposit_entire_worktop(account)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn can_withdraw_from_unfrozen_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_token(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .unfreeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_from_account(account, token_address, 1)
        .deposit_entire_worktop(account)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn cannot_burn_frozen_burn_non_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_non_fungible(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_burn(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .burn_non_fungibles_in_account(account, token_address, [NonFungibleLocalId::integer(1)])
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn cannot_deposit_into_frozen_deposit_non_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_non_fungible(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_deposit(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(
            account,
            token_address,
            [NonFungibleLocalId::integer(1)],
        )
        .deposit_entire_worktop(account)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn cannot_withdraw_from_frozen_non_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let token_address = ledger.create_freezeable_non_fungible(account);
    let vaults = ledger.get_component_vaults(account, token_address);
    let vault_id = vaults[0];
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .withdraw_non_fungibles_from_account(
            account,
            token_address,
            [NonFungibleLocalId::integer(1)],
        )
        .deposit_entire_worktop(account)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::VaultError(VaultError::VaultIsFrozen))
        )
    });
}

#[test]
fn can_freezy_recall_unfreezy_non_fungible_vault() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (key, _priv, account) = ledger.new_account(true);
    let resource_address = ledger.create_freezeable_non_fungible(account);
    let vaults = ledger.get_component_vaults(account, resource_address);
    let vault_id = vaults[0];
    let internal_address = InternalAddress::new_or_panic(vault_id.into());

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .freeze_withdraw(internal_address)
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .assert_worktop_contains_non_fungibles(resource_address, [])
        .recall_non_fungibles(
            internal_address,
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
            ],
        )
        .assert_worktop_contains_non_fungibles(
            resource_address,
            [
                NonFungibleLocalId::integer(1),
                NonFungibleLocalId::integer(2),
            ],
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

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
        .lock_fee_from_faucet()
        .unfreeze_withdraw(internal_address)
        .build();
    let receipt =
        ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&key)]);

    // Assert
    receipt.expect_commit_success();
}
