use radix_common::prelude::*;
use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::blueprints::pool::v1::errors::two_resource_pool::Error as TwoResourcePoolError;
use radix_engine::blueprints::pool::v1::events::two_resource_pool::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine::transaction::{BalanceChange, TransactionReceipt};
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::object_modules::metadata::MetadataValue;
use radix_substate_store_queries::typed_substate_layout::FungibleResourceManagerError;
use scrypto::prelude::Pow;
use scrypto_test::prelude::*;

#[test]
pub fn two_resource_pool_can_be_instantiated() {
    TestEnvironment::new((18, 18));
}

pub fn test_set_metadata<F: FnOnce(TransactionReceipt)>(
    key: &str,
    pool: bool,
    sign: bool,
    result: F,
) {
    // Arrange
    let (owner_role, virtual_signature_badge) = {
        let public_key = Secp256k1PrivateKey::from_u64(1).unwrap().public_key();
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);
        let rule = rule!(require(virtual_signature_badge.clone()));
        (OwnerRole::Fixed(rule), virtual_signature_badge)
    };
    let mut ledger = TestEnvironment::new_with_owner((18, 18), owner_role);

    let global_address = if pool {
        GlobalAddress::from(ledger.pool_component_address)
    } else {
        GlobalAddress::from(ledger.pool_unit_resource_address)
    };

    // Act
    let initial_proofs = if sign {
        vec![virtual_signature_badge]
    } else {
        vec![]
    };
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .set_metadata(global_address, key, MetadataValue::Bool(false))
        .build();
    let receipt = ledger.ledger.execute_manifest(manifest, initial_proofs);

    // Assert
    result(receipt);
}

#[test]
pub fn cannot_set_pool_vault_number_metadata() {
    test_set_metadata("pool_vault_number", true, true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
            )
        });
    });
}

#[test]
pub fn cannot_set_pool_resources_metadata() {
    test_set_metadata("pool_resources", true, true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
            )
        });
    });
}

#[test]
pub fn cannot_set_pool_unit_metadata() {
    test_set_metadata("pool_unit", true, true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
            )
        });
    });
}

#[test]
pub fn can_set_some_arbitrary_metadata_if_owner() {
    test_set_metadata("some_other_key", true, true, |receipt| {
        receipt.expect_commit_success();
    });
}

#[test]
pub fn cannot_set_some_arbitrary_metadata_if_not_owner() {
    test_set_metadata("some_other_key", true, false, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
            )
        });
    });
}

#[test]
pub fn cannot_set_pool_resource_pool_metadata() {
    test_set_metadata("pool", false, true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::KeyValueEntryLocked)
            )
        });
    });
}

#[test]
pub fn can_set_pool_resource_pool_metadata_if_owner() {
    test_set_metadata("some_other_key", false, true, |receipt| {
        receipt.expect_commit_success();
    });
}

#[test]
pub fn cannot_set_pool_resource_pool_metadata_if_not_owner() {
    test_set_metadata("some_other_key", false, false, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemModuleError(SystemModuleError::AuthError(..))
            )
        });
    });
}

#[test]
pub fn contribution_provides_expected_pool_unit_resources1() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    let contribution1 = (ledger.pool_resource1, 100);
    let contribution2 = (ledger.pool_resource2, 100);
    let expected_pool_units = 100;
    let expected_change1 = 0;
    let expected_change2 = 0;

    // Act
    let receipt = ledger.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );

    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource1).cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource2).cloned(),
        if expected_change2 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change2.into()))
        }
    );
}

#[test]
pub fn contribution_provides_expected_pool_unit_resources2() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    {
        let contribution1 = (ledger.pool_resource1, 100);
        let contribution2 = (ledger.pool_resource2, 100);

        ledger
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();
    }

    // Arrange
    let contribution1 = (ledger.pool_resource1, 100);
    let contribution2 = (ledger.pool_resource2, 100);
    let expected_pool_units = 100;
    let expected_change1 = 0;
    let expected_change2 = 0;

    // Act
    let receipt = ledger.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );

    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource1).cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource2).cloned(),
        if expected_change2 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change2.into()))
        }
    );
}

