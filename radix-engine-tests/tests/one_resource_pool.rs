use radix_engine::blueprints::pool::one_resource_pool::*;
use radix_engine::errors::{ApplicationError, RuntimeError, SystemError, SystemModuleError};
use radix_engine::transaction::{BalanceChange, TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::blueprints::pool::*;
use scrypto_unit::*;
use transaction::builder::*;

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
    let mut test_runner = TestEnvironment::new(18);

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
fn initial_contribution_to_pool_mints_expected_amount() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    let receipt = test_runner.contribute(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.pool_unit_resource_address)
        .unwrap();

    assert_eq!(balance_change.clone(), BalanceChange::Fungible(100.into()));
}

#[test]
fn initial_contribution_to_pool_check_amount() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    test_runner.contribute(10, true).expect_commit_success();
    let amount = test_runner.get_vault_amount(true);

    // Assert
    assert_eq!(amount, 10.into());
}

#[test]
fn contribution_to_pool_mints_expected_amount_1() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    let _ = test_runner.contribute(100, true);
    let receipt = test_runner.contribute(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.pool_unit_resource_address)
        .unwrap();

    assert_eq!(balance_change.clone(), BalanceChange::Fungible(100.into()));
}

#[test]
fn contribution_to_pool_mints_expected_amount_2() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    let _ = test_runner.contribute(100, true);
    let receipt = test_runner.contribute(200, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.pool_unit_resource_address)
        .unwrap();

    assert_eq!(balance_change.clone(), BalanceChange::Fungible(200.into()));
}

#[test]
fn contribution_to_pool_mints_expected_amount_3() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    let _ = test_runner.contribute(100, true);
    let receipt = test_runner.contribute(50, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.pool_unit_resource_address)
        .unwrap();

    assert_eq!(balance_change.clone(), BalanceChange::Fungible(50.into()));
}

#[test]
fn contribution_to_pool_mints_expected_amount_after_all_pool_units_are_redeemed() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);
    let initial_contribution = 100;

    // Act
    {
        test_runner
            .contribute(initial_contribution, true)
            .expect_commit_success();
        test_runner
            .redeem(initial_contribution, true)
            .expect_commit_success();
    };
    let receipt = test_runner.contribute(50, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.pool_unit_resource_address)
        .unwrap();

    assert_eq!(balance_change.clone(), BalanceChange::Fungible(50.into()));
}

#[test]
fn redemption_of_pool_units_rounds_down_for_resources_with_divisibility_not_18() {
    // Arrange
    let divisibility = 2;
    let mut test_runner = TestEnvironment::new(divisibility);

    test_runner.contribute(100, true).expect_commit_success();

    // Act
    let receipt = test_runner.redeem(dec!("1.111111111111"), true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.resource_address)
        .unwrap();

    assert_eq!(
        balance_change.clone(),
        BalanceChange::Fungible(dec!("1.11"))
    );
}

#[test]
fn redeem_and_get_redemption_value_agree_on_amount_to_get_when_redeeming() {
    // Arrange
    let divisibility = 2;
    let mut test_runner = TestEnvironment::new(divisibility);

    test_runner.contribute(100, true).expect_commit_success();

    // Act
    let receipt = test_runner.redeem(dec!("1.111111111111"), true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.resource_address)
        .unwrap();

    assert_eq!(
        balance_change.clone(),
        BalanceChange::Fungible(test_runner.get_redemption_value(dec!("1.111111111111"), false))
    );
}

#[test]
fn redeem_and_get_redemption_value_agree_on_amount_to_get_when_redeeming_after_protected_withdraws_and_deposits(
) {
    // Arrange
    let divisibility = 2;
    let amount_to_redeem = dec!("1.111111111111");
    let mut test_runner = TestEnvironment::new(divisibility);

    test_runner.contribute(100, true).expect_commit_success();

    // Act
    test_runner
        .protected_deposit(50, true)
        .expect_commit_success();
    test_runner
        .protected_withdraw(20, true)
        .expect_commit_success();
    let receipt = test_runner.redeem(amount_to_redeem, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.resource_address)
        .unwrap();

    assert_eq!(
        balance_change.clone(),
        BalanceChange::Fungible(test_runner.get_redemption_value(amount_to_redeem, false))
    );
}

#[test]
fn protected_withdraw_from_the_pool_lowers_how_much_resources_the_pool_units_are_redeemable_for() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    test_runner.contribute(100, true).expect_commit_success();

    // Act
    test_runner.protected_withdraw(50, true);
    let receipt = test_runner.redeem(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.resource_address)
        .unwrap();

    assert_eq!(balance_change.clone(), BalanceChange::Fungible(50.into()));
}

