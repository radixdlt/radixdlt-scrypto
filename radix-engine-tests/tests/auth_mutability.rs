extern crate core;

use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::METADATA_SET_IDENT;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::resource::{FromPublicKey, FUNGIBLE_VAULT_BLUEPRINT, MethodKey, ObjectKey, require, RESOURCE_MANAGER_BURN_IDENT};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

enum ResourceAuth {
    Mint,
    Burn,
    Withdraw,
    Deposit,
    Recall,
    UpdateMetadata,
}

fn lock_resource_auth_and_try_update(action: ResourceAuth, lock: bool) -> TransactionReceipt {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (token_address, _, _, _, _, _, admin_auth) = test_runner.create_restricted_token(account);
    let (_, updated_auth) = test_runner.create_restricted_burn_token(account);
    {
        let function = match action {
            ResourceAuth::Mint => "lock_mintable",
            ResourceAuth::Burn => "lock_burnable",
            ResourceAuth::Withdraw => "lock_withdrawable",
            ResourceAuth::Deposit => "lock_depositable",
            ResourceAuth::Recall => "lock_recallable",
            ResourceAuth::UpdateMetadata => "lock_metadata_updateable",
        };
        test_runner.lock_resource_auth(function, admin_auth, token_address, account, public_key);
    }

    // Act

    let package = test_runner.compile_and_publish("./tests/blueprints/resource_creator");
    let mut builder = ManifestBuilder::new();
    builder
        .lock_fee(test_runner.faucet_component(), 100u32.into())
        .create_proof_from_account(account, admin_auth);

    let builder = if lock {
        let function = match action {
            ResourceAuth::Mint => "lock_mintable",
            ResourceAuth::Burn => "lock_burnable",
            ResourceAuth::Withdraw => "lock_withdrawable",
            ResourceAuth::Deposit => "lock_depositable",
            ResourceAuth::Recall => "lock_recallable",
            ResourceAuth::UpdateMetadata => "lock_metadata_updateable",
        };

        let args = manifest_args!(token_address);
        builder.call_function(package, "ResourceCreator", function, args)
    } else {
        match action {
            ResourceAuth::Mint => {
                builder.set_group_access_rule(
                    token_address.into(),
                    ObjectKey::SELF,
                    "mint".to_string(),
                    rule!(require(updated_auth)),
                )
            },
            ResourceAuth::Burn => {
                builder.set_method_access_rule(
                    token_address.into(),
                    MethodKey::new(ObjectModuleId::SELF, RESOURCE_MANAGER_BURN_IDENT),
                    rule!(require(updated_auth)),
                )
            },
            ResourceAuth::Withdraw => {
                builder.set_group_access_rule(
                    token_address.into(),
                    ObjectKey::ChildBlueprint(FUNGIBLE_VAULT_BLUEPRINT.to_string()),
                    "withdraw".to_string(),
                    rule!(require(updated_auth)),
                )
            },
            ResourceAuth::Deposit => builder.call_function(
                package,
                "ResourceCreator",
                "set_depositable",
                manifest_args!(token_address, updated_auth),
            ),
            ResourceAuth::Recall => {
                builder.set_group_access_rule(
                    token_address.into(),
                    ObjectKey::ChildBlueprint(FUNGIBLE_VAULT_BLUEPRINT.to_string()),
                    "deposit".to_string(),
                    rule!(require(updated_auth)),
                )
            },
            ResourceAuth::UpdateMetadata => {
                builder.set_method_access_rule(
                    token_address.into(),
                    MethodKey::new(ObjectModuleId::Metadata, METADATA_SET_IDENT),
                    rule!(require(updated_auth)),
                )
            },
        }
    };

    let manifest = builder
        .call_method(
            account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    receipt
}

#[test]
fn locked_mint_auth_cannot_be_updated() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Mint, false);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_mint_auth_cannot_be_relocked() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Mint, true);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_burn_auth_cannot_be_updated() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Burn, false);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_burn_auth_cannot_be_relocked() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Burn, true);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_withdraw_auth_cannot_be_updated() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Withdraw, false);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_withdraw_auth_cannot_be_relocked() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Withdraw, true);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_deposit_auth_cannot_be_updated() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Deposit, false);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_deposit_auth_cannot_be_relocked() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Deposit, true);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_recall_auth_cannot_be_updated() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Recall, false);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_recall_auth_cannot_be_relocked() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::Recall, true);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_update_metadata_auth_cannot_be_updated() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::UpdateMetadata, false);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}

#[test]
fn locked_update_metadata_auth_cannot_be_relocked() {
    let receipt = lock_resource_auth_and_try_update(ResourceAuth::UpdateMetadata, true);
    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(e, RuntimeError::ModuleError(ModuleError::AuthError(..)))
    })
}
