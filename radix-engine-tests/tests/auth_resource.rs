extern crate core;

use radix_engine::types::*;
use radix_engine_interface::api::ObjectModuleId;
use radix_engine_interface::blueprints::resource::{require, FromPublicKey};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

enum Action {
    Mint,
    Burn,
    Withdraw,
    Deposit,
    Recall,
    Freeze,
}

impl Action {
    fn get_role(&self) -> (ObjectModuleId, RoleKey) {
        match self {
            Action::Mint => (ObjectModuleId::Main, RoleKey::new(MINTER_ROLE)),
            Action::Burn => (ObjectModuleId::Main, RoleKey::new(BURNER_ROLE)),
            Action::Withdraw => (ObjectModuleId::Main, RoleKey::new(WITHDRAWER_ROLE)),
            Action::Deposit => (ObjectModuleId::Main, RoleKey::new(DEPOSITOR_ROLE)),
            Action::Recall => (ObjectModuleId::Main, RoleKey::new(RECALLER_ROLE)),
            Action::Freeze => (ObjectModuleId::Main, RoleKey::new(FREEZER_ROLE)),
        }
    }
}

fn test_resource_auth(action: Action, update_auth: bool, use_other_auth: bool, expect_err: bool) {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let (
        token_address,
        mint_auth,
        burn_auth,
        withdraw_auth,
        recall_auth,
        _update_metadata_auth,
        freeze_auth,
        admin_auth,
    ) = test_runner.create_restricted_token(OwnerRole::None, account);
    let (_, updated_auth) = test_runner.create_restricted_burn_token(account);

    if update_auth {
        let (module, role_key) = action.get_role();
        let manifest = ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 500u32.into())
            .create_proof_from_account(account, admin_auth)
            .update_role(
                token_address.into(),
                module,
                role_key,
                rule!(require(updated_auth)),
            )
            .build();
        test_runner
            .execute_manifest(
                manifest,
                btreeset![NonFungibleGlobalId::from_public_key(&public_key)],
            )
            .expect_commit_success();
    }

    let auth_to_use = if use_other_auth {
        updated_auth
    } else {
        match action {
            Action::Mint => mint_auth,
            Action::Burn => burn_auth,
            Action::Withdraw => withdraw_auth,
            Action::Deposit => mint_auth, // Any bad auth
            Action::Recall => recall_auth,
            Action::Freeze => freeze_auth,
        }
    };

    // Act
    let mut builder = ManifestBuilder::new();
    builder.lock_fee(test_runner.faucet_component(), 500u32.into());
    builder.create_proof_from_account_of_amount(account, auth_to_use, Decimal::one());

    match action {
        Action::Mint => builder
            .mint_fungible(token_address, dec!("1.0"))
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            ),
        Action::Burn => builder
            .create_proof_from_account(account, withdraw_auth)
            .withdraw_from_account(account, token_address, dec!("1.0"))
            .burn_from_worktop(dec!("1.0"), token_address)
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            ),
        Action::Withdraw => builder
            .withdraw_from_account(account, token_address, dec!("1.0"))
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            ),
        Action::Deposit => builder
            .create_proof_from_account(account, withdraw_auth)
            .withdraw_from_account(account, token_address, dec!("1.0"))
            .take_all_from_worktop(token_address, |builder, bucket_id| {
                builder.call_method(account, "try_deposit_or_abort", manifest_args!(bucket_id))
            })
            .call_method(
                account,
                "try_deposit_batch_or_abort",
                manifest_args!(ManifestExpression::EntireWorktop),
            ),
        Action::Recall => {
            let vaults = test_runner.get_component_vaults(account, token_address);
            let vault_id = vaults[0];

            builder
                .recall(InternalAddress::new_or_panic(vault_id.into()), Decimal::ONE)
                .call_method(
                    account,
                    "try_deposit_batch_or_abort",
                    manifest_args!(ManifestExpression::EntireWorktop),
                )
        }
        Action::Freeze => {
            let vaults = test_runner.get_component_vaults(account, token_address);
            let vault_id = vaults[0];
            builder.freeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        }
    };

    let manifest = builder.build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    if expect_err {
        receipt.expect_specific_failure(is_auth_error);
    } else {
        receipt.expect_commit_success();
    }
}

#[test]
fn can_mint_with_right_auth() {
    test_resource_auth(Action::Mint, false, false, false);
    test_resource_auth(Action::Mint, true, true, false);
}

#[test]
fn cannot_mint_with_wrong_auth() {
    test_resource_auth(Action::Mint, false, true, true);
    test_resource_auth(Action::Mint, true, false, true);
}

#[test]
fn can_burn_with_auth() {
    test_resource_auth(Action::Burn, false, false, false);
    test_resource_auth(Action::Burn, true, true, false);
}

#[test]
fn cannot_burn_with_wrong_auth() {
    test_resource_auth(Action::Burn, false, true, true);
    test_resource_auth(Action::Burn, true, false, true);
}

#[test]
fn can_withdraw_with_auth() {
    test_resource_auth(Action::Withdraw, false, false, false);
    test_resource_auth(Action::Withdraw, true, true, false);
}

#[test]
fn cannot_withdraw_with_wrong_auth() {
    test_resource_auth(Action::Withdraw, false, true, true);
    test_resource_auth(Action::Withdraw, true, false, true);
}

#[test]
fn can_recall_with_auth() {
    test_resource_auth(Action::Recall, false, false, false);
    test_resource_auth(Action::Recall, true, true, false);
}

#[test]
fn cannot_recall_with_wrong_auth() {
    test_resource_auth(Action::Recall, false, true, true);
    test_resource_auth(Action::Recall, true, false, true);
}

#[test]
fn can_freeze_with_auth() {
    test_resource_auth(Action::Freeze, false, false, false);
    test_resource_auth(Action::Freeze, true, true, false);
}

#[test]
fn cannot_freeze_with_wrong_auth() {
    test_resource_auth(Action::Freeze, false, true, true);
    test_resource_auth(Action::Freeze, true, false, true);
}

#[test]
fn cannot_deposit_with_wrong_auth() {
    test_resource_auth(Action::Deposit, true, false, true);
}

#[test]
fn can_deposit_with_right_auth() {
    test_resource_auth(Action::Deposit, true, true, false);
}
