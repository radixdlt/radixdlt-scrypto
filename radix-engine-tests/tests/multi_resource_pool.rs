use radix_engine::errors::{SystemError, SystemModuleError};
use radix_engine::{
    blueprints::pool::multi_resource_pool::*,
    errors::{ApplicationError, RuntimeError},
    transaction::{BalanceChange, TransactionReceipt},
    types::*,
};
use radix_engine_interface::api::node_modules::metadata::MetadataValue;
use radix_engine_interface::blueprints::pool::*;
use scrypto_unit::{is_auth_error, TestRunner};
use transaction::prelude::{ManifestBuilder, TransactionManifestV1};

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
    let mut test_runner = TestEnvironment::new([18, 18, 18]);

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
fn contributing_provides_expected_amount_of_pool_units1() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("100"),
        test_runner.pool_resources[1] => dec!("100"),
        test_runner.pool_resources[2] => dec!("100")
    );

    let expected_change = btreemap!(
        test_runner.pool_resources[0] => dec!("0"),
        test_runner.pool_resources[1] => dec!("0"),
        test_runner.pool_resources[2] => dec!("0")
    );
    let expected_pool_units = dec!("1000");

    // Act
    let receipt = test_runner.contribute(contributions, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();
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
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
}

#[test]
fn contributing_provides_expected_amount_of_pool_units2() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    {
        let contributions = btreemap!(
            test_runner.pool_resources[0] => dec!("100"),
            test_runner.pool_resources[1] => dec!("100"),
            test_runner.pool_resources[2] => dec!("100")
        );
        test_runner
            .contribute(contributions, true)
            .expect_commit_success();
    }

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("100"),
        test_runner.pool_resources[1] => dec!("100"),
        test_runner.pool_resources[2] => dec!("100")
    );

    let expected_change = btreemap!(
        test_runner.pool_resources[0] => dec!("0"),
        test_runner.pool_resources[1] => dec!("0"),
        test_runner.pool_resources[2] => dec!("0")
    );
    let expected_pool_units = dec!("1000");

    // Act
    let receipt = test_runner.contribute(contributions, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();
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
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
}

#[test]
fn contributing_provides_expected_amount_of_pool_units3() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    {
        let contributions = btreemap!(
            test_runner.pool_resources[0] => dec!("100"),
            test_runner.pool_resources[1] => dec!("100"),
            test_runner.pool_resources[2] => dec!("100")
        );
        test_runner
            .contribute(contributions, true)
            .expect_commit_success();
    }

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("100"),
        test_runner.pool_resources[1] => dec!("90"),
        test_runner.pool_resources[2] => dec!("100")
    );

    let expected_change = btreemap!(
        test_runner.pool_resources[0] => dec!("10"),
        test_runner.pool_resources[1] => dec!("0"),
        test_runner.pool_resources[2] => dec!("10")
    );
    let expected_pool_units = dec!("900");

    // Act
    let receipt = test_runner.contribute(contributions, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();
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
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
}

#[test]
fn contributing_provides_expected_amount_of_pool_units4() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    {
        let contributions = btreemap!(
            test_runner.pool_resources[0] => dec!("100"),
            test_runner.pool_resources[1] => dec!("100"),
            test_runner.pool_resources[2] => dec!("100")
        );
        test_runner
            .contribute(contributions, true)
            .expect_commit_success();
    }

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("100"),
        test_runner.pool_resources[1] => dec!("90"),
        test_runner.pool_resources[2] => dec!("80")
    );

    let expected_change = btreemap!(
        test_runner.pool_resources[0] => dec!("20"),
        test_runner.pool_resources[1] => dec!("10"),
        test_runner.pool_resources[2] => dec!("0")
    );
    let expected_pool_units = dec!("800");

    // Act
    let receipt = test_runner.contribute(contributions, true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();
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
            .get(&test_runner.pool_unit_resource_address)
            .cloned(),
        Some(BalanceChange::Fungible(expected_pool_units.into()))
    );
}

