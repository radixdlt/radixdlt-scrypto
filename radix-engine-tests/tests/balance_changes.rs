use radix_engine::{transaction::BalanceChange, types::*};
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_balance_changes_when_success() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address = test_runner.compile_and_publish_with_owner(
        "./tests/blueprints/balance_changes",
        owner_badge_addr.clone(),
    );

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .set_package_royalty_config(
                package_address,
                btreemap!(
                    "BalanceChangesTest".to_owned() => RoyaltyConfigBuilder::new()
                        .add_rule("put", 2)
                        .add_rule("boom", 2)
                        .default(0)
                ),
            )
            .call_function(
                package_address,
                "BalanceChangesTest",
                "instantiate",
                manifest_args!(),
            )
            .build(),
        vec![
            NonFungibleGlobalId::from_public_key(&public_key),
            owner_badge_addr.clone(),
        ],
    );
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Call the put method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 100.into())
            .withdraw_from_account(account, RADIX_TOKEN, Decimal::ONE)
            .take_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(component_address, "put", manifest_args!(bucket))
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let result = receipt.expect_commit(true);
    assert_eq!(
        result.balance_changes(),
        &indexmap!(
            test_runner.faucet_component().into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(-(result.fee_summary.total_execution_cost_xrd + result.fee_summary.total_royalty_cost_xrd))
            ),
            package_address.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(dec!("0.0000002"))
            ),
            component_address.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(dec!("1.0000001"))
            ),
            account.into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(dec!("-1"))
            )
        )
    );
    assert!(result.direct_vault_updates().is_empty());
}

#[test]
fn test_balance_changes_when_failure() {
    // Basic setup
    let mut test_runner = TestRunner::builder().build();
    let (public_key, _, account) = test_runner.new_allocated_account();

    // Publish package
    let owner_badge_resource = test_runner.create_non_fungible_resource(account);
    let owner_badge_addr =
        NonFungibleGlobalId::new(owner_badge_resource, NonFungibleLocalId::integer(1));
    let package_address = test_runner.compile_and_publish_with_owner(
        "./tests/blueprints/balance_changes",
        owner_badge_addr.clone(),
    );

    // Instantiate component
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(account, 10u32.into())
            .set_package_royalty_config(
                package_address,
                btreemap!(
                    "BalanceChangesTest".to_owned() => RoyaltyConfigBuilder::new()
                        .add_rule("put", 2)
                        .add_rule("boom", 2)
                        .default(0)
                ),
            )
            .call_function(
                package_address,
                "BalanceChangesTest",
                "instantiate",
                manifest_args!(),
            )
            .build(),
        vec![
            NonFungibleGlobalId::from_public_key(&public_key),
            owner_badge_addr.clone(),
        ],
    );
    let component_address = receipt.expect_commit(true).new_component_addresses()[0];

    // Call the put method
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee(test_runner.faucet_component(), 100.into())
            .withdraw_from_account(account, RADIX_TOKEN, Decimal::ONE)
            .take_from_worktop(RADIX_TOKEN, |builder, bucket| {
                builder.call_method(component_address, "boom", manifest_args!(bucket))
            })
            .build(),
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    let vault_id = test_runner.get_component_vaults(test_runner.faucet_component(), RADIX_TOKEN)[0];
    let result = receipt.expect_commit(false);
    assert!(result.balance_changes().is_empty());
    assert_eq!(
        result.direct_vault_updates(),
        &indexmap!(
            vault_id => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(-(result.fee_summary.total_execution_cost_xrd + result.fee_summary.total_royalty_cost_xrd))
            )
        )
    )
}

#[test]
fn test_balance_changes_when_recall() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let recallable_token = test_runner.create_recallable_token(account);
    let vaults = test_runner.get_component_vaults(account, recallable_token);
    let vault_id = vaults[0];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .recall(LocalAddress::new_unchecked(vault_id.into()), Decimal::one())
        .call_method(
            other_account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    let result = receipt.expect_commit(true);
    assert_eq!(
        result.balance_changes(),
        &indexmap!(
            test_runner.faucet_component().into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(-(result.fee_summary.total_execution_cost_xrd + result.fee_summary.total_royalty_cost_xrd))
            ),
            other_account.into() => indexmap!(
                recallable_token => BalanceChange::Fungible(dec!("1"))
            ),
        )
    );
    assert_eq!(
        result.direct_vault_updates(),
        &indexmap!(
            vault_id => indexmap!(
                recallable_token => BalanceChange::Fungible(dec!("-1"))
            )
        )
    )
}

#[test]
fn test_balance_changes_when_transferring_non_fungibles() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (pk, _, account) = test_runner.new_allocated_account();
    let (_, _, other_account) = test_runner.new_allocated_account();

    let resource_address = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10u32.into())
        .withdraw_from_account(account, resource_address, dec!("1.0"))
        .call_method(
            other_account,
            "deposit_batch",
            manifest_args!(ManifestExpression::EntireWorktop),
        )
        .build();
    let receipt =
        test_runner.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&pk)]);

    // Assert
    let result = receipt.expect_commit(true);
    assert_eq!(
        result.balance_changes(),
        &indexmap!(
            test_runner.faucet_component().into() => indexmap!(
                RADIX_TOKEN => BalanceChange::Fungible(-(result.fee_summary.total_execution_cost_xrd + result.fee_summary.total_royalty_cost_xrd))
            ),
            account.into() => indexmap!(
                resource_address => BalanceChange::NonFungible {
                    added: BTreeSet::new(),
                    removed: btreeset!(NonFungibleLocalId::integer(1))
                }
            ),
            other_account.into() => indexmap!(
                resource_address => BalanceChange::NonFungible {
                    added: btreeset!(NonFungibleLocalId::integer(1)),
                    removed: BTreeSet::new()
                }
            ),
        )
    );
    assert!(result.direct_vault_updates().is_empty())
}
