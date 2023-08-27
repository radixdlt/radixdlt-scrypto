use radix_engine::transaction::TransactionReceipt;
use radix_engine::types::*;
use radix_engine_interface::blueprints::pool::*;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use radix_engine::system::bootstrap::{DEFAULT_TESTING_FAUCET_SUPPLY, GenesisDataChunk, GenesisStakeAllocation, GenesisValidator};
use radix_engine_interface::blueprints::consensus_manager::{VALIDATOR_GET_REDEMPTION_VALUE_IDENT, ValidatorGetRedemptionValueInput};
use resource_tests::ResourceTestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_consensus() {
    (1u64..64u64).into_par_iter().for_each(|seed| {
        let mut one_pool_fuzz_test = ConsensusFuzzTest::new(seed);
        one_pool_fuzz_test.run_fuzz();
    })
}

struct ConsensusFuzzTest {
    fuzzer: ResourceTestFuzzer,
    test_runner: DefaultTestRunner,
    validator_address: ComponentAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl ConsensusFuzzTest {
    fn new(seed: u64) -> Self {
        let mut fuzzer = ResourceTestFuzzer::new(seed);

        let initial_epoch = Epoch::of(5);
        let genesis = CustomGenesis::default(
            initial_epoch,
            CustomGenesis::default_consensus_manager_config(),
        );
        let (mut test_runner, validator_set) = TestRunnerBuilder::new()
            .with_custom_genesis(genesis)
            .build_and_get_epoch();
        let public_key = Secp256k1PrivateKey::from_u64(1u64).unwrap().public_key();
        let account = ComponentAddress::virtual_account_from_public_key(&public_key);

        let validator_address = validator_set.validators_by_stake_desc.iter().next().unwrap().0.clone();

        Self {
            fuzzer,
            test_runner,
            validator_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn run_fuzz(&mut self) {
        for _ in 0..100 {
            match self.fuzzer.next_u32(1u32) {
                _ => {
                    let amount = self.fuzzer.next_amount();
                    self.get_redemption_value(amount)
                }
            };
        }
    }

    fn get_redemption_value(
        &mut self,
        amount_of_stake_units: Decimal,
    ) -> TransactionReceipt {
        let manifest = ManifestBuilder::new()
            .call_method(
                self.validator_address,
                VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
                ValidatorGetRedemptionValueInput {
                    amount_of_stake_units,
                },
            )
            .build();
        self.test_runner
            .execute_manifest_ignoring_fee(manifest, vec![])
    }
}
