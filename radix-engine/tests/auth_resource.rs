extern crate core;

use radix_engine::types::*;
use radix_engine_interface::model::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

enum Action {
    Mint,
    Burn,
    Withdraw,
    Deposit,
    Recall,
    UpdateMetadata,
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
        update_metadata_auth,
        admin_auth,
    ) = test_runner.create_restricted_token(account);
    let (_, updated_auth) = test_runner.create_restricted_burn_token(account);

    if update_auth {
        let function = match action {
            Action::Mint => "set_mintable",
            Action::Burn => "set_burnable",
            Action::Withdraw => "set_withdrawable",
            Action::Deposit => "set_depositable",
            Action::Recall => "set_recallable",
            Action::UpdateMetadata => "set_updateable_metadata",
        };
        test_runner.update_resource_auth(
            function,
            admin_auth,
            token_address,
            updated_auth,
            account,
            public_key,
        );
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
            Action::UpdateMetadata => update_metadata_auth,
        }
    };

    // Act
    let mut builder = ManifestBuilder::new();
    builder.lock_fee(FAUCET_COMPONENT, 10u32.into());
    builder.create_proof_from_account_by_amount(account, Decimal::one(), auth_to_use);

    match action {
        Action::Mint => builder
            .mint_fungible(token_address, Decimal::from("1.0"))
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            ),
        Action::Burn => builder
            .create_proof_from_account(account, withdraw_auth)
            .withdraw_from_account_by_amount(account, Decimal::from("1.0"), token_address)
            .burn(Decimal::from("1.0"), token_address)
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            ),
        Action::Withdraw => builder
            .withdraw_from_account_by_amount(account, Decimal::from("1.0"), token_address)
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            ),
        Action::Deposit => builder
            .create_proof_from_account(account, withdraw_auth)
            .withdraw_from_account_by_amount(account, Decimal::from("1.0"), token_address)
            .take_from_worktop(token_address, |builder, bucket_id| {
                builder.call_method(account, "deposit", args!(bucket_id))
            })
            .call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            ),
        Action::Recall => {
            let vaults = test_runner.get_component_vaults(account, token_address);
            let vault_id = vaults[0];

            builder.recall(vault_id, Decimal::ONE).call_method(
                account,
                "deposit_batch",
                args!(ManifestExpression::EntireWorktop),
            )
        }
        Action::UpdateMetadata => builder.set_metadata(
            GlobalAddress::Resource(token_address),
            "key".to_string(),
            "value".to_string(),
        ),
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
fn can_update_metadata_with_auth() {
    test_resource_auth(Action::UpdateMetadata, false, false, false);
    test_resource_auth(Action::UpdateMetadata, true, true, false);
}

#[test]
fn cannot_update_metadata_with_wrong_auth() {
    test_resource_auth(Action::UpdateMetadata, false, true, true);
    test_resource_auth(Action::UpdateMetadata, true, false, true);
}

#[test]
fn cannot_deposit_with_wrong_auth() {
    test_resource_auth(Action::Deposit, true, false, true);
}

#[test]
fn can_deposit_with_right_auth() {
    test_resource_auth(Action::Deposit, true, true, false);
}