#[test]
fn protected_deposit_into_the_pool_increases_how_much_resources_the_pool_units_are_redeemable_for()
{
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    test_runner.contribute(100, true).expect_commit_success();

    // Act
    test_runner.protected_deposit(50, true);
    let receipt = test_runner.redeem(100, true);

    // Assert
    let commit_result = receipt.expect_commit_success();
    let balance_change = commit_result
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap()
        .get(&test_runner.resource_address)
        .unwrap();

    assert_eq!(balance_change.clone(), BalanceChange::Fungible(150.into()));
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
            ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
            ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
            to_manifest_value_and_unwrap!(&OneResourcePoolInstantiateManifestInput {
                resource_address: non_fungible_resource,
                pool_manager_rule: rule!(allow_all),
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt
        .expect_specific_failure(is_one_resource_pool_does_non_fungible_resources_are_not_accepted)
}

#[test]
fn contribution_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new(2);

    // Act
    let receipt = test_runner.contribute(dec!("2.22"), true);

    // Assert
    let ContributionEvent {
        amount_of_resources_contributed,
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
    assert_eq!(amount_of_resources_contributed, dec!("2.22"));
    assert_eq!(pool_units_minted, dec!("2.22"));
}

#[test]
fn redemption_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new(2);

    // Act
    test_runner
        .contribute(dec!("2.22"), true)
        .expect_commit_success();
    let receipt = test_runner.redeem(dec!("2.22"), true);

    // Assert
    let RedemptionEvent {
        pool_unit_tokens_redeemed,
        redeemed_amount,
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
    assert_eq!(pool_unit_tokens_redeemed, dec!("2.22"));
    assert_eq!(redeemed_amount, dec!("2.22"));
}

#[test]
fn deposits_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new(2);

    // Act
    let receipt = test_runner.protected_deposit(dec!("2.22"), true);

    // Assert
    let DepositEvent { amount } = receipt
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
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraw_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::new(2);

    // Act
    test_runner
        .protected_deposit(dec!("2.22"), true)
        .expect_commit_success();
    let receipt = test_runner.protected_withdraw(dec!("2.22"), true);

    // Assert
    let WithdrawEvent { amount } = receipt
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
    assert_eq!(amount, dec!("2.22"));
}

#[test]
pub fn protected_deposit_fails_without_proper_authority_present() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    let receipt = test_runner.protected_deposit(10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn protected_withdraw_fails_without_proper_authority_present() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    test_runner
        .protected_deposit(10, true)
        .expect_commit_success();
    let receipt = test_runner.protected_withdraw(10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
pub fn contribute_fails_without_proper_authority_present() {
    // Arrange
    let mut test_runner = TestEnvironment::new(18);

    // Act
    let receipt = test_runner.contribute(10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

//===================================
// Test Runner and Utility Functions
//===================================

struct TestEnvironment {
    test_runner: TestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl TestEnvironment {
    fn new(divisibility: u8) -> Self {
        let mut test_runner = TestRunner::builder().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility,
            account,
        );

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    ONE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    ONE_RESOURCE_POOL_INSTANTIATE_IDENT,
                    to_manifest_value_and_unwrap!(&OneResourcePoolInstantiateManifestInput {
                        resource_address,
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
            resource_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
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

    fn contribute<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .mint_fungible(self.resource_address, amount.into())
            .take_all_from_worktop(self.resource_address, |builder, bucket| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    to_manifest_value_and_unwrap!(&OneResourcePoolContributeManifestInput {
                        bucket
                    }),
                )
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
                    ONE_RESOURCE_POOL_REDEEM_IDENT,
                    to_manifest_value_and_unwrap!(&OneResourcePoolRedeemManifestInput { bucket }),
                )
            })
            .try_deposit_batch_or_abort(self.account_component_address)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_deposit<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .mint_fungible(self.resource_address, amount.into())
            .take_all_from_worktop(self.resource_address, |builder, bucket| {
                builder.call_method(
                    self.pool_component_address,
                    ONE_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    to_manifest_value_and_unwrap!(&OneResourcePoolProtectedDepositManifestInput {
                        bucket
                    }),
                )
            })
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        amount: D,
        sign: bool,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                to_manifest_value_and_unwrap!(&OneResourcePoolProtectedWithdrawManifestInput {
                    amount: amount.into(),
                }),
            )
            .try_deposit_batch_or_abort(self.account_component_address)
            .build();
        self.execute_manifest(manifest, sign)
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> Decimal {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                to_manifest_value_and_unwrap!(&OneResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                }),
            )
            .build();
        let receipt = self.execute_manifest(manifest, sign);
        receipt.expect_commit_success().output(1)
    }

    fn get_vault_amount(&mut self, sign: bool) -> Decimal {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_GET_VAULT_AMOUNT_IDENT,
                to_manifest_value_and_unwrap!(&OneResourcePoolGetVaultAmountManifestInput),
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
