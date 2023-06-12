extern crate core;

use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::{require, FromPublicKey};
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_all_resource_roles_have_immutable_updater() {
    for key in ALL_RESOURCE_AUTH_KEYS {
        ensure_auth_updater_is_immutable(key);
    }
}

fn ensure_auth_updater_is_immutable(action: ResourceMethodAuthKey) {
    // Arrange 1
    let mut test_runner = TestRunner::builder().build();
    let resource_address = test_runner.create_everything_allowed_non_fungible_resource();

    // Act - check that despite everything being allowed, you cannot update the role mutability
    // In other words, both roles are always set to have mutability always be the updater role
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .update_role_mutability(
                    resource_address.into(),
                    action.action_role_key(),
                    (RoleList::none(), false),
                )
                .build(),
            vec![],
        )
        .expect_auth_failure();
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .update_role_mutability(
                    resource_address.into(),
                    action.updater_role_key(),
                    (RoleList::none(), false),
                )
                .build(),
            vec![],
        )
        .expect_auth_failure();
}

#[test]
fn test_locked_resource_auth_cannot_be_updated() {
    for key in ALL_RESOURCE_AUTH_KEYS {
        assert_locked_auth_can_no_longer_be_updated(key);
    }
}

pub fn assert_locked_auth_can_no_longer_be_updated(action: ResourceMethodAuthKey) {
    // Arrange 1
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();
    let token_address = test_runner.create_everything_allowed_non_fungible_resource();
    let admin_auth = test_runner.create_non_fungible_resource(account);

    // Act 1 - Show that updating the action auth is initially possible
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_proof_from_account(account, admin_auth)
                .update_role(
                    token_address.into(),
                    action.action_role_key(),
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_proof_from_account(account, admin_auth)
                .update_role(
                    token_address.into(),
                    action.updater_role_key(),
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();

    // Act 2 - Double check that the previous updating the action auth still lets us update
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_proof_from_account(account, admin_auth)
                .update_role(
                    token_address.into(),
                    action.action_role_key(),
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_proof_from_account(account, admin_auth)
                .update_role(
                    token_address.into(),
                    action.updater_role_key(),
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();

    // Arrange - We now use the updater role to update the updater role to DenyAll
    {
        test_runner
            .execute_manifest_ignoring_fee(
                ManifestBuilder::new()
                    .create_proof_from_account(account, admin_auth)
                    .update_role(
                        token_address.into(),
                        action.updater_role_key(),
                        AccessRule::DenyAll,
                    )
                    .build(),
                vec![NonFungibleGlobalId::from_public_key(&public_key)],
            )
            .expect_commit_success();
    }

    // Act 3 - After locking, now attempting to update the action or updater role should fail
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_proof_from_account(account, admin_auth)
                .update_role(
                    token_address.into(),
                    action.action_role_key(),
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_auth_failure();
    test_runner
        .execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_proof_from_account(account, admin_auth)
                .update_role(
                    token_address.into(),
                    action.updater_role_key(),
                    rule!(require(admin_auth)),
                )
                .build(),
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_auth_failure();
}
