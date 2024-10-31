use radix_common::prelude::*;
use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::blueprints::pool::v1::errors::multi_resource_pool::Error as MultiResourcePoolError;
use radix_engine::blueprints::pool::v1::events::multi_resource_pool::*;
use radix_engine::errors::{SystemError, SystemModuleError};
use radix_engine::{
    errors::{ApplicationError, RuntimeError},
    transaction::{BalanceChange, TransactionReceipt},
};
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::object_modules::metadata::MetadataValue;
use radix_engine_interface::prelude::*;
use radix_transactions::prelude::*;
use scrypto::prelude::Pow;
use scrypto_test::prelude::{is_auth_error, DefaultLedgerSimulator, LedgerSimulatorBuilder};

#[test]
fn multi_resource_pool_can_be_instantiated() {
    TestEnvironment::<3>::new([18, 18, 18]);
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
    let mut ledger = TestEnvironment::new_with_owner([18, 18, 18], owner_role);

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
fn contributing_provides_expected_amount_of_pool_units1() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("100"),
        ledger.pool_resources[1] => dec!("100"),
        ledger.pool_resources[2] => dec!("100")
    );

    let expected_change = indexmap!(
        ledger.pool_resources[0] => dec!("0"),
        ledger.pool_resources[1] => dec!("0"),
        ledger.pool_resources[2] => dec!("0")
    );
    let expected_pool_units = dec!("100");

    // Act
    let receipt = ledger.contribute(contributions, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );
    for (resource_address, amount) in expected_change.iter() {
        assert_eq!(
            account_balance_changes.get(resource_address).cloned(),
            if *amount == Decimal::ZERO {
                None
            } else {
                Some(BalanceChange::Fungible(*amount))
            }
        );
    }
    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units))
    );
}

#[test]
fn contributing_provides_expected_amount_of_pool_units2() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    {
        let contributions = indexmap!(
            ledger.pool_resources[0] => dec!("100"),
            ledger.pool_resources[1] => dec!("100"),
            ledger.pool_resources[2] => dec!("100")
        );
        ledger
            .contribute(contributions, true)
            .expect_commit_success();
    }

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("100"),
        ledger.pool_resources[1] => dec!("100"),
        ledger.pool_resources[2] => dec!("100")
    );

    let expected_change = indexmap!(
        ledger.pool_resources[0] => dec!("0"),
        ledger.pool_resources[1] => dec!("0"),
        ledger.pool_resources[2] => dec!("0")
    );
    let expected_pool_units = dec!("100");

    // Act
    let receipt = ledger.contribute(contributions, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );
    for (resource_address, amount) in expected_change.iter() {
        assert_eq!(
            account_balance_changes.get(resource_address).cloned(),
            if *amount == Decimal::ZERO {
                None
            } else {
                Some(BalanceChange::Fungible(*amount))
            }
        );
    }
    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units))
    );
}

#[test]
fn contributing_provides_expected_amount_of_pool_units3() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    {
        let contributions = indexmap!(
            ledger.pool_resources[0] => dec!("100"),
            ledger.pool_resources[1] => dec!("100"),
            ledger.pool_resources[2] => dec!("100")
        );
        ledger
            .contribute(contributions, true)
            .expect_commit_success();
    }

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("100"),
        ledger.pool_resources[1] => dec!("90"),
        ledger.pool_resources[2] => dec!("100")
    );

    let expected_change = indexmap!(
        ledger.pool_resources[0] => dec!("10"),
        ledger.pool_resources[1] => dec!("0"),
        ledger.pool_resources[2] => dec!("10")
    );
    let expected_pool_units = dec!("90");

    // Act
    let receipt = ledger.contribute(contributions, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );
    for (resource_address, amount) in expected_change.iter() {
        assert_eq!(
            account_balance_changes.get(resource_address).cloned(),
            if *amount == Decimal::ZERO {
                None
            } else {
                Some(BalanceChange::Fungible(*amount))
            }
        );
    }
    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units))
    );
}

