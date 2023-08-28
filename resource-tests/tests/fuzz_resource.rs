use radix_engine::blueprints::consensus_manager::EpochChangeEvent;
use radix_engine::transaction::{TransactionOutcome, TransactionReceipt};
use radix_engine::types::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ValidatorGetRedemptionValueInput, VALIDATOR_CLAIM_XRD_IDENT,
    VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT, VALIDATOR_GET_REDEMPTION_VALUE_IDENT,
    VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT, VALIDATOR_STAKE_IDENT,
    VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT, VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT,
    VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT, VALIDATOR_UNSTAKE_IDENT, VALIDATOR_UPDATE_FEE_IDENT,
};
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use native_sdk::resource::ResourceManager;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use resource_tests::ResourceTestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn fuzz_resource() {
    let results: Vec<BTreeMap<ResourceFuzzAction, BTreeMap<ConsensusFuzzActionResult, u64>>> =
        (1u64..64u64)
            .into_par_iter()
            .map(|seed| {
                let mut resource_fuzz_test = ResourceFuzzTest::new(seed);
                resource_fuzz_test.run_fuzz()
            })
            .collect();

    println!("{:#?}", results);

    panic!("oops");
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ResourceFuzzStartAction {
    Mint
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ResourceFuzzEndAction {
    Burn
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct ResourceFuzzAction {
    start: ResourceFuzzStartAction,
    end: ResourceFuzzEndAction,
}


#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum ConsensusFuzzActionResult {
    TrivialSuccess,
    Success,
    TrivialFailure,
    Failure,
}

const BLUEPRINT_NAME: &str = "MyBlueprint";
const CUSTOM_PACKAGE_CODE_ID: u64 = 1024;

#[derive(Clone)]
struct TestInvoke;
impl VmInvoke for TestInvoke {
    fn invoke<Y>(
        &mut self,
        export_name: &str,
        _input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match export_name {
            "new" => {
                let metadata = Metadata::create(api)?;
                let access_rules = RoleAssignment::create(OwnerRole::None, btreemap!(), api)?;
                let node_id = api.new_simple_object(BLUEPRINT_NAME, btreemap!(0u8 => FieldValue::new(&())))?;
                api.globalize(
                    node_id,
                    btreemap!(
                            ModuleId::Metadata => metadata.0,
                            ModuleId::RoleAssignment => access_rules.0.0,
                        ),
                    None,
                )?;
            }
            _ => {}
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}

struct ResourceFuzzTest {
    fuzzer: ResourceTestFuzzer,
    test_runner: TestRunner<OverridePackageCode<TestInvoke>, InMemorySubstateDatabase>,
    resource_address: ResourceAddress,
    component_address: ComponentAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl ResourceFuzzTest {
    fn new(seed: u64) -> Self {
        let fuzzer = ResourceTestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new()
            .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
            .build();
        let package_address = test_runner.publish_native_package(
            CUSTOM_PACKAGE_CODE_ID,
            PackageDefinition::new_with_field_test_definition(
                BLUEPRINT_NAME,
                vec![("new", "new", false)],
            ),
        );

        let (public_key, _, account) = test_runner.new_account(false);

        let resource_address = test_runner.create_freely_mintable_and_burnable_fungible_resource(
            OwnerRole::None,
            None,
            18u8,
            account,
        );


        let receipt = test_runner.execute_manifest(
            ManifestBuilder::new()
                .lock_fee(test_runner.faucet_component(), 500u32)
                .call_function(package_address, BLUEPRINT_NAME, "new", manifest_args!())
                .build(),
            vec![],
        );
        let component_address = receipt.expect_commit_success().new_component_addresses()[0];

        Self {
            fuzzer,
            test_runner,
            resource_address,
            component_address,
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn next_amount(&mut self) -> Decimal {
        self.fuzzer.next_amount()
    }

    fn run_fuzz(
        &mut self,
    ) -> BTreeMap<ResourceFuzzAction, BTreeMap<ConsensusFuzzActionResult, u64>> {
        let mut fuzz_results: BTreeMap<
            ResourceFuzzAction,
            BTreeMap<ConsensusFuzzActionResult, u64>,
        > = BTreeMap::new();
        for _ in 0..100 {
            let mut builder = ManifestBuilder::new();
            let start = ResourceFuzzStartAction::from_repr(self.fuzzer.next_u8(1u8)).unwrap();
            let (mut builder, start_trivial) = match start {
                ResourceFuzzStartAction::Mint => {
                    let amount = self.next_amount();
                    let builder = builder
                        .call_method(
                            self.resource_address,
                            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                            FungibleResourceManagerMintInput {
                                amount,
                            }
                        );
                    (builder, amount.is_zero())
                }
            };


            let end = ResourceFuzzEndAction::from_repr(self.fuzzer.next_u8(1u8)).unwrap();
            let (mut builder, end_trivial) = match end {
                ResourceFuzzEndAction::Burn => {
                    {
                        let amount = self.next_amount();
                        let builder = builder
                            .take_from_worktop(self.resource_address, amount, "bucket")
                            .burn_resource("bucket");
                        (builder, amount.is_zero())
                    }
                }
            };



            let manifest = builder
                .deposit_batch(self.account_component_address)
                .build();
            let receipt = self.test_runner.execute_manifest_ignoring_fee(
                manifest,
                vec![NonFungibleGlobalId::from_public_key(
                    &self.account_public_key,
                )],
            );

            let result = receipt.expect_commit_ignore_outcome();
            let result = match (&result.outcome, start_trivial && end_trivial) {
                (TransactionOutcome::Success(..), true) => {
                    ConsensusFuzzActionResult::TrivialSuccess
                }
                (TransactionOutcome::Success(..), false) => ConsensusFuzzActionResult::Success,
                (TransactionOutcome::Failure(..), true) => {
                    ConsensusFuzzActionResult::TrivialFailure
                }
                (TransactionOutcome::Failure(..), false) => ConsensusFuzzActionResult::Failure,
            };

            let results = fuzz_results.entry(ResourceFuzzAction { start, end }).or_default();
            results.entry(result).or_default().add_assign(&1);


        }

        fuzz_results
    }
}
