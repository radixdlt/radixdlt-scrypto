use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::role_assignment::RoleAssignment;
use native_sdk::resource::NativeVault;
use radix_engine::errors::RuntimeError;
use radix_engine::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use radix_engine::system::system_callback::SystemLockData;
use radix_engine::transaction::TransactionOutcome;
use radix_engine::types::*;
use radix_engine::vm::{OverridePackageCode, VmInvoke};
use radix_engine_interface::blueprints::package::PackageDefinition;
use radix_engine_stores::memory_db::InMemorySubstateDatabase;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use resource_tests::TestFuzzer;
use scrypto_unit::*;
use transaction::prelude::*;
use radix_engine_interface::api::node_modules::auth::ToRoleEntry;
use radix_engine::prelude::node_modules::auth::RoleDefinition;
use radix_engine_interface::prelude::node_modules::ModuleConfig;

#[test]
fn fuzz_non_fungible_resource() {
    let mut summed_results: BTreeMap<NonFungibleResourceFuzzTxn, BTreeMap<FuzzTxnResult, u64>> = BTreeMap::new();

    let results: Vec<BTreeMap<NonFungibleResourceFuzzTxn, BTreeMap<FuzzTxnResult, u64>>> =
        (1u64..64u64)
            .into_par_iter()
            .map(|seed| {
                let mut resource_fuzz_test = ResourceFuzzTest::new(seed);
                resource_fuzz_test.run_fuzz()
            })
            .collect();

    for run_result in results {
        for (txn, txn_results) in run_result {
            for (txn_result, count) in txn_results {
                summed_results.entry(txn).or_default().entry(txn_result).or_default().add_assign(&count);
            }
        }
    }

    println!("{:#?}", summed_results);

    panic!("oops");
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum NonFungibleResourceFuzzStartAction {
    Mint,
    VaultTake,
    VaultTakeNonFungibles,
    VaultTakeAdvanced,
    VaultRecall,
    VaultRecallNonFungibles,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum NonFungibleResourceFuzzEndAction {
    Burn,
    VaultPut,
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
struct NonFungibleResourceFuzzTxn(NonFungibleResourceFuzzStartAction, NonFungibleResourceFuzzEndAction);

#[repr(u8)]
#[derive(Copy, Clone, Debug, FromRepr, Ord, PartialOrd, Eq, PartialEq)]
enum FuzzTxnResult {
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
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError> + KernelNodeApi + KernelSubstateApi<SystemLockData>,
    {
        match export_name {
            "call_vault" => {
                let handle = api
                    .actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::read_only())
                    .unwrap();
                let vault: Vault = api.field_read_typed(handle).unwrap();

                let input: (String, ScryptoValue) = scrypto_decode(input.as_slice()).unwrap();

                let rtn = api.call_method(
                    vault.0.as_node_id(),
                    input.0.as_str(),
                    scrypto_encode(&input.1).unwrap(),
                )?;
                return Ok(IndexedScryptoValue::from_vec(rtn).unwrap());
            }
            "new" => {
                let resource_address: (ResourceAddress,) =
                    scrypto_decode(input.as_slice()).unwrap();
                let vault = Vault::create(resource_address.0, api).unwrap();

                let metadata = Metadata::create(api)?;
                let access_rules = RoleAssignment::create(OwnerRole::None, btreemap!(), api)?;
                let node_id = api
                    .new_simple_object(BLUEPRINT_NAME, btreemap!(0u8 => FieldValue::new(&vault)))?;

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
    fuzzer: TestFuzzer,
    test_runner: TestRunner<OverridePackageCode<TestInvoke>, InMemorySubstateDatabase>,
    resource_address: ResourceAddress,
    component_address: ComponentAddress,
    vault_id: InternalAddress,
    account_public_key: PublicKey,
    account_component_address: ComponentAddress,
}

impl ResourceFuzzTest {
    fn new(seed: u64) -> Self {
        let fuzzer = TestFuzzer::new(seed);
        let mut test_runner = TestRunnerBuilder::new()
            .with_custom_extension(OverridePackageCode::new(CUSTOM_PACKAGE_CODE_ID, TestInvoke))
            .build();
        let package_address = test_runner.publish_native_package(
            CUSTOM_PACKAGE_CODE_ID,
            PackageDefinition::new_with_field_test_definition(
                BLUEPRINT_NAME,
                vec![("call_vault", "call_vault", true), ("new", "new", false)],
            ),
        );

        let (public_key, _, account) = test_runner.new_account(false);

        let receipt = test_runner.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .create_non_fungible_resource(
                    OwnerRole::None,
                    NonFungibleIdType::Integer,
                    true,
                    NonFungibleResourceRoles {
                        mint_roles: mint_roles! {
                            minter => rule!(allow_all);
                            minter_updater => rule!(deny_all);
                        },
                        burn_roles: burn_roles! {
                            burner => rule!(allow_all);
                            burner_updater => rule!(deny_all);
                        },
                        recall_roles: recall_roles! {
                            recaller => rule!(allow_all);
                            recaller_updater => rule!(deny_all);
                        },
                        ..Default::default()
                    },
                    metadata!(),
                    Option::<BTreeMap<NonFungibleLocalId, ()>>::None,
                )
                .build(),
            vec![],
        );
        let resource_address = receipt.expect_commit_success().new_resource_addresses()[0];

        let receipt = test_runner.execute_manifest_ignoring_fee(
            ManifestBuilder::new()
                .call_function(
                    package_address,
                    BLUEPRINT_NAME,
                    "new",
                    manifest_args!(resource_address),
                )
                .build(),
            vec![],
        );
        let component_address = receipt.expect_commit_success().new_component_addresses()[0];

        let vault_id = test_runner.get_component_vaults(component_address, resource_address)[0];

        Self {
            fuzzer,
            test_runner,
            resource_address,
            component_address,
            vault_id: InternalAddress::try_from(vault_id).unwrap(),
            account_public_key: public_key.into(),
            account_component_address: account,
        }
    }

    fn next_amount(&mut self) -> Decimal {
        self.fuzzer.next_amount()
    }

    fn run_fuzz(
        &mut self,
    ) -> BTreeMap<NonFungibleResourceFuzzTxn, BTreeMap<FuzzTxnResult, u64>> {
        let mut fuzz_results: BTreeMap<
            NonFungibleResourceFuzzTxn,
            BTreeMap<FuzzTxnResult, u64>,
        > = BTreeMap::new();
        for _ in 0..700 {
            let builder = ManifestBuilder::new();
            let start = NonFungibleResourceFuzzStartAction::from_repr(self.fuzzer.next_u8(6u8)).unwrap();
            let (builder, mut trivial) = match start {
                NonFungibleResourceFuzzStartAction::Mint => {
                    let entries: BTreeMap<NonFungibleLocalId, (ManifestValue, )> = self.fuzzer.next_non_fungible_id_set()
                        .into_iter().map(|id| {
                        let value: ManifestValue = manifest_decode(&manifest_encode(&()).unwrap()).unwrap();
                        (id, (value, ))
                    }).collect();

                    let builder = builder.call_method(
                        self.resource_address,
                        NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                        NonFungibleResourceManagerMintManifestInput {
                            entries,
                        }
                    );

                    (builder, false)
                }
                NonFungibleResourceFuzzStartAction::VaultTake => {
                    let amount = self.next_amount();
                    let builder = builder.call_method(
                        self.component_address,
                        "call_vault",
                        manifest_args!(VAULT_TAKE_IDENT, (amount,)),
                    );
                    (builder, amount.is_zero())
                }
                NonFungibleResourceFuzzStartAction::VaultTakeNonFungibles => {
                    let ids = self.fuzzer.next_non_fungible_id_set();
                    let trivial = ids.is_empty();
                    let builder = builder.call_method(
                        self.component_address,
                        "call_vault",
                        manifest_args!(NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT, (ids,)),
                    );
                    (builder, trivial)
                }
                NonFungibleResourceFuzzStartAction::VaultTakeAdvanced => {
                    let amount = self.next_amount();
                    let withdraw_strategy = self.fuzzer.next_withdraw_strategy();
                    let builder = builder.call_method(
                        self.component_address,
                        "call_vault",
                        manifest_args!(VAULT_TAKE_ADVANCED_IDENT, (amount, withdraw_strategy)),
                    );
                    (builder, amount.is_zero())
                }
                NonFungibleResourceFuzzStartAction::VaultRecall => {
                    let amount = self.next_amount();
                    let builder = builder.recall(self.vault_id, amount);
                    (builder, amount.is_zero())
                }
                NonFungibleResourceFuzzStartAction::VaultRecallNonFungibles => {
                    let ids = self.fuzzer.next_non_fungible_id_set();
                    let trivial = ids.is_empty();
                    let builder = builder.recall_non_fungibles(self.vault_id, &ids);
                    (builder, trivial)
                }
            };

            let end = NonFungibleResourceFuzzEndAction::from_repr(self.fuzzer.next_u8(2u8)).unwrap();
            let (builder, end_trivial) = match end {
                NonFungibleResourceFuzzEndAction::Burn => {
                    let amount = self.next_amount();
                    let builder = builder
                        .take_from_worktop(self.resource_address, amount, "bucket")
                        .burn_resource("bucket");
                    (builder, amount.is_zero())
                }
                NonFungibleResourceFuzzEndAction::VaultPut => {
                    let amount = self.next_amount();
                    let builder = builder
                        .take_from_worktop(self.resource_address, amount, "bucket")
                        .with_bucket("bucket", |builder, bucket| {
                            builder.call_method(
                                self.component_address,
                                "call_vault",
                                manifest_args!(VAULT_PUT_IDENT, (bucket,)),
                            )
                        });
                    (builder, amount.is_zero())
                }
            };
            trivial = trivial || end_trivial;

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
            let result = match (&result.outcome, trivial) {
                (TransactionOutcome::Success(..), true) => {
                    FuzzTxnResult::TrivialSuccess
                }
                (TransactionOutcome::Success(..), false) => FuzzTxnResult::Success,
                (TransactionOutcome::Failure(..), true) => {
                    FuzzTxnResult::TrivialFailure
                }
                (TransactionOutcome::Failure(..), false) => FuzzTxnResult::Failure,
            };

            let results = fuzz_results
                .entry(NonFungibleResourceFuzzTxn(start, end))
                .or_default();
            results.entry(result).or_default().add_assign(&1);
        }

        fuzz_results
    }
}