#[test]
fn contributing_provides_expected_amount_of_pool_units4() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    {
        let contributions = indexmap!(
            ledger.pool_resources[0] => dec!("100"),
            ledger.pool_resources[1] => dec!("100"),
            ledger.pool_resources[2] => dec!("100")
        );
        ledger
            .contribute(contributions, true)
            .expect_commit_success();
    }

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("100"),
        ledger.pool_resources[1] => dec!("90"),
        ledger.pool_resources[2] => dec!("80")
    );

    let expected_change = indexmap!(
        ledger.pool_resources[0] => dec!("20"),
        ledger.pool_resources[1] => dec!("10"),
        ledger.pool_resources[2] => dec!("0")
    );
    let expected_pool_units = dec!("80");

    // Act
    let receipt = ledger.contribute(contributions, true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );
    for (resource_address, amount) in expected_change.iter() {
        assert_eq!(
            account_balance_changes.get(resource_address).cloned(),
            if *amount == Decimal::ZERO {
                None
            } else {
                Some(BalanceChange::Fungible(*amount))
            }
        );
    }
    assert_eq!(
        account_balance_changes
            .get(&ledger.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units))
    );
}

#[test]
fn initial_contribution_to_pool_check_amounts() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("10"),
        ledger.pool_resources[1] => dec!("10"),
        ledger.pool_resources[2] => dec!("10")
    );

    // Act
    ledger
        .contribute(contributions, true)
        .expect_commit_success();
    let amounts = ledger.get_vault_amounts(true);

    // Assert
    assert_eq!(amounts.len(), 3);
    for item in amounts.iter() {
        assert_eq!(*item.1, 10.into());
    }
}

#[test]
fn contributing_tokens_that_do_not_belong_to_pool_fails() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);
    let resource_address = ledger
        .ledger
        .create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            18,
            ledger.account_component_address,
        );

    let contributions = indexmap!(
        resource_address => dec!("100"),
        ledger.pool_resources[1] => dec!("100"),
        ledger.pool_resources[2] => dec!("100")
    );

    // Act
    let receipt = ledger.contribute(contributions, true);

    // Assert
    receipt
        .expect_specific_failure(is_multi_resource_pool_resource_does_not_belong_to_the_pool_error)
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
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
            MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
            MultiResourcePoolInstantiateManifestInput {
                resource_addresses: indexset!(non_fungible_resource.into()),
                pool_manager_rule: rule!(allow_all),
                owner_role: OwnerRole::None,
                address_reservation: None,
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(
        is_multi_resource_pool_does_non_fungible_resources_are_not_accepted,
    )
}

#[test]
fn redemption_of_pool_units_rounds_down_for_resources_with_divisibility_not_18() {
    // Arrange
    let mut ledger = TestEnvironment::<2>::new([18, 2]);

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("100"),
        ledger.pool_resources[1] => dec!("100"),
    );
    let expected_change = indexmap!(
        ledger.pool_resources[0] => dec!("1.11111111111111"),
        ledger.pool_resources[1] => dec!("1.11"),
    );
    ledger.contribute(contributions, true);

    // Act
    let receipt = ledger.get_redemption_value(dec!("1.11111111111111"), true);

    // Assert
    assert_eq!(
        receipt[&ledger.pool_resources[0]],
        expected_change[&ledger.pool_resources[0]]
    );
    assert_eq!(
        receipt[&ledger.pool_resources[1]],
        expected_change[&ledger.pool_resources[1]]
    );

    // Act
    let receipt = ledger.redeem(dec!("1.11111111111111"), true);

    // Assert
    let account_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.account_component_address.as_node_id(),
    );
    for (resource_address, amount) in expected_change.iter() {
        assert_eq!(
            account_balance_changes.get(resource_address).cloned(),
            if *amount == Decimal::ZERO {
                None
            } else {
                Some(BalanceChange::Fungible(*amount))
            }
        );
    }
}

