extern crate core;

use radix_common::prelude::*;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::resource::require;
use radix_engine_interface::types::FromPublicKey;
use scrypto_test::prelude::*;

enum Action {
    Mint,
    Burn,
    Withdraw,
    Deposit,
    Recall,
    Freeze,
}

impl Action {
    fn get_role(&self) -> (ModuleId, RoleKey) {
        match self {
            Action::Mint => (ModuleId::Main, RoleKey::new(MINTER_ROLE)),
            Action::Burn => (ModuleId::Main, RoleKey::new(BURNER_ROLE)),
            Action::Withdraw => (ModuleId::Main, RoleKey::new(WITHDRAWER_ROLE)),
            Action::Deposit => (ModuleId::Main, RoleKey::new(DEPOSITOR_ROLE)),
            Action::Recall => (ModuleId::Main, RoleKey::new(RECALLER_ROLE)),
            Action::Freeze => (ModuleId::Main, RoleKey::new(FREEZER_ROLE)),
        }
    }
}

fn test_resource_auth(action: Action, update_auth: bool, use_other_auth: bool, expect_err: bool) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let (
        token_address,
        mint_auth,
        burn_auth,
        withdraw_auth,
        recall_auth,
        _update_metadata_auth,
        freeze_auth,
        admin_auth,
    ) = ledger.create_restricted_token(account);
    let (_, updated_auth) = ledger.create_restricted_burn_token(account);

    if update_auth {
        let (module, role_key) = action.get_role();
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_amount(account, admin_auth, dec!(1))
            .set_role(
                token_address,
                module,
                role_key,
                rule!(require(updated_auth)),
            )
            .build();
        ledger
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
    let mut builder = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(account, auth_to_use, Decimal::one());

    builder = match action {
        Action::Mint => builder
            .mint_fungible(token_address, dec!("1.0"))
            .try_deposit_entire_worktop_or_abort(account, None),
        Action::Burn => builder
            .create_proof_from_account_of_amount(account, withdraw_auth, dec!(1))
            .withdraw_from_account(account, token_address, dec!("1.0"))
            .burn_from_worktop(dec!("1.0"), token_address)
            .try_deposit_entire_worktop_or_abort(account, None),
        Action::Withdraw => builder
            .withdraw_from_account(account, token_address, dec!("1.0"))
            .try_deposit_entire_worktop_or_abort(account, None),
        Action::Deposit => builder
            .create_proof_from_account_of_amount(account, withdraw_auth, dec!(1))
            .withdraw_from_account(account, token_address, dec!("1.0"))
            .take_all_from_worktop(token_address, "withdrawn")
            .try_deposit_or_abort(account, None, "withdrawn")
            .try_deposit_entire_worktop_or_abort(account, None),
        Action::Recall => {
            let vaults = ledger.get_component_vaults(account, token_address);
            let vault_id = vaults[0];

            builder
                .recall(InternalAddress::new_or_panic(vault_id.into()), Decimal::ONE)
                .try_deposit_entire_worktop_or_abort(account, None)
        }
        Action::Freeze => {
            let vaults = ledger.get_component_vaults(account, token_address);
            let vault_id = vaults[0];
            builder.freeze_withdraw(InternalAddress::new_or_panic(vault_id.into()))
        }
    };

    let manifest = builder.build();
    let receipt = ledger.execute_manifest(
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
