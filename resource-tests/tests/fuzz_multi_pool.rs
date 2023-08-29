use radix_engine::blueprints::pool::multi_resource_pool::MULTI_RESOURCE_POOL_BLUEPRINT_IDENT;
use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::TestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_multi_pool() {
    (1u64..64u64).into_par_iter().for_each(|seed| {
        let mut multi_pool_fuzz_test = MultiPoolFuzzTest::new(seed);
        multi_pool_fuzz_test.run_fuzz();
    });
}

struct MultiPoolFuzzTest {
    fuzzer: TestFuzzer,
    test_runner: DefaultTestRunner,
    pool_component_address: ComponentAddress,
    pool_unit_resource_address: ResourceAddress,
    pool_resources: Vec<ResourceAddress>,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl MultiPoolFuzzTest {
    pub fn new(seed: u64) -> Self {
        let mut fuzzer = TestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new().without_trace().build();
        let (public_key, _, account) = test_runner.new_account(false);
        let virtual_signature_badge = NonFungibleGlobalId::from_public_key(&public_key);

        let divisibility = vec![
            fuzzer.next_valid_divisibility(),
            fuzzer.next_valid_divisibility(),
            fuzzer.next_valid_divisibility(),
        ];

        let resource_addresses: Vec<ResourceAddress> = divisibility
            .into_iter()
            .map(|divisibility| {
                test_runner.create_freely_mintable_and_burnable_fungible_resource(
                    OwnerRole::None,
                    None,
                    divisibility,
                    account,
                )
            })
            .collect();

        let (pool_component, pool_unit_resource) = {
            let manifest = ManifestBuilder::new()
                .call_function(
                    POOL_PACKAGE,
                    MULTI_RESOURCE_POOL_BLUEPRINT_IDENT,
                    MULTI_RESOURCE_POOL_INSTANTIATE_IDENT,
                    MultiResourcePoolInstantiateManifestInput {
                        resource_addresses: resource_addresses.clone().into_iter().collect(),
                        pool_manager_rule: rule!(require(virtual_signature_badge)),
                        owner_role: OwnerRole::None,
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
            fuzzer,
            test_runner,
            pool_component_address: pool_component,
            pool_unit_resource_address: pool_unit_resource,
            pool_resources: resource_addresses,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn run_fuzz(&mut self) {
        for _ in 0..5000 {
            match self.fuzzer.next_u32(5u32) {
                0u32 => {
                    let resource_to_amount_mapping = self
                        .pool_resources
                        .iter()
                        .map(|resource| (*resource, self.fuzzer.next_amount()))
                        .collect();
                    self.contribute(resource_to_amount_mapping);
                }
                1u32 => {
                    let resource = self
                        .pool_resources
                        .get(self.fuzzer.next_usize(self.pool_resources.len()))
                        .unwrap();
                    let amount = self.fuzzer.next_amount();
                    self.protected_deposit(*resource, amount);
                }
                2u32 => {
                    let resource = self
                        .pool_resources
                        .get(self.fuzzer.next_usize(self.pool_resources.len()))
                        .unwrap();
                    let amount = self.fuzzer.next_amount();
                    let withdraw_strategy = self.fuzzer.next_withdraw_strategy();
                    self.protected_withdraw(*resource, amount, withdraw_strategy);
                }
                3u32 => {
                    let amount = self.fuzzer.next_amount();
                    self.redeem(amount);
                }
                _ => {
                    let amount = self.fuzzer.next_amount();
                    self.get_redemption_value(amount);
                }
            };
        }
    }

    pub fn contribute(
        &mut self,
        resource_to_amount_mapping: BTreeMap<ResourceAddress, Decimal>,
    ) -> TransactionReceipt {
        let mut manifest_builder = ManifestBuilder::new();
        for (resource_address, amount) in resource_to_amount_mapping.iter() {
            manifest_builder = manifest_builder.mint_fungible(*resource_address, *amount)
        }
        let manifest = manifest_builder
            .call_method(
                self.pool_component_address,
                MULTI_RESOURCE_POOL_CONTRIBUTE_IDENT,
                manifest_args!(ManifestExpression::EntireWorktop),
            )
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
                    MULTI_RESOURCE_POOL_REDEEM_IDENT,
                    MultiResourcePoolRedeemManifestInput {
                        bucket: lookup.bucket("pool_unit"),
                    },
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
                MULTI_RESOURCE_POOL_PROTECTED_WITHDRAW_IDENT,
                MultiResourcePoolProtectedWithdrawManifestInput {
                    resource_address,
                    amount: amount.into(),
                    withdraw_strategy,
                },
            )
            .try_deposit_batch_or_abort(self.account_component_address, None)
            .build();
        self.execute_manifest(manifest)
    }

    fn execute_manifest(&mut self, manifest: TransactionManifestV1) -> TransactionReceipt {
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
                MULTI_RESOURCE_POOL_GET_REDEMPTION_VALUE_IDENT,
                MultiResourcePoolGetRedemptionValueManifestInput {
                    amount_of_pool_units: amount_of_pool_units.into(),
                },
            )
            .build();
        self.execute_manifest(manifest)
    }
}
