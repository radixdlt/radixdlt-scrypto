use rand_chacha::rand_core::{RngCore};
use radix_engine::blueprints::pool::two_resource_pool::TWO_RESOURCE_POOL_BLUEPRINT_IDENT;
use radix_engine::transaction::{TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use resource_tests::ResourceTestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_two_pool() {
    let mut two_pool_test = TwoPoolTest::new((7, 18));
    let mut fuzzer = ResourceTestFuzzer::new();

    for _ in 0..5000 {
        match fuzzer.next_u32(8u32) {
            0u32 => {
                two_pool_test.contribute(
                    (two_pool_test.pool_resource1, fuzzer.next_amount()),
                    (two_pool_test.pool_resource2, fuzzer.next_amount()),
                )
            },
            1u32 => {
                two_pool_test.contribute(
                    (two_pool_test.pool_resource2, fuzzer.next_amount()),
                    (two_pool_test.pool_resource1, fuzzer.next_amount()),
                )
            }
            2u32 => two_pool_test.protected_deposit(two_pool_test.pool_resource1, fuzzer.next_amount()),
            3u32 => two_pool_test.protected_deposit(two_pool_test.pool_resource2, fuzzer.next_amount()),
            4u32 => two_pool_test.protected_withdraw(two_pool_test.pool_resource1, fuzzer.next_amount(), fuzzer.next_withdraw_strategy()),
            5u32 => two_pool_test.protected_withdraw(two_pool_test.pool_resource2, fuzzer.next_amount(), fuzzer.next_withdraw_strategy()),
            6u32 => two_pool_test.redeem(fuzzer.next_amount()),
            _ => two_pool_test.get_redemption_value(fuzzer.next_amount()),
        };
    }
}

struct TwoPoolTest {
    test_runner: DefaultTestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    pool_resource1: ResourceAddress,
    pool_resource2: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl TwoPoolTest {
    pub fn new((divisibility1, divisibility2): (u8, u8)) -> Self {
        Self::new_with_owner((divisibility1, divisibility2), OwnerRole::None)
    }

    pub fn new_with_owner((divisibility1, divisibility2): (u8, u8), owner_role: OwnerRole) -> Self {
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
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
                    TwoResourcePoolInstantiateManifestInput {
                        resource_addresses: (pool_resource1, pool_resource2),
                        pool_manager_rule: rule!(require(virtual_signature_badge)),
                        owner_role,
                        address_reservation: None,
                    },
                )
                .build();
            let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);
            let commit_result = receipt.expect_commit_success();

            (
                commit_result.new_component_addresses()[0],
                commit_result.new_resource_addresses()[0],
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
    ) -> TransactionReceipt
        where
            A: Into<Decimal>,
            B: Into<Decimal>,
    {
        let manifest = ManifestBuilder::new()
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
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest)
    }

    fn redeem<D: Into<Decimal>>(&mut self, amount: D) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
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
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest)
    }

    fn protected_deposit<D: Into<Decimal>>(
        &mut self,
        resource_address: ResourceAddress,
        amount: D,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
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
        self.execute_manifest(manifest)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        resource_address: ResourceAddress,
        amount: D,
        withdraw_strategy: WithdrawStrategy,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                TWO_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                TwoResourcePoolProtectedWithdrawManifestInput {
                    resource_address,
                    amount: amount.into(),
                    withdraw_strategy,
                },
            )
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest)
    }

    fn execute_manifest(
        &mut self,
        manifest: TransactionManifestV1,
    ) -> TransactionReceipt {
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, self.initial_proofs())
    }

    fn virtual_signature_badge(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::from_public_key(&self.account_public_key)
    }

    fn initial_proofs(&self) -> Vec<NonFungibleGlobalId> {
        vec![self.virtual_signature_badge()]
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                TWO_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                TwoResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                },
            )
            .build();
        self.execute_manifest(manifest)
    }
}