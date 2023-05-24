use radix_engine::blueprints::pool::single_resource_pool::{
    SingleResourcePoolError, SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT,
};
use radix_engine::errors::{ApplicationError, RuntimeError};
use radix_engine::transaction::{BalanceChange, TransactionReceipt};
use radix_engine_interface::blueprints::pool::*;
use scrypto::prelude::*;
use scrypto_unit::*;
use transaction::builder::*;

#[test]
pub fn single_resource_pool_can_be_instantiated() {
    SingleResourcePoolTestRunner::new(18);
}

#[test]
pub fn initial_contribution_to_pool_mints_expected_amount() {
    // Arrange
    let mut test_runner = SingleResourcePoolTestRunner::new(18);

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
pub fn contribution_to_pool_mints_expected_amount_1() {
    // Arrange
    let mut test_runner = SingleResourcePoolTestRunner::new(18);

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
pub fn contribution_to_pool_mints_expected_amount_2() {
    // Arrange
    let mut test_runner = SingleResourcePoolTestRunner::new(18);

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
pub fn contribution_to_pool_mints_expected_amount_3() {
    // Arrange
    let mut test_runner = SingleResourcePoolTestRunner::new(18);

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
pub fn contribution_to_pool_mints_expected_amount_after_all_pool_units_are_redeemed() {
    // Arrange
    let mut test_runner = SingleResourcePoolTestRunner::new(18);
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

    assert_eq!(
        balance_change.clone(),
        BalanceChange::Fungible(initial_contribution.into())
    );
}

#[test]
pub fn redemption_of_pool_units_rounds_down_for_resources_with_divisibility_not_18() {
    // Arrange
    let divisibility = 2;
    let mut test_runner = SingleResourcePoolTestRunner::new(divisibility);

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
pub fn redeem_and_get_redemption_value_agree_on_amount_to_get_when_redeeming() {
    // Arrange
    let divisibility = 2;
    let mut test_runner = SingleResourcePoolTestRunner::new(divisibility);

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
pub fn creating_a_pool_with_non_fungible_resources_fails() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let (_, _, account) = test_runner.new_account(false);

    let non_fungible_resource = test_runner.create_non_fungible_resource(account);

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            POOL_PACKAGE,
            SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT,
            SINGLE_RESOURCE_POOL_INSTANTIATE_IDENT,
            to_manifest_value(&SingleResourcePoolInstantiateManifestInput {
                resource_address: non_fungible_resource,
                pool_manager_rule: rule!(allow_all),
            }),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(
        is_single_resource_pool_does_not_support_non_fungible_resources_error,
    )
}

//===================================
// Test Runner and Utility Functions
//===================================

struct SingleResourcePoolTestRunner {
    test_runner: TestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl SingleResourcePoolTestRunner {
    pub fn new(divisibility: u8) -> Self {
        let mut test_runner = TestRunner::builder().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            None,
            divisibility,
            account,
        );

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    SINGLE_RESOURCE_POOL_BLUEPRINT_IDENT,
                    SINGLE_RESOURCE_POOL_INSTANTIATE_IDENT,
                    to_manifest_value(&SingleResourcePoolInstantiateManifestInput {
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

    pub fn contribute<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .mint_fungible(self.resource_address, amount.into())
            .take_all_from_worktop(self.resource_address, |builder, bucket| {
                builder.call_method(
                    self.pool_component_address,
                    SINGLE_RESOURCE_POOL_CONTRIBUTE_IDENT,
                    to_manifest_value(&SingleResourcePoolContributeManifestInput { bucket }),
                )
            })
            .safe_deposit_batch(self.account_component_address)
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn redeem<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .withdraw_from_account(
                self.account_component_address,
                self.pool_unit_resource_address,
                amount.into(),
            )
            .take_all_from_worktop(self.pool_unit_resource_address, |builder, bucket| {
                builder.call_method(
                    self.pool_component_address,
                    SINGLE_RESOURCE_POOL_REDEEM_IDENT,
                    to_manifest_value(&SingleResourcePoolRedeemManifestInput { bucket }),
                )
            })
            .safe_deposit_batch(self.account_component_address)
            .build();
        self.execute_manifest(manifest, sign)
    }

    pub fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
    ) -> Decimal {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                SINGLE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                to_manifest_value(&SingleResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                }),
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

fn is_single_resource_pool_does_not_support_non_fungible_resources_error(
    runtime_error: &RuntimeError,
) -> bool {
    matches!(
        runtime_error,
        RuntimeError::ApplicationError(ApplicationError::SingleResourcePoolError(
            SingleResourcePoolError::PoolsDoNotSupportNonFungibleResources { .. }
        ))
    )
}