#[test]
pub fn contribution_provides_expected_pool_unit_resources3() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    {
        let contribution1 = (ledger.pool_resource1, 100);
        let contribution2 = (ledger.pool_resource2, 100);

        ledger
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();
    }

    let contribution1 = (ledger.pool_resource1, 100);
    let contribution2 = (ledger.pool_resource2, 120);
    let expected_pool_units = 100;
    let expected_change1 = 0;
    let expected_change2 = 20;

    // Act
    let receipt = ledger.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );

    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource1).cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource2).cloned(),
        if expected_change2 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change2.into()))
        }
    );
}

#[test]
pub fn contribution_provides_expected_pool_unit_resources4() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    {
        let contribution1 = (ledger.pool_resource1, 1_000_000);
        let contribution2 = (ledger.pool_resource2, 81);

        ledger
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();
    }

    let contribution1 = (ledger.pool_resource1, 400_000);
    let contribution2 = (ledger.pool_resource2, 40);
    let expected_pool_units = 3600;
    let expected_change1 = 0;
    let expected_change2 = dec!("7.6");

    // Act
    let receipt = ledger.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );

    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource1).cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource2).cloned(),
        if expected_change2 == Decimal::ZERO {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change2.into()))
        }
    );
}

#[test]
pub fn contribution_provides_expected_pool_unit_resources5() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    {
        let contribution1 = (ledger.pool_resource1, 100);
        let contribution2 = (ledger.pool_resource2, 100);

        ledger
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();

        ledger.redeem(100, true).expect_commit_success();
    }

    // Arrange
    let contribution1 = (ledger.pool_resource1, 50);
    let contribution2 = (ledger.pool_resource2, 50);
    let expected_pool_units = 50;
    let expected_change1 = 0;
    let expected_change2 = 0;

    // Act
    let receipt = ledger.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );

    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource1).cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource2).cloned(),
        if expected_change2 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change2.into()))
        }
    );
}

#[test]
fn contributing_tokens_that_do_not_belong_to_the_pool_fails() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));
    let resource_address = ledger
        .ledger
        .create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            18,
            ledger.account_component_address,
        );

    let contribution1 = (resource_address, 1_000_000);
    let contribution2 = (ledger.pool_resource2, 81);

    // Act
    let receipt = ledger.contribute(contribution1, contribution2, true);

    // Assert
    receipt
        .expect_specific_failure(is_two_resource_pool_resource_does_not_belong_to_the_pool_error);
}

#[test]
fn creating_a_pool_with_non_fungible_resources_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (_, _, account) = ledger.new_account(false);

    let non_fungible_resource = ledger.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            POOL_PACKAGE,
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
            TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
            TwoResourcePoolInstantiateManifestInput {
                resource_addresses: (non_fungible_resource.into(), XRD.into()),
                pool_manager_rule: rule!(allow_all),
                owner_role: OwnerRole::None,
                address_reservation: None,
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt
        .expect_specific_failure(is_two_resource_pool_does_non_fungible_resources_are_not_accepted)
}

#[test]
fn redemption_of_pool_units_rounds_down_for_resources_with_divisibility_not_18() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 2));

    let contribution1 = (ledger.pool_resource1, 100);
    let contribution2 = (ledger.pool_resource2, 100);
    ledger
        .contribute(contribution1, contribution2, true)
        .expect_commit_success();

    // Act
    let receipt = ledger.get_redemption_value(dec!("1.11111111111111"), true);

    // Assert
    assert_eq!(receipt[&ledger.pool_resource1], dec!("1.11111111111111"));
    assert_eq!(receipt[&ledger.pool_resource2], dec!("1.11")); // rounded due to divisibility == 2

    // Act
    let receipt = ledger.redeem(dec!("1.11111111111111"), true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );

    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource1).cloned(),
        Some(BalanceChange::Fungible(dec!("1.11111111111111")))
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource2).cloned(),
        Some(BalanceChange::Fungible(dec!("1.11")))
    );
}

