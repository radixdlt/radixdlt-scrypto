use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use radix_engine::transaction::{TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use scrypto_unit::*;
use transaction::prelude::*;

struct ResourceTestFuzzer {
    rng: ChaCha8Rng,
}

impl ResourceTestFuzzer {
    fn new() -> Self {
        let rng = ChaCha8Rng::seed_from_u64(1234);
        Self {
            rng,
        }
    }

    fn next_amount(&mut self) -> Decimal {
        let next_amount_type = self.rng.gen_range(0u32..6u32);
        match next_amount_type {
            0 => Decimal::ZERO,
            1 => Decimal::ONE,
            2 => Decimal::MAX,
            3 => Decimal::MIN,
            4 => Decimal(I192::ONE),
            _ => {
                let mut bytes = [0u8; 24];
                self.rng.fill_bytes(&mut bytes);
                Decimal(I192::from_le_bytes(&bytes))
            }
        }
    }

    fn next_u32(&mut self, count: u32) -> u32 {
        self.rng.gen_range(0u32..count)
    }
}

#[test]
fn fuzz_one_pool() {
    let mut one_pool_test = OnePoolTest::new(7);
    let mut fuzzer = ResourceTestFuzzer::new();

    for _ in 0..5000 {
        match fuzzer.next_u32(5u32) {
            0u32 => one_pool_test.contribute(fuzzer.next_amount(), true),
            1u32 => one_pool_test.protected_deposit(fuzzer.next_amount(), true),
            2u32 => one_pool_test.protected_withdraw(fuzzer.next_amount(), WithdrawStrategy::Exact, true),
            3u32 => one_pool_test.redeem(fuzzer.next_amount(), true),
            _ => one_pool_test.get_redemption_value(fuzzer.next_amount(), true),
        };
    }
}

struct OnePoolTest {
    test_runner: DefaultTestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    resource_address: ResourceAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl OnePoolTest {
    fn new(divisibility: u8) -> Self {
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            divisibility,
            account,
        );

        let (pool_component, pool_unit_resource) = test_runner.create_one_resource_pool(
            resource_address,
            rule!(require(virtual_signature_badge)),
        );

        Self {
            test_runner,
            pool_component_address: pool_component,
            pool_unit_resource_address: pool_unit_resource,
            resource_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn contribute<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
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
        self.execute_manifest(manifest, sign)
    }

    fn redeem<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
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
        self.execute_manifest(manifest, sign)
    }

    fn protected_deposit<D: Into<Decimal>>(&mut self, amount: D, sign: bool) -> TransactionReceipt {
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
        self.execute_manifest(manifest, sign)
    }

    fn protected_withdraw<D: Into<Decimal>>(
        &mut self,
        amount: D,
        withdraw_strategy: WithdrawStrategy,
        sign: bool,
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
        self.execute_manifest(manifest, sign)
    }

    fn get_redemption_value<D: Into<Decimal>>(
        &mut self,
        amount_of_pool_units: D,
        sign: bool,
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
}
