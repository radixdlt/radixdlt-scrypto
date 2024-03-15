extern crate core;

use radix_common::prelude::*;
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::resource::require;
use radix_engine_interface::types::FromPublicKey;
use scrypto_test::prelude::*;

pub enum TestResourceAction {
    Mint,
    Burn,
    UpdateNonFungibleData,
    Withdraw,
    Deposit,
    Recall,
    Freeze,
}

pub const ALL_RESOURCE_AUTH_KEYS: [TestResourceAction; 7] = [
    TestResourceAction::Mint,
    TestResourceAction::Burn,
    TestResourceAction::UpdateNonFungibleData,
    TestResourceAction::Withdraw,
    TestResourceAction::Deposit,
    TestResourceAction::Recall,
    TestResourceAction::Freeze,
];

impl TestResourceAction {
    pub fn action_role_key(&self) -> RoleKey {
        match self {
            Self::Mint => RoleKey::new(MINTER_ROLE),
            Self::Burn => RoleKey::new(BURNER_ROLE),
            Self::UpdateNonFungibleData => RoleKey::new(NON_FUNGIBLE_DATA_UPDATER_ROLE),
            Self::Withdraw => RoleKey::new(WITHDRAWER_ROLE),
            Self::Deposit => RoleKey::new(DEPOSITOR_ROLE),
            Self::Recall => RoleKey::new(RECALLER_ROLE),
            Self::Freeze => RoleKey::new(FREEZER_ROLE),
        }
    }

    pub fn updater_role_key(&self) -> RoleKey {
        match self {
            Self::Mint => RoleKey::new(MINTER_UPDATER_ROLE),
            Self::Burn => RoleKey::new(BURNER_UPDATER_ROLE),
            Self::UpdateNonFungibleData => RoleKey::new(NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE),
            Self::Withdraw => RoleKey::new(WITHDRAWER_UPDATER_ROLE),
            Self::Deposit => RoleKey::new(DEPOSITOR_UPDATER_ROLE),
            Self::Recall => RoleKey::new(RECALLER_UPDATER_ROLE),
            Self::Freeze => RoleKey::new(FREEZER_UPDATER_ROLE),
        }
    }
}

#[test]
fn test_locked_resource_auth_cannot_be_updated() {
    for key in ALL_RESOURCE_AUTH_KEYS {
        assert_locked_auth_can_no_longer_be_updated(key);
    }
}

pub fn assert_locked_auth_can_no_longer_be_updated(action: TestResourceAction) {
    // Arrange 1
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _, account) = ledger.new_allocated_account();
    let token_address = ledger.create_everything_allowed_non_fungible_resource(OwnerRole::None);
    let admin_auth = ledger.create_non_fungible_resource(account);

    // Act 1 - Show that updating both the action_role_key and updater_role_key is initially possible
    let role_key = action.action_role_key();
    let updater_role_key = action.updater_role_key();
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_non_fungibles(
                    account,
                    admin_auth,
                    [NonFungibleLocalId::integer(1)],
                )
                .set_role(
                    token_address,
                    ModuleId::Main,
                    role_key,
                    rule!(require(admin_auth)),
                )
                .set_role(
                    token_address,
                    ModuleId::Main,
                    updater_role_key,
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();

    // Act 2 - Double check that the previous updating the action auth still lets us update
    let role_key = action.action_role_key();
    let updater_role_key = action.updater_role_key();
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_non_fungibles(
                    account,
                    admin_auth,
                    [NonFungibleLocalId::integer(1)],
                )
                .set_role(
                    token_address,
                    ModuleId::Main,
                    role_key,
                    rule!(require(admin_auth)),
                )
                .set_role(
                    token_address,
                    ModuleId::Main,
                    updater_role_key,
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();

    // Arrange - We now use the updater role to update the updater role's rule to DenyAll
    {
        let role_key = action.updater_role_key();
        ledger
            .execute_manifest(
                ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .create_proof_from_account_of_non_fungibles(
                        account,
                        admin_auth,
                        [NonFungibleLocalId::integer(1)],
                    )
                    .set_role(token_address, ModuleId::Main, role_key, AccessRule::DenyAll)
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(&public_key)],
            )
            .expect_commit_success();
    }

    // Act 3 - After locking, now attempting to update the action or updater role should fail
    let role_key = action.action_role_key();
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_non_fungibles(
                    account,
                    admin_auth,
                    [NonFungibleLocalId::integer(1)],
                )
                .set_role(
                    token_address,
                    ModuleId::Main,
                    role_key,
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_auth_failure();

    let role_key = action.updater_role_key();
    ledger
        .execute_manifest(
            ManifestBuilder::new()
                .lock_fee_from_faucet()
                .create_proof_from_account_of_non_fungibles(
                    account,
                    admin_auth,
                    [NonFungibleLocalId::integer(1)],
                )
                .set_role(
                    token_address,
                    ModuleId::Main,
                    role_key,
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_auth_failure();
}