#[test]
fn contribution_calculations_work_for_resources_with_divisibility_not_18() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 2));

    let contribution1 = (ledger.pool_resource1, 100);
    let contribution2 = (ledger.pool_resource2, 100);
    ledger
        .contribute(contribution1, contribution2, true)
        .expect_commit_success();

    // Act
    let receipt = ledger.contribute(
        (ledger.pool_resource1, dec!("1.1111111111111")),
        (ledger.pool_resource2, 500),
        true,
    );

    // Assert
    let pool_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.pool_component_address.as_node_id(),
    );
    assert_eq!(
        pool_balance_changes.get(&ledger.pool_resource1).cloned(),
        Some(BalanceChange::Fungible(dec!("1.1111111111111")))
    );
    assert_eq!(
        pool_balance_changes.get(&ledger.pool_resource2).cloned(),
        Some(BalanceChange::Fungible(dec!("1.11")))
    );
}

#[test]
fn contribution_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new((2, 2));

    // Act
    let receipt = ledger.contribute(
        (ledger.pool_resource1, dec!("2.22")),
        (ledger.pool_resource2, dec!("8.88")),
        true,
    );

    // Assert
    let ContributionEvent {
        contributed_resources,
        pool_units_minted,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if ledger.ledger.event_name(event_type_identifier) == "ContributionEvent"
                && is_pool_emitter(event_type_identifier)
            {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        contributed_resources,
        indexmap!(
            ledger.pool_resource1 => dec!("2.22"),
            ledger.pool_resource2 => dec!("8.88"),
        )
    );
    assert_eq!(pool_units_minted, dec!("4.44"));
}