#[test]
fn contribution_calculations_work_for_resources_with_divisibility_not_18() {
    // Arrange
    let mut ledger = TestEnvironment::<2>::new([18, 2]);

    {
        let contributions = indexmap!(
            ledger.pool_resources[0] => dec!("100"),
            ledger.pool_resources[1] => dec!("100"),
        );
        ledger
            .contribute(contributions, true)
            .expect_commit_success()
    };

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("1.1111111111111"),
        ledger.pool_resources[1] => dec!("500"),
    );

    // Act
    let receipt = ledger.contribute(contributions, true);

    // Assert
    let pool_balance_changes = ledger.ledger.sum_descendant_balance_changes(
        receipt.expect_commit_success(),
        ledger.pool_component_address.as_node_id(),
    );
    assert_eq!(
        pool_balance_changes.get(&ledger.pool_resources[0]).cloned(),
        Some(BalanceChange::Fungible(dec!("1.1111111111111")))
    );
    assert_eq!(
        pool_balance_changes.get(&ledger.pool_resources[1]).cloned(),
        Some(BalanceChange::Fungible(dec!("1.11")))
    );
}

#[test]
fn contribution_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::<2>::new([2, 2]);

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("2.22"),
        ledger.pool_resources[1] => dec!("8.88"),
    );

    // Act
    let receipt = ledger.contribute(contributions.clone(), true);

    // Assert
    let ContributionEvent {
        contributed_resources,
        pool_units_minted,
    } = receipt
        .expect_commit_success()
        .application_events
        .iter()
        .find_map(|(event_type_identifier, event_data)| {
            if ledger.ledger.event_name(event_type_identifier) == "ContributionEvent" {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(contributed_resources, contributions);
    assert_eq!(pool_units_minted, dec!("4.44"));
}

#[test]
fn redemption_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::<2>::new([2, 2]);

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("2.22"),
        ledger.pool_resources[1] => dec!("8.88"),
    );
    ledger
        .contribute(contributions.clone(), true)
        .expect_commit_success();

    // Act
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
            if ledger.ledger.event_name(event_type_identifier) == "RedemptionEvent" {
                Some(scrypto_decode(event_data).unwrap())
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(pool_unit_tokens_redeemed, dec!("4.44"));
    assert_eq!(redeemed_resources, contributions);
}

#[test]
fn deposits_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::<2>::new([2, 2]);

    // Act
    let receipt = ledger.protected_deposit(ledger.pool_resources[0], dec!("2.22"), true);

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
    assert_eq!(resource_address, ledger.pool_resources[0]);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraws_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::<2>::new([2, 2]);

    // Act
    ledger
        .protected_deposit(ledger.pool_resources[0], dec!("2.22"), true)
        .expect_commit_success();
    let receipt = ledger.protected_withdraw(
        ledger.pool_resources[0],
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
    assert_eq!(resource_address, ledger.pool_resources[0]);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraws_with_rounding_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::<2>::new([2, 2]);

    // Act
    ledger
        .protected_deposit(ledger.pool_resources[0], dec!("2.22"), true)
        .expect_commit_success();
    let receipt = ledger.protected_withdraw(
        ledger.pool_resources[0],
        dec!("2.211"),
        WithdrawStrategy::Rounded(RoundingMode::AwayFromZero),
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
    assert_eq!(resource_address, ledger.pool_resources[0]);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn cant_contribute_without_proper_signature() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    let contributions = indexmap!(
        ledger.pool_resources[0] => dec!("100"),
        ledger.pool_resources[1] => dec!("100"),
        ledger.pool_resources[2] => dec!("100")
    );

    // Act
    let receipt = ledger.contribute(contributions, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
fn cant_deposit_without_proper_signature() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    // Act
    let receipt = ledger.protected_deposit(ledger.pool_resources[0], 10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
fn cant_withdraw_without_proper_signature() {
    // Arrange
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    // Act
    ledger
        .protected_deposit(ledger.pool_resources[0], 10, true)
        .expect_commit_success();
    let receipt =
        ledger.protected_withdraw(ledger.pool_resources[0], 10, WithdrawStrategy::Exact, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
fn contribution_of_large_values_should_not_cause_panic() {
    // Arrange
    let max_mint_amount = Decimal::from_attos(I192::from(2).pow(152));
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);

    // Act
    let receipt = ledger.contribute(
        indexmap!(
            ledger.pool_resources[0] => max_mint_amount,
            ledger.pool_resources[1] => max_mint_amount,
            ledger.pool_resources[2] => max_mint_amount,
        ),
        true,
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn get_redemption_value_should_not_panic_on_large_values() {
    // Arrange
    let mint_amount = Decimal::from_attos(I192::from(2).pow(40));
    let mut ledger = TestEnvironment::<3>::new([18, 18, 18]);
    let receipt = ledger.contribute(
        indexmap!(
            ledger.pool_resources[0] => mint_amount,
            ledger.pool_resources[1] => mint_amount,
            ledger.pool_resources[2] => mint_amount,
        ),
        true,
    );
    receipt.expect_commit_success();

    // Act
    let receipt = ledger.call_get_redemption_value(Decimal::MAX, true);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
                MultiResourcePoolError::InvalidGetRedemptionAmount
            ))
        )
    });
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

struct TestEnvironment<const N: usize> {
    ledger: DefaultLedgerSimulator,

    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,

    pool_resources: [ResourceAddress; N],

    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl<const N: usize> TestEnvironment<N> {
    pub fn new(divisibility: [u8; N]) -> Self {
        Self::new_with_owner(divisibility, OwnerRole::None)
    }

    pub fn new_with_owner(divisibility: [u8; N], owner_role: OwnerRole) -> Self {
        let mut ledger = LedgerSimulatorBuilder::new().build();
        let (public_key, _, account) = ledger.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_addresses = divisibility.map(|divisibility| {
            ledger.create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                divisibility,
                account,
            )
        });

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    POOL_PACKAGE,
                    MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                    MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                    MultiResourcePoolInstantiateManifestInput {
                        resource_addresses: resource_addresses
                            .into_iter()
                            .map(Into::into)
                            .collect(),
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
            pool_resources: resource_addresses,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    pub fn contribute(
        &mut self,
        resource_to_amount_mapping: IndexMap<ResourceAddress, Decimal>,
        sign: bool,
    ) -> TransactionReceipt {
        let mut manifest_builder = ManifestBuilder::new().lock_fee_from_faucet();
        for (resource_address, amount) in resource_to_amount_mapping.iter() {
            manifest_builder = manifest_builder.mint_fungible(*resource_address, *amount)
        }
        let manifest = manifest_builder
            .call_method(
                self.pool_component_address,
                MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
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
            .take_all_from_worktop(self.pool_unit_resource_address, "pool_unit")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    MULTI_RESOURCE_POOL_REDEEM_IDENT,
                    MultiResourcePoolRedeemManifestInput {
                        bucket: lookup.bucket("pool_unit"),
                    },
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
            .take_all_from_worktop(resource_address, "to_deposit")
            .with_name_lookup(|builder, lookup| {
                let bucket = lookup.bucket("to_deposit");
                builder.call_method(
                    self.pool_component_address,
                    MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    MultiResourcePoolProtectedDepositManifestInput { bucket },
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
                MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                MultiResourcePoolProtectedWithdrawManifestInput {
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

    fn get_vault_amounts(&mut self, sign: bool) -> MultiResourcePoolGetVaultAmountsOutput {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.pool_component_address,
                MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
                MultiResourcePoolGetVaultAmountsManifestInput,
            )
            .build();
        let receipt = self.execute_manifest(manifest, sign);
        receipt.expect_commit_success().output(1)
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> MultiResourcePoolGetRedemptionValueOutput {
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
                MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                MultiResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                },
            )
            .build();
        self.execute_manifest(manifest, sign)
    }
}

fn is_multi_resource_pool_resource_does_not_belong_to_the_pool_error(
    runtime_error: &RuntimeError,
) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
            MultiResourcePoolError::ResourceDoesNotBelongToPool { .. }
        ))
    )
}

fn is_multi_resource_pool_does_non_fungible_resources_are_not_accepted(
    runtime_error: &RuntimeError,
) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::MultiResourcePoolError(
            MultiResourcePoolError::NonFungibleResourcesAreNotAccepted { .. }
        ))
    )
}
