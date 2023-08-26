use rand_chacha::rand_core::{RngCore};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use radix_engine::transaction::{TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use resource_tests::ResourceTestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_one_pool() {
    (1u64..12u64).into_par_iter().for_each(|seed| {
        let mut one_pool_fuzz_test = OnePoolFuzzTest::new(seed);
        one_pool_fuzz_test.run_fuzz();
    })
}

struct OnePoolFuzzTest {
    fuzzer: ResourceTestFuzzer,
    test_runner: DefaultTestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl OnePoolFuzzTest {
    fn new(seed: u64) -> Self {
        let fuzzer = ResourceTestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            7,
            account,
        );

        let (pool_component, pool_unit_resource) = test_runner.create_one_resource_pool(
            resource_address,
            rule!(require(virtual_signature_badge)),
        );

        Self {
            fuzzer,
            test_runner,
            pool_component_address: pool_component,
            pool_unit_resource_address: pool_unit_resource,
            resource_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn run_fuzz(&mut self) {
        for _ in 0..5000 {
            match self.fuzzer.next_u32(5u32) {
                0u32 => {
                    let amount = self.fuzzer.next_amount();
                    self.contribute(amount)
                },
                1u32 => {
                    let amount = self.fuzzer.next_amount();
                    self.protected_deposit(amount)
                },
                2u32 => {
                    let amount = self.fuzzer.next_amount();
                    let withdraw_strategy = self.fuzzer.next_withdraw_strategy();
                    self.protected_withdraw(amount, withdraw_strategy)
                },
                3u32 => {
                    let amount = self.fuzzer.next_amount();
                    self.redeem(amount)
                },
                _ => {
                    let amount = self.fuzzer.next_amount();
                    self.get_redemption_value(amount)
                },
            };
        }
    }

    fn contribute<D: Into<Decimal>>(&mut self, amount: D) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
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
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest)
    }

    fn protected_deposit<D: Into<Decimal>>(&mut self, amount: D) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
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
        self.execute_manifest(manifest)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        amount: D,
        withdraw_strategy: WithdrawStrategy,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                OneResourcePoolProtectedWithdrawManifestInput {
                    amount: amount.into(),
                    withdraw_strategy,
                },
            )
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest)
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.pool_component_address,
                ONE_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                OneResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                },
            )
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
}