#[test]
fn redemption_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new((2, 2));

    // Act
    ledger
        .contribute(
            (ledger.pool_resource1, dec!("2.22")),
            (ledger.pool_resource2, dec!("8.88")),
            true,
        )
        .expect_commit_success();
    let receipt = ledger.redeem(dec!("4.44"), true);

    // Assert
    let RedemptionEvent {
        pool_unit_tokens_redeemed,
        redeemed_resources,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if ledger.ledger.event_name(event_type_identifier) == "RedemptionEvent"
                && is_pool_emitter(event_type_identifier)
            {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(pool_unit_tokens_redeemed, dec!("4.44"));
    assert_eq!(
        redeemed_resources,
        indexmap!(
            ledger.pool_resource1 => dec!("2.22"),
            ledger.pool_resource2 => dec!("8.88"),
        )
    );
}

#[test]
fn deposits_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new((2, 2));

    // Act
    let receipt = ledger.protected_deposit(ledger.pool_resource1, dec!("2.22"), true);

    // Assert
    let DepositEvent {
        resource_address,
        amount,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if ledger.ledger.event_name(event_type_identifier) == "DepositEvent"
                && is_pool_emitter(event_type_identifier)
            {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(resource_address, ledger.pool_resource1);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraw_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new((2, 2));

    // Act
    ledger
        .protected_deposit(ledger.pool_resource1, dec!("2.22"), true)
        .expect_commit_success();
    let receipt = ledger.protected_withdraw(
        ledger.pool_resource1,
        dec!("2.22"),
        WithdrawStrategy::Exact,
        true,
    );

    // Assert
    let WithdrawEvent {
        resource_address,
        amount,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if ledger.ledger.event_name(event_type_identifier) == "WithdrawEvent"
                && is_pool_emitter(event_type_identifier)
            {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(resource_address, ledger.pool_resource1);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraw_with_rounding_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new((2, 2));

    // Act
    ledger
        .protected_deposit(ledger.pool_resource1, dec!("2.22"), true)
        .expect_commit_success();
    let receipt = ledger.protected_withdraw(
        ledger.pool_resource1,
        dec!("2.2211"),
        WithdrawStrategy::Rounded(RoundingMode::ToZero),
        true,
    );

    // Assert
    let WithdrawEvent {
        resource_address,
        amount,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if ledger.ledger.event_name(event_type_identifier) == "WithdrawEvent"
                && is_pool_emitter(event_type_identifier)
            {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(resource_address, ledger.pool_resource1);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn redemption_after_protected_deposit_redeems_expected_amount() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 2));

    ledger
        .contribute(
            (ledger.pool_resource1, 100),
            (ledger.pool_resource2, 100),
            true,
        )
        .expect_commit_success();

    // Act
    ledger
        .protected_deposit(ledger.pool_resource1, 500, true)
        .expect_commit_success();
    let receipt = ledger.redeem(100, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource1).cloned(),
        Some(BalanceChange::Fungible(600.into()))
    );
    assert_eq!(
        account_balance_changes.get(&ledger.pool_resource2).cloned(),
        Some(BalanceChange::Fungible(100.into()))
    );
}

#[test]
pub fn test_complete_interactions() {
    let mut ledger = TestEnvironment::new((18, 2));

    {
        // Act
        let receipt = ledger.contribute(
            (ledger.pool_resource1, 500),
            (ledger.pool_resource2, 200),
            true,
        );

        // Assert
        let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
            receipt.expect_commit_success(),
            ledger.account_component_address.as_node_id(),
        );
        assert_eq!(
            account_balance_changes
                .get(&ledger.pool_unit_resource_address)
                .cloned(),
            Some(BalanceChange::Fungible(dec!(316.2277660168379332)))
        );
        assert_eq!(
            account_balance_changes.get(&ledger.pool_resource1).cloned(),
            None
        );
        assert_eq!(
            account_balance_changes.get(&ledger.pool_resource2).cloned(),
            None
        );
    }

    {
        // Act
        let receipt = ledger.contribute(
            (ledger.pool_resource1, 700),
            (ledger.pool_resource2, 700),
            true,
        );

        // Assert
        let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
            receipt.expect_commit_success(),
            ledger.account_component_address.as_node_id(),
        );
        assert_eq!(
            account_balance_changes
                .get(&ledger.pool_unit_resource_address)
                .cloned(),
            Some(BalanceChange::Fungible(dec!(442.71887242357310648)))
        );
        assert_eq!(
            account_balance_changes.get(&ledger.pool_resource1).cloned(),
            None
        );
        assert_eq!(
            account_balance_changes.get(&ledger.pool_resource2).cloned(),
            Some(BalanceChange::Fungible(420.into()))
        );
    }
}

#[test]
pub fn protected_deposit_fails_without_proper_authority_present() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    // Act
    let receipt = ledger.protected_deposit(ledger.pool_resource1, 10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn protected_withdraw_fails_without_proper_authority_present() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    // Act
    ledger
        .protected_deposit(ledger.pool_resource1, 10, true)
        .expect_commit_success();
    let receipt =
        ledger.protected_withdraw(ledger.pool_resource1, 10, WithdrawStrategy::Exact, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn contribute_fails_without_proper_authority_present() {
    // Arrange
    let mut ledger = TestEnvironment::new((18, 18));

    // Act
    let receipt = ledger.contribute(
        (ledger.pool_resource1, 10),
        (ledger.pool_resource2, 10),
        false,
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
fn contribution_of_large_values_should_not_cause_panic() {
    // Arrange
    let max_mint_amount = Decimal::from_attos(I192::from(2).pow(152));
    let mut ledger = TestEnvironment::new((18, 18));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_fungible(ledger.pool_resource1, max_mint_amount)
        .mint_fungible(ledger.pool_resource1, max_mint_amount)
        .mint_fungible(ledger.pool_resource2, max_mint_amount)
        .mint_fungible(ledger.pool_resource2, max_mint_amount)
        .take_all_from_worktop(ledger.pool_resource1, "resource_1")
        .take_all_from_worktop(ledger.pool_resource2, "resource_2")
        .with_name_lookup(|builder, lookup| {
            let bucket1 = lookup.bucket("resource_1");
            let bucket2 = lookup.bucket("resource_2");
            builder.call_method(
                ledger.pool_component_address,
                TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                TwoResourcePoolContributeManifestInput {
                    buckets: (bucket1, bucket2),
                },
            )
        })
        .try_deposit_entire_worktop_or_abort(ledger.account_component_address, None)
        .build();

    // Act
    let receipt = ledger.execute_manifest(manifest, true);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::FungibleResourceManagerError(
                FungibleResourceManagerError::MaxMintAmountExceeded
            ))
        )
    });
}

#[test]
fn get_redemption_value_should_not_panic_on_large_values() {
    // Arrange
    let mint_amount = Decimal::from_attos(I192::from(2).pow(60));
    let mut ledger = TestEnvironment::new((18, 18));
    let receipt = ledger.contribute(
        (ledger.pool_resource1, mint_amount),
        (ledger.pool_resource2, mint_amount),
        true,
    );
    receipt.expect_commit_success();

    // Act
    let receipt = ledger.call_get_redemption_value(Decimal::MAX, true);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
                TwoResourcePoolError::InvalidGetRedemptionAmount
            ))
        )
    })
}

