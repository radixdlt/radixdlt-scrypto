use radix_engine::blueprints::pool::two_resource_pool::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine::transaction::{BalanceChange, TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::blueprints::pool::*;
use scrypto_unit::*;
use transaction::builder::*;

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
    let mut test_runner = TestEnvironment::new((18, 18));

    // Act
    let receipt = if pool {
        test_runner.set_pool_metadata(key, MetadataValue::U8(2u8), sign)
    } else {
        test_runner.set_pool_unit_resource_metadata(key, MetadataValue::U8(2u8), sign)
    };

    // Assert
    result(receipt);
}

#[test]
pub fn cannot_set_pool_vault_number_metadata() {
    test_set_metadata("pool_vault_number", true, true, |receipt| {
        receipt.expect_specific_failure(|e| {
            matches!(
                e,
                RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
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
                RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
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
                RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
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
                RuntimeError::SystemError(SystemError::MutatingImmutableSubstate)
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
    let mut test_runner = TestEnvironment::new((18, 18));

    let contribution1 = (test_runner.pool_resource1, 100);
    let contribution2 = (test_runner.pool_resource2, 100);
    let expected_pool_units = 100;
    let expected_change1 = 0;
    let expected_change2 = 0;

    // Act
    let receipt = test_runner.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();

    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
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
    let mut test_runner = TestEnvironment::new((18, 18));

    {
        let contribution1 = (test_runner.pool_resource1, 100);
        let contribution2 = (test_runner.pool_resource2, 100);

        test_runner
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();
    }

    // Arrange
    let contribution1 = (test_runner.pool_resource1, 100);
    let contribution2 = (test_runner.pool_resource2, 100);
    let expected_pool_units = 100;
    let expected_change1 = 0;
    let expected_change2 = 0;

    // Act
    let receipt = test_runner.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();

    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
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
    let mut test_runner = TestEnvironment::new((18, 18));

    {
        let contribution1 = (test_runner.pool_resource1, 100);
        let contribution2 = (test_runner.pool_resource2, 100);

        test_runner
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();
    }

    let contribution1 = (test_runner.pool_resource1, 100);
    let contribution2 = (test_runner.pool_resource2, 120);
    let expected_pool_units = 100;
    let expected_change1 = 0;
    let expected_change2 = 20;

    // Act
    let receipt = test_runner.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();

    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
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
    let mut test_runner = TestEnvironment::new((18, 18));

    {
        let contribution1 = (test_runner.pool_resource1, 1_000_000);
        let contribution2 = (test_runner.pool_resource2, 81);

        test_runner
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();
    }

    let contribution1 = (test_runner.pool_resource1, 400_000);
    let contribution2 = (test_runner.pool_resource2, 40);
    let expected_pool_units = 3600;
    let expected_change1 = 0;
    let expected_change2 = dec!("7.6");

    // Act
    let receipt = test_runner.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();

    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
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
    let mut test_runner = TestEnvironment::new((18, 18));

    {
        let contribution1 = (test_runner.pool_resource1, 100);
        let contribution2 = (test_runner.pool_resource2, 100);

        test_runner
            .contribute(contribution1, contribution2, true)
            .expect_commit_success();

        test_runner.redeem(100, true).expect_commit_success();
    }

    // Arrange
    let contribution1 = (test_runner.pool_resource1, 50);
    let contribution2 = (test_runner.pool_resource2, 50);
    let expected_pool_units = 50;
    let expected_change1 = 0;
    let expected_change2 = 0;

    // Act
    let receipt = test_runner.contribute(contribution1, contribution2, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();

    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        if expected_change1 == 0 {
            None
        } else {
            Some(BalanceChange::Fungible(expected_change1.into()))
        }
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
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
    let mut test_runner = TestEnvironment::new((18, 18));
    let resource_address = test_runner
        .test_runner
        .create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            18,
            test_runner.account_component_address,
        );

    let contribution1 = (resource_address, 1_000_000);
    let contribution2 = (test_runner.pool_resource2, 81);

    // Act
    let receipt = test_runner.contribute(contribution1, contribution2, true);

    // Assert
    receipt
        .expect_specific_failure(is_two_resource_pool_resource_does_not_belong_to_the_pool_error);
}

#[test]
fn creating_a_pool_with_non_fungible_resources_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().without_trace().build();
    let (_, _, account) = test_runner.new_account(false);

    let non_fungible_resource = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            POOL_PACKAGE,
            TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
            TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
            to_manifest_value_and_unwrap!(&TwoResourcePoolInstantiateManifestInput {
                resource_addresses: (non_fungible_resource, RADIX_TOKEN),
                pool_manager_rule: rule!(allow_all),
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt
        .expect_specific_failure(is_two_resource_pool_does_non_fungible_resources_are_not_accepted)
}

#[test]
fn redemption_of_pool_units_rounds_down_for_resources_with_divisibility_not_18() {
    // Arrange
    let mut test_runner = TestEnvironment::new((18, 2));

    let contribution1 = (test_runner.pool_resource1, 100);
    let contribution2 = (test_runner.pool_resource2, 100);
    test_runner
        .contribute(contribution1, contribution2, true)
        .expect_commit_success();

    // Act
    let receipt = test_runner.get_redemption_value(dec!("1.11111111111111"), true);

    // Assert
    assert_eq!(
        receipt[&test_runner.pool_resource1],
        dec!("1.11111111111111")
    );
    assert_eq!(receipt[&test_runner.pool_resource2], dec!("1.11")); // rounded due to divisibility == 2

    // Act
    let receipt = test_runner.redeem(dec!("1.11111111111111"), true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();

    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        Some(BalanceChange::Fungible(dec!("1.11111111111111")))
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
        Some(BalanceChange::Fungible(dec!("1.11")))
    );
}

#[test]
fn contribution_calculations_work_for_resources_with_divisibility_not_18() {
    // Arrange
    let mut test_runner = TestEnvironment::new((18, 2));

    let contribution1 = (test_runner.pool_resource1, 100);
    let contribution2 = (test_runner.pool_resource2, 100);
    test_runner
        .contribute(contribution1, contribution2, true)
        .expect_commit_success();

    // Act
    let receipt = test_runner.contribute(
        (test_runner.pool_resource1, dec!("1.1111111111111")),
        (test_runner.pool_resource2, 500),
        true,
    );

    // Assert
    let pool_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.pool_component_address))
        .unwrap();
    assert_eq!(
        pool_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        Some(BalanceChange::Fungible(dec!("1.1111111111111")))
    );
    assert_eq!(
        pool_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
        Some(BalanceChange::Fungible(dec!("1.11")))
    );
}

#[test]
fn contribution_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new((2, 2));

    // Act
    let receipt = test_runner.contribute(
        (test_runner.pool_resource1, dec!("2.22")),
        (test_runner.pool_resource2, dec!("8.88")),
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
            if test_runner.test_runner.event_name(event_type_identifier) == "ContributionEvent" {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(
        contributed_resources,
        btreemap!(
            test_runner.pool_resource1 => dec!("2.22"),
            test_runner.pool_resource2 => dec!("8.88"),
        )
    );
    assert_eq!(pool_units_minted, dec!("4.44"));
}

#[test]
fn redemption_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new((2, 2));

    // Act
    test_runner
        .contribute(
            (test_runner.pool_resource1, dec!("2.22")),
            (test_runner.pool_resource2, dec!("8.88")),
            true,
        )
        .expect_commit_success();
    let receipt = test_runner.redeem(dec!("4.44"), true);

    // Assert
    let RedemptionEvent {
        pool_unit_tokens_redeemed,
        redeemed_resources,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if test_runner.test_runner.event_name(event_type_identifier) == "RedemptionEvent" {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(pool_unit_tokens_redeemed, dec!("4.44"));
    assert_eq!(
        redeemed_resources,
        btreemap!(
            test_runner.pool_resource1 => dec!("2.22"),
            test_runner.pool_resource2 => dec!("8.88"),
        )
    );
}

#[test]
fn deposits_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new((2, 2));

    // Act
    let receipt = test_runner.protected_deposit(test_runner.pool_resource1, dec!("2.22"), true);

    // Assert
    let DepositEvent {
        resource_address,
        amount,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if test_runner.test_runner.event_name(event_type_identifier) == "DepositEvent" {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(resource_address, test_runner.pool_resource1);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraw_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new((2, 2));

    // Act
    test_runner
        .protected_deposit(test_runner.pool_resource1, dec!("2.22"), true)
        .expect_commit_success();
    let receipt = test_runner.protected_withdraw(test_runner.pool_resource1, dec!("2.22"), true);

    // Assert
    let WithdrawEvent {
        resource_address,
        amount,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if test_runner.test_runner.event_name(event_type_identifier) == "WithdrawEvent" {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(resource_address, test_runner.pool_resource1);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn redemption_after_protected_deposit_redeems_expected_amount() {
    // Arrange
    let mut test_runner = TestEnvironment::new((18, 2));

    test_runner
        .contribute(
            (test_runner.pool_resource1, 100),
            (test_runner.pool_resource2, 100),
            true,
        )
        .expect_commit_success();

    // Act
    test_runner
        .protected_deposit(test_runner.pool_resource1, 500, true)
        .expect_commit_success();
    let receipt = test_runner.redeem(100, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource1)
            .cloned(),
        Some(BalanceChange::Fungible(600.into()))
    );
    assert_eq!(
        account_balance_changes
            .get(&test_runner.pool_resource2)
            .cloned(),
        Some(BalanceChange::Fungible(100.into()))
    );
}

#[test]
pub fn test_complete_interactions() {
    let mut test_runner = TestEnvironment::new((18, 2));

    {
        // Act
        let receipt = test_runner.contribute(
            (test_runner.pool_resource1, 500),
            (test_runner.pool_resource2, 200),
            true,
        );

        // Assert
        let account_balance_changes = receipt
            .expect_commit_success()
            .balance_changes()
            .get(&GlobalAddress::from(test_runner.account_component_address))
            .unwrap();
        assert_eq!(
            account_balance_changes
                .get(&test_runner.pool_unit_resource_address)
                .cloned(),
            Some(BalanceChange::Fungible(
                (dec!("500") * dec!("200")).sqrt().unwrap()
            ))
        );
        assert_eq!(
            account_balance_changes
                .get(&test_runner.pool_resource1)
                .cloned(),
            None
        );
        assert_eq!(
            account_balance_changes
                .get(&test_runner.pool_resource2)
                .cloned(),
            None
        );
    }

    {
        // Act
        let receipt = test_runner.contribute(
            (test_runner.pool_resource1, 700),
            (test_runner.pool_resource2, 700),
            true,
        );

        // Assert
        let account_balance_changes = receipt
            .expect_commit_success()
            .balance_changes()
            .get(&GlobalAddress::from(test_runner.account_component_address))
            .unwrap();
        assert_eq!(
            account_balance_changes
                .get(&test_runner.pool_unit_resource_address)
                .cloned(),
            Some(BalanceChange::Fungible(dec!("442.718872423573106478")))
        );
        assert_eq!(
            account_balance_changes
                .get(&test_runner.pool_resource1)
                .cloned(),
            None
        );
        assert_eq!(
            account_balance_changes
                .get(&test_runner.pool_resource2)
                .cloned(),
            Some(BalanceChange::Fungible(420.into()))
        );
    }
}

#[test]
pub fn protected_deposit_fails_without_proper_authority_present() {
    // Arrange
    let mut test_runner = TestEnvironment::new((18, 18));

    // Act
    let receipt = test_runner.protected_deposit(test_runner.pool_resource1, 10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn protected_withdraw_fails_without_proper_authority_present() {
    // Arrange
    let mut test_runner = TestEnvironment::new((18, 18));

    // Act
    test_runner
        .protected_deposit(test_runner.pool_resource1, 10, true)
        .expect_commit_success();
    let receipt = test_runner.protected_withdraw(test_runner.pool_resource1, 10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn contribute_fails_without_proper_authority_present() {
    // Arrange
    let mut test_runner = TestEnvironment::new((18, 18));

    // Act
    let receipt = test_runner.contribute(
        (test_runner.pool_resource1, 10),
        (test_runner.pool_resource2, 10),
        false,
    );

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

struct TestEnvironment {
    test_runner: TestRunner,

    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,

    pool_resource1: ResourceAddress,
    pool_resource2: ResourceAddress,

    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl TestEnvironment {
    pub fn new((divisibility1, divisibility2): (u8, u8)) -> Self {
        let mut test_runner = TestRunner::builder().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let pool_resource1 = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility1,
            account,
        );
        let pool_resource2 = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility2,
            account,
        );

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    TWO_RESOURCE_POOL_BLUEPRINT_IDENT,
                    TWO_RESOURCE_POOL_INSTANTIATE_IDENT,
                    to_manifest_value_and_unwrap!(&TwoResourcePoolInstantiateManifestInput {
                        resource_addresses: (pool_resource1, pool_resource2),
                        pool_manager_rule: rule!(require(virtual_signature_badge)),
                    }),
                )
                .build();
            let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
            let commit_result = receipt.expect_commit_success();

            (
                commit_result
                    .new_component_addresses()
                    .get(0)
                    .unwrap()
                    .clone(),
                commit_result
                    .new_resource_addresses()
                    .get(0)
                    .unwrap()
                    .clone(),
            )
        };

        Self {
            test_runner,
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
            .mint_fungible(resource_address1, amount1.into())
            .mint_fungible(resource_address2, amount2.into())
            .take_all_from_worktop(resource_address1, |builder, bucket1| {
                builder.take_all_from_worktop(resource_address2, |builder, bucket2| {
                    builder.call_method(
                        self.pool_component_address,
                        TWO_RESOURCE_POOL_CONTRIBUTE_IDENT,
                        to_manifest_value_and_unwrap!(&TwoResourcePoolContributeManifestInput {
                            buckets: (bucket1, bucket2),
                        }),
                    )
                })
            })
            .try_deposit_batch_or_abort(self.account_component_address)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn redeem<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                self.account_component_address,
                self.pool_unit_resource_address,
                amount.into(),
            )
            .take_all_from_worktop(self.pool_unit_resource_address, |builder, bucket| {
                builder.call_method(
                    self.pool_component_address,
                    TWO_RESOURCE_POOL_REDEEM_IDENT,
                    to_manifest_value_and_unwrap!(&TwoResourcePoolRedeemManifestInput { bucket }),
                )
            })
            .try_deposit_batch_or_abort(self.account_component_address)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn set_pool_metadata<S: ToString>(
        &mut self,
        key: S,
        value: MetadataValue,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .set_metadata(self.pool_component_address, key, value)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn set_pool_unit_resource_metadata<S: ToString>(
        &mut self,
        key: S,
        value: MetadataValue,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .set_metadata(self.pool_unit_resource_address, key, value)
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
            .mint_fungible(resource_address, amount.into())
            .take_all_from_worktop(resource_address, |builder, bucket| {
                builder.call_method(
                    self.pool_component_address,
                    TWO_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    to_manifest_value_and_unwrap!(&TwoResourcePoolProtectedDepositManifestInput {
                        bucket
                    }),
                )
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        resource_address: ResourceAddress,
        amount: D,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                to_manifest_value_and_unwrap!(&TwoResourcePoolProtectedWithdrawManifestInput {
                    resource_address,
                    amount: amount.into(),
                }),
            )
            .try_deposit_batch_or_abort(self.account_component_address)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
        sign: bool,
    ) -> TransactionReceipt {
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, self.initial_proofs(sign))
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
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                to_manifest_value_and_unwrap!(&TwoResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                }),
            )
            .build();
        let receipt = self.execute_manifest(manifest, sign);
        receipt.expect_commit_success().output(1)
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