#[test]
fn initial_contribution_to_pool_check_amounts() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("10"),
        test_runner.pool_resources[1] => dec!("10"),
        test_runner.pool_resources[2] => dec!("10")
    );

    // Act
    test_runner
        .contribute(contributions, true)
        .expect_commit_success();
    let amounts = test_runner.get_vault_amounts(true);

    // Assert
    assert_eq!(amounts.len(), 3);
    for item in amounts.iter() {
        assert_eq!(*item.1, 10.into());
    }
}

#[test]
fn contributing_tokens_that_do_not_belong_to_pool_fails() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);
    let resource_address = test_runner
        .test_runner
        .create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            18,
            test_runner.account_component_address,
        );

    let contributions = btreemap!(
        resource_address => dec!("100"),
        test_runner.pool_resources[1] => dec!("100"),
        test_runner.pool_resources[2] => dec!("100")
    );

    // Act
    let receipt = test_runner.contribute(contributions, true);

    // Assert
    receipt
        .expect_specific_failure(is_multi_resource_pool_resource_does_not_belong_to_the_pool_error)
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
            MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
            MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
            to_manifest_value_and_unwrap!(&MultiResourcePoolInstantiateManifestInput {
                resource_addresses: [non_fungible_resource].into(),
                pool_manager_rule: rule!(allow_all),
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(
        is_multi_resource_pool_does_non_fungible_resources_are_not_accepted,
    )
}

#[test]
fn redemption_of_pool_units_rounds_down_for_resources_with_divisibility_not_18() {
    // Arrange
    let mut test_runner = TestEnvironment::<2>::new([18, 2]);

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("100"),
        test_runner.pool_resources[1] => dec!("100"),
    );
    let expected_change = btreemap!(
        test_runner.pool_resources[0] => dec!("1.11111111111111"),
        test_runner.pool_resources[1] => dec!("1.11"),
    );
    test_runner.contribute(contributions, true);

    // Act
    let receipt = test_runner.get_redemption_value(dec!("1.11111111111111"), true);

    // Assert
    assert_eq!(
        receipt[&test_runner.pool_resources[0]],
        expected_change[&test_runner.pool_resources[0]]
    );
    assert_eq!(
        receipt[&test_runner.pool_resources[1]],
        expected_change[&test_runner.pool_resources[1]]
    );

    // Act
    let receipt = test_runner.redeem(dec!("1.11111111111111"), true);

    // Assert
    let account_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.account_component_address))
        .unwrap();
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
    let mut test_runner = TestEnvironment::<2>::new([18, 2]);

    {
        let contributions = btreemap!(
            test_runner.pool_resources[0] => dec!("100"),
            test_runner.pool_resources[1] => dec!("100"),
        );
        test_runner
            .contribute(contributions, true)
            .expect_commit_success()
    };

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("1.1111111111111"),
        test_runner.pool_resources[1] => dec!("500"),
    );

    // Act
    let receipt = test_runner.contribute(contributions, true);

    // Assert
    let pool_balance_changes = receipt
        .expect_commit_success()
        .balance_changes()
        .get(&GlobalAddress::from(test_runner.pool_component_address))
        .unwrap();
    assert_eq!(
        pool_balance_changes
            .get(&test_runner.pool_resources[0])
            .cloned(),
        Some(BalanceChange::Fungible(dec!("1.1111111111111")))
    );
    assert_eq!(
        pool_balance_changes
            .get(&test_runner.pool_resources[1])
            .cloned(),
        Some(BalanceChange::Fungible(dec!("1.11")))
    );
}

#[test]
fn contribution_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::<2>::new([2, 2]);

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("2.22"),
        test_runner.pool_resources[1] => dec!("8.88"),
    );

    // Act
    let receipt = test_runner.contribute(contributions.clone(), true);

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
    assert_eq!(contributed_resources, contributions);
    assert_eq!(pool_units_minted, dec!("4.44"));
}

#[test]
fn redemption_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::<2>::new([2, 2]);

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("2.22"),
        test_runner.pool_resources[1] => dec!("8.88"),
    );
    test_runner
        .contribute(contributions.clone(), true)
        .expect_commit_success();

    // Act
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
    assert_eq!(redeemed_resources, contributions);
}

