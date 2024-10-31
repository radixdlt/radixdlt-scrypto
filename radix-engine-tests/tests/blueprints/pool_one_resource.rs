use radix_common::prelude::*;
use radix_engine::blueprints::pool::v1::constants::*;
use radix_engine::blueprints::pool::v1::errors::one_resource_pool::Error as OneResourcePoolError;
use radix_engine::blueprints::pool::v1::events::one_resource_pool::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine::transaction::{BalanceChange, TransactionReceipt};
use radix_engine_interface::api::ModuleId;
use radix_engine_interface::blueprints::pool::*;
use radix_engine_interface::object_modules::metadata::MetadataValue;
use scrypto::prelude::Pow;
use scrypto_test::prelude::*;

#[test]
fn one_resource_pool_can_be_instantiated() {
    TestEnvironment::new(18);
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
    let mut ledger = TestEnvironment::new_with_owner(18, owner_role);

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
fn initial_contribution_to_pool_mints_expected_amount() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    let receipt = ledger.contribute(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.pool_unit_resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(100.into()));
}

#[test]
fn initial_contribution_to_pool_check_amount() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    ledger.contribute(10, true).expect_commit_success();
    let amount = ledger.get_vault_amount(true);

    // Assert
    assert_eq!(amount, 10.into());
}

#[test]
fn pool_check_debug_output() {
    // Arrange
    let input_args_empty = OneResourcePoolGetVaultAmountManifestInput;
    let _debug_fmt_coverage = format!("{:?}", input_args_empty);

    let input_args = OneResourcePoolProtectedWithdrawManifestInput {
        amount: 10.into(),
        withdraw_strategy: WithdrawStrategy::Rounded(RoundingMode::ToZero),
    };
    let _debug_fmt_coverage = format!("{:?}", input_args);
}

#[test]
fn contribution_to_pool_mints_expected_amount_1() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    let _ = ledger.contribute(100, true);
    let receipt = ledger.contribute(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.pool_unit_resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(100.into()));
}

#[test]
fn contribution_to_pool_mints_expected_amount_2() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    let _ = ledger.contribute(100, true);
    let receipt = ledger.contribute(200, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.pool_unit_resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(200.into()));
}

#[test]
fn contribution_to_pool_mints_expected_amount_3() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    let _ = ledger.contribute(100, true);
    let receipt = ledger.contribute(50, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.pool_unit_resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(50.into()));
}

#[test]
fn contribution_to_pool_mints_expected_amount_after_all_pool_units_are_redeemed() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);
    let initial_contribution = 100;

    // Act
    {
        ledger
            .contribute(initial_contribution, true)
            .expect_commit_success();
        ledger
            .redeem(initial_contribution, true)
            .expect_commit_success();
    };
    let receipt = ledger.contribute(50, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.pool_unit_resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(50.into()));
}

#[test]
fn redemption_of_pool_units_rounds_down_for_resources_with_divisibility_not_18() {
    // Arrange
    let divisibility = 2;
    let mut ledger = TestEnvironment::new(divisibility);

    ledger.contribute(100, true).expect_commit_success();

    // Act
    let receipt = ledger.redeem(dec!("1.111111111111"), true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(dec!("1.11")));
}

#[test]
fn redeem_and_get_redemption_value_agree_on_amount_to_get_when_redeeming() {
    // Arrange
    let divisibility = 2;
    let mut ledger = TestEnvironment::new(divisibility);

    ledger.contribute(100, true).expect_commit_success();

    // Act
    let receipt = ledger.redeem(dec!("1.111111111111"), true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.resource_address)
        .unwrap()
        .clone();

    assert_eq!(
        balance_change,
        BalanceChange::Fungible(ledger.get_redemption_value(dec!("1.111111111111"), false))
    );
}

#[test]
fn redeem_and_get_redemption_value_agree_on_amount_to_get_when_redeeming_after_protected_withdraws_and_deposits(
) {
    // Arrange
    let divisibility = 2;
    let amount_to_redeem = dec!("1.111111111111");
    let mut ledger = TestEnvironment::new(divisibility);

    ledger.contribute(100, true).expect_commit_success();

    // Act
    ledger.protected_deposit(50, true).expect_commit_success();
    ledger
        .protected_withdraw(20, WithdrawStrategy::Exact, true)
        .expect_commit_success();
    let receipt = ledger.redeem(amount_to_redeem, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.resource_address)
        .unwrap()
        .clone();

    assert_eq!(
        balance_change,
        BalanceChange::Fungible(ledger.get_redemption_value(amount_to_redeem, false))
    );
}

#[test]
fn protected_withdraw_from_the_pool_lowers_how_much_resources_the_pool_units_are_redeemable_for() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    ledger.contribute(100, true).expect_commit_success();

    // Act
    ledger.protected_withdraw(50, WithdrawStrategy::Exact, true);
    let receipt = ledger.redeem(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(50.into()));
}