#[test]
fn contributing_to_a_pool_with_very_large_difference_in_reserves_succeeds() {
    // Arrange
    let max_mint_amount = Decimal::from_attos(I192::from(2).pow(152));
    let mut ledger = TestEnvironment::new((18, 18));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .mint_fungible(ledger.pool_resource1, max_mint_amount)
        .mint_fungible(ledger.pool_resource2, dec!("1"))
        .take_all_from_worktop(ledger.pool_resource1, "resource_1")
        .take_all_from_worktop(ledger.pool_resource2, "resource_2")
        .with_name_lookup(|builder, lookup| {
            let bucket1 = lookup.bucket("resource_1");
            let bucket2 = lookup.bucket("resource_2");
            builder.call_method(
                ledger.pool_component_address,
                TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                TwoResourcePoolContributeManifestInput {
                    buckets: (bucket1, bucket2),
                },
            )
        })
        .try_deposit_entire_worktop_or_abort(ledger.account_component_address, None)
        .build();
    ledger
        .execute_manifest(manifest, true)
        .expect_commit_success();

    // Act
    let receipt = ledger.contribute(
        (ledger.pool_resource1, dec!("5708990770.82384")),
        (ledger.pool_resource2, dec!("0.000000000000000001")),
        true,
    );

    // Assert
    receipt.expect_commit_success();
}

fn is_pool_emitter(event_type_identifier: &EventTypeIdentifier) -> bool {
    match event_type_identifier.0 {
        Emitter::Method(node_id, ModuleId::Main) => match node_id.entity_type() {
            Some(
                EntityType::GlobalOneResourcePool
                | EntityType::GlobalTwoResourcePool
                | EntityType::GlobalMultiResourcePool,
            ) => true,
            _ => false,
        },
        _ => false,
    }
}

struct TestEnvironment {
    ledger: DefaultLedgerSimulator,

    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,