#[test]
fn deposits_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::<2>::new([2, 2]);

    // Act
    let receipt = test_runner.protected_deposit(test_runner.pool_resources[0], dec!("2.22"), true);

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
    assert_eq!(resource_address, test_runner.pool_resources[0]);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn withdraws_emits_expected_event() {
    // Arrange
    let mut test_runner = TestEnvironment::<2>::new([2, 2]);

    // Act
    test_runner
        .protected_deposit(test_runner.pool_resources[0], dec!("2.22"), true)
        .expect_commit_success();
    let receipt = test_runner.protected_withdraw(test_runner.pool_resources[0], dec!("2.22"), true);

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
    assert_eq!(resource_address, test_runner.pool_resources[0]);
    assert_eq!(amount, dec!("2.22"));
}

#[test]
fn cant_contribute_without_proper_signature() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    let contributions = btreemap!(
        test_runner.pool_resources[0] => dec!("100"),
        test_runner.pool_resources[1] => dec!("100"),
        test_runner.pool_resources[2] => dec!("100")
    );

    // Act
    let receipt = test_runner.contribute(contributions, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
fn cant_deposit_without_proper_signature() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    // Act
    let receipt = test_runner.protected_deposit(test_runner.pool_resources[0], 10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

#[test]
fn cant_withdraw_without_proper_signature() {
    // Arrange
    let mut test_runner = TestEnvironment::<3>::new([18, 18, 18]);

    // Act
    test_runner
        .protected_deposit(test_runner.pool_resources[0], 10, true)
        .expect_commit_success();
    let receipt = test_runner.protected_withdraw(test_runner.pool_resources[0], 10, false);

    // Assert
    receipt.expect_specific_failure(is_auth_error)
}

struct TestEnvironment<const N: usize> {
    test_runner: TestRunner,

    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,

    pool_resources: [ResourceAddress; N],

    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl<const N: usize> TestEnvironment<N> {
    pub fn new(divisibility: [u8; N]) -> Self {
        let mut test_runner = TestRunner::builder().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_addresses = divisibility.map(|divisibility| {
            test_runner.create_freely_mintable_and_burnable_fungible_resource(
                OwnerRole::None,
                None,
                divisibility,
                account,
            )
        });

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                    MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                    to_manifest_value_and_unwrap!(&MultiResourcePoolInstantiateManifestInput {
                        resource_addresses: resource_addresses.clone().into(),
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
            pool_resources: resource_addresses,
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

    pub fn contribute(
        &mut self,
        resource_to_amount_mapping: BTreeMap<ResourceAddress, Decimal>,
        sign: bool,
    ) -> TransactionReceipt {
        let mut manifest_builder = &mut ManifestBuilder::new();
        for (resource_address, amount) in resource_to_amount_mapping.iter() {
            manifest_builder = manifest_builder.mint_fungible(*resource_address, *amount)
        }
        let manifest = manifest_builder
            .call_method(
                self.pool_component_address,
                MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
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
                    MULTI_RESOURCE_POOL_REDEEM_IDENT,
                    to_manifest_value_and_unwrap!(&MultiResourcePoolRedeemManifestInput { bucket }),
                )
            })
            .try_deposit_batch_or_abort(self.account_component_address)
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
                    MULTI_RESOURCE_POOL_PROTECTED_DEPOSIT_IDENT,
                    to_manifest_value_and_unwrap!(
                        &MultiResourcePoolProtectedDepositManifestInput { bucket }
                    ),
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
                MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                to_manifest_value_and_unwrap!(&MultiResourcePoolProtectedWithdrawManifestInput {
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

    fn get_vault_amounts(&mut self, sign: bool) -> MultiResourcePoolGetVaultAmountsOutput {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                MULTI_RESOURCE_POOL_GET_VAULT_AMOUNTS_IDENT,
                to_manifest_value_and_unwrap!(&MultiResourcePoolGetVaultAmountsManifestInput),
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
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                to_manifest_value_and_unwrap!(&MultiResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                }),
            )
            .build();
        let receipt = self.execute_manifest(manifest, sign);
        receipt.expect_commit_success().output(1)
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