#[test]
fn protected_deposit_into_the_pool_increases_how_much_resources_the_pool_units_are_redeemable_for()
{
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    ledger.contribute(100, true).expect_commit_success();

    // Act
    ledger.protected_deposit(50, true);
    let receipt = ledger.redeem(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = ledger
        .ledger
        .sum_descendant_balance_changes(
            commit_result,
            ledger.account_component_address.as_node_id(),
        )
        .get(&ledger.resource_address)
        .unwrap()
        .clone();

    assert_eq!(balance_change, BalanceChange::Fungible(150.into()));
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
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
            ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
            OneResourcePoolInstantiateManifestInput {
                resource_address: non_fungible_resource.into(),
                pool_manager_rule: rule!(allow_all),
                owner_role: OwnerRole::None,
                address_reservation: None,
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt
        .expect_specific_failure(is_one_resource_pool_does_non_fungible_resources_are_not_accepted)
}

#[test]
fn contribution_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new(2);

    // Act
    let receipt = ledger.contribute(dec!("2.22"), true);

    // Assert
    let ContributionEvent {
        amount_of_resources_contributed,
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
    assert_eq!(amount_of_resources_contributed, dec!("2.22"));
    assert_eq!(pool_units_minted, dec!("2.22"));
}

#[test]
fn redemption_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new(2);

    // Act
    ledger
        .contribute(dec!("2.22"), true)
        .expect_commit_success();
    let receipt = ledger.redeem(dec!("2.22"), true);

    // Assert
    let RedemptionEvent {
        pool_unit_tokens_redeemed,
        redeemed_amount,
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
    assert_eq!(pool_unit_tokens_redeemed, dec!("2.22"));
    assert_eq!(redeemed_amount, dec!("2.22"));
}

#[test]
fn deposits_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new(2);

    // Act
    let receipt = ledger.protected_deposit(dec!("2.22"), true);

    // Assert
    let DepositEvent { amount } = receipt
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
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraw_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new(2);

    // Act
    ledger
        .protected_deposit(dec!("2.22"), true)
        .expect_commit_success();
    let receipt = ledger.protected_withdraw(dec!("2.22"), WithdrawStrategy::Exact, true);

    // Assert
    let WithdrawEvent { amount } = receipt
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
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraw_with_rounding_emits_expected_event() {
    // Arrange
    let mut ledger = TestEnvironment::new(2);

    // Act
    ledger
        .protected_deposit(dec!("2.22"), true)
        .expect_commit_success();
    let receipt = ledger.protected_withdraw(
        dec!("2.2211"),
        WithdrawStrategy::Rounded(RoundingMode::ToZero),
        true,
    );

    // Assert
    let WithdrawEvent { amount } = receipt
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
    assert_eq!(amount, dec!("2.22"));
}

#[test]
pub fn protected_deposit_fails_without_proper_authority_present() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    let receipt = ledger.protected_deposit(10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn protected_withdraw_fails_without_proper_authority_present() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    ledger.protected_deposit(10, true).expect_commit_success();
    let receipt = ledger.protected_withdraw(10, WithdrawStrategy::Exact, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn contribute_fails_without_proper_authority_present() {
    // Arrange
    let mut ledger = TestEnvironment::new(18);

    // Act
    let receipt = ledger.contribute(10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn owner_can_update_pool_metadata() {
    // Arrange
}

#[test]
fn get_redemption_value_should_not_panic_on_large_values() {
    // Arrange
    let max_mint_amount = Decimal::from_attos(I192::from(2).pow(152));
    let mut ledger = TestEnvironment::new(18);
    let receipt = ledger.contribute(max_mint_amount, true);
    receipt.expect_commit_success();

    // Act
    let receipt = ledger.call_get_redemption_value(Decimal::MAX, true);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::OneResourcePoolError(
                OneResourcePoolError::InvalidGetRedemptionAmount
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

//===================================
// Test Runner and Utility Functions
//===================================

struct TestEnvironment {
    ledger: DefaultLedgerSimulator,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl TestEnvironment {
    fn new(divisibility: u8) -> Self {
        Self::new_with_owner(divisibility, OwnerRole::None)
    }

    fn new_with_owner(divisibility: u8, owner_role: OwnerRole) -> Self {
        let mut ledger = LedgerSimulatorBuilder::new().build();
        let (public_key, _, account) = ledger.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = ledger.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility,
            account,
        );

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_function(
                    POOL_PACKAGE,
                    ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
                    OneResourcePoolInstantiateManifestInput {
                        resource_address: resource_address.into(),
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
            resource_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn contribute<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(self.resource_address, amount.into())
            .take_all_from_worktop(self.resource_address, "contribution")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    OneResourcePoolContributeManifestInput {
                        bucket: lookup.bucket("contribution"),
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
            .take_all_from_worktop(self.pool_unit_resource_address, "pool_unit")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_REDEEM_IDENT,
                    OneResourcePoolRedeemManifestInput {
                        bucket: lookup.bucket("pool_unit"),
                    },
                )
            })
            .try_deposit_entire_worktop_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_deposit<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(self.resource_address, amount.into())
            .take_all_from_worktop(self.resource_address, "to_deposit")
            .with_name_lookup(|builder, lookup| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    OneResourcePoolProtectedDepositManifestInput {
                        bucket: lookup.bucket("to_deposit"),
                    },
                )
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        amount: D,
        withdraw_strategy: WithdrawStrategy,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                OneResourcePoolProtectedWithdrawManifestInput {
                    amount: amount.into(),
                    withdraw_strategy,
                },
            )
            .try_deposit_entire_worktop_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> Decimal {
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
                ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                OneResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                },
            )
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn get_vault_amount(&mut self, sign: bool) -> Decimal {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT,
                OneResourcePoolGetVaultAmountManifestInput,
            )
            .build();
        let receipt = self.execute_manifest(manifest, sign);
        receipt.expect_commit_success().output(1)
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
}

fn is_one_resource_pool_does_non_fungible_resources_are_not_accepted(
    runtime_error: &RuntimeError,
) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::OneResourcePoolError(
            OneResourcePoolError::NonFungibleResourcesAreNotAccepted { .. }
        ))
    )
}