    pool_resource1: ResourceAddress,
    pool_resource2: ResourceAddress,

    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl TestEnvironment {
    pub fn new((divisibility1, divisibility2): (u8, u8)) -> Self {
        Self::new_with_owner((divisibility1, divisibility2), OwnerRole::None)
    }

    pub fn new_with_owner((divisibility1, divisibility2): (u8, u8), owner_role: OwnerRole) -> Self {
        let mut ledger = LedgerSimulatorBuilder::new().build();
        let (public_key, _, account) = ledger.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let pool_resource1 = ledger.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility1,
            account,
        );
        let pool_resource2 = ledger.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility2,
            account,
        );

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    POOL_PACKAGE,
                    TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                    TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                    TwoResourcePoolInstantiateManifestInput {
                        resource_addresses: (pool_resource1.into(), pool_resource2.into()),
                        pool_manager_rule: rule!(require(virtual_signature_badge)),
                        owner_role,
                        address_reservation: None,
                    },
                )
                .build();
            let receipt = ledger.execute_manifest(manifest, vec![]);
            let commit_result = receipt.expect_commit_success();

            (
                commit_result.new_component_addresses()[0],
                commit_result.new_resource_addresses()[0],
            )
        };

        Self {
            ledger,
            pool_component_address: pool_component,
            pool_unit_resource_address: pool_unit_resource,
            pool_resource1,
            pool_resource2,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    pub fn contribute<A, B>(
        &mut self,
        (resource_address1, amount1): (ResourceAddress, A),
        (resource_address2, amount2): (ResourceAddress, B),
        sign: bool,
    ) -> TransactionReceipt
    where
        A: Into<Decimal>,
        B: Into<Decimal>,
    {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(resource_address1, amount1.into())
            .mint_fungible(resource_address2, amount2.into())
            .take_all_from_worktop(resource_address1, "resource_1")
            .take_all_from_worktop(resource_address2, "resource_2")
            .with_name_lookup(|builder, lookup| {
                let bucket1 = lookup.bucket("resource_1");
                let bucket2 = lookup.bucket("resource_2");
                builder.call_method(
                    self.pool_component_address,
                    TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    TwoResourcePoolContributeManifestInput {
                        buckets: (bucket1, bucket2),
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn redeem<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_from_account(
                self.account_component_address,
                self.pool_unit_resource_address,
                amount.into(),
            )
            .take_all_from_worktop(self.pool_unit_resource_address, "pool_units")
            .with_name_lookup(|builder, lookup| {
                let bucket = lookup.bucket("pool_units");
                builder.call_method(
                    self.pool_component_address,
                    TWO_RESOURCE_POOL_REDEEM_IDENT,
                    TwoResourcePoolRedeemManifestInput { bucket },
                )
            })
            .try_deposit_entire_worktop_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_deposit<D: Into<Decimal>>(
        &mut self,
        resource_address: ResourceAddress,
        amount: D,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(resource_address, amount.into())
            .take_all_from_worktop(resource_address, "deposit")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    TwoResourcePoolProtectedDepositManifestInput {
                        bucket: lookup.bucket("deposit"),
                    },
                )
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        resource_address: ResourceAddress,
        amount: D,
        withdraw_strategy: WithdrawStrategy,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.pool_component_address,
                TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                TwoResourcePoolProtectedWithdrawManifestInput {
                    resource_address: resource_address.into(),
                    amount: amount.into(),
                    withdraw_strategy,
                },
            )
            .try_deposit_entire_worktop_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        sign: bool,
    ) -> TransactionReceipt {
        self.ledger
            .execute_manifest(manifest, self.initial_proofs(sign))
    }

    fn virtual_signature_badge(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::from_public_key(&self.account_public_key)
    }

    fn initial_proofs(&self, sign: bool) -> Vec<NonFungibleGlobalId> {
        if sign {
            vec![self.virtual_signature_badge()]
        } else {
            vec![]
        }
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> TwoResourcePoolGetRedemptionValueOutput {
        let receipt = self.call_get_redemption_value(amount_of_pool_units, sign);
        receipt.expect_commit_success().output(1)
    }

    fn call_get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.pool_component_address,
                TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                TwoResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                },
            )
            .build();
        self.execute_manifest(manifest, sign)
    }
}

fn is_two_resource_pool_resource_does_not_belong_to_the_pool_error(
    runtime_error: &RuntimeError,
) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
            TwoResourcePoolError::ResourceDoesNotBelongToPool { .. }
        ))
    )
}

fn is_two_resource_pool_does_non_fungible_resources_are_not_accepted(
    runtime_error: &RuntimeError,
) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::TwoResourcePoolError(
            TwoResourcePoolError::NonFungibleResourcesAreNotAccepted { .. }
        ))
    )
}
