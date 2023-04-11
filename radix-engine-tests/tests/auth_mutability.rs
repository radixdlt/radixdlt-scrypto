extern crate core;

use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
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
    let (function, args) = if lock {
        let function = match action {
            ResourceAuth::Mint => "lock_mintable",
            ResourceAuth::Burn => "lock_burnable",
            ResourceAuth::Withdraw => "lock_withdrawable",
            ResourceAuth::Deposit => "lock_depositable",
            ResourceAuth::Recall => "lock_recallable",
            ResourceAuth::UpdateMetadata => "lock_metadata_updateable",
        };

        let args = manifest_args!(token_address);
        (function, args)
    } else {
        let function = match action {
            ResourceAuth::Mint => "set_mintable",
            ResourceAuth::Burn => "set_burnable",
            ResourceAuth::Withdraw => "set_withdrawable",
            ResourceAuth::Deposit => "set_depositable",
            ResourceAuth::Recall => "set_recallable",
            ResourceAuth::UpdateMetadata => "set_updateable_metadata",
        };
        let args = manifest_args!(token_address, updated_auth);
        (function, args)
    };

    let package = test_runner.compile_and_publish("./tests/blueprints/resource_creator");
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 100u32.into())
        .create_proof_from_account(account, admin_auth)
        .call_function(package, "ResourceCreator", function, args)
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
