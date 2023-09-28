//! This module contains the fuzz test for joint fuzzing of the state and behavior of component and
//! package royalties. The following are the invariants tested for:
//!
//! 1. No package royalties exist for function that are not defined the package schema.
//! 2. Royalty amounts can not be negative or greater than the maximum.
//! 3. No transaction involving the royalty module should end in a native-vm panic.

use fuzz_tests::utils::*;
use fuzz_tests::{
    continue_if_manifest_is_unpreparable, fuzz_template, return_if_manifest_is_unpreparable,
};

use radix_engine::blueprints::package::*;
use radix_engine::system::attached_modules::royalty::*;
use radix_engine::system::checkers::*;
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_engine::transaction::*;
use radix_engine_stores::memory_db::*;
use transaction::prelude::*;

use arbitrary::Arbitrary;
use radix_engine_interface::prelude::node_modules::royalty::*;

fuzz_template!(|input: RoyaltyFuzzerInput| { fuzz_func(input) });

fn fuzz_func(input: RoyaltyFuzzerInput) {
    let mut test_runner = package::test_runner();

    // Generate the package definition, change the royalty configuration and then attempt to publish
    // it.
    let package_address = {
        let package_royalties = input.0.royalty_config.package_royalty_config();
        let mut package_definition = package::RoyaltyFuzzBlueprint::definition();
        package_definition
            .blueprints
            .get_mut(package::BLUEPRINT_IDENT)
            .unwrap()
            .royalty_config = package_royalties;

        let receipt = test_runner.execute_system_transaction(
            return_if_manifest_is_unpreparable!(ManifestBuilder::new()
                .call_function(
                    PACKAGE_PACKAGE,
                    PACKAGE_BLUEPRINT,
                    PACKAGE_PUBLISH_NATIVE_IDENT,
                    &PackagePublishNativeManifestInput {
                        definition: package_definition,
                        native_package_code_id: package::PACKAGE_CODE_ID,
                        metadata: Default::default(),
                        package_address: None
                    }
                )
                .build())
            .instructions,
            Default::default(),
        );
        panic_if_native_vm_trap(&receipt);

        if let Some(address) = map_if_commit_success(&receipt, |_, commit_result, _| {
            *commit_result
                .state_update_summary
                .new_packages
                .first()
                .unwrap()
        }) {
            address
        } else {
            return;
        }
    };

    // Instantiate a new royalty test component and get the component address.
    let component_address = {
        let receipt = test_runner.execute_manifest(
            assert_can_be_prepared!(
                return,
                TransactionManifestV1 {
                    instructions: vec![
                        InstructionV1::CallMethod {
                            address: DynamicGlobalAddress::Static(FAUCET.into()),
                            method_name: "lock_fee".to_owned(),
                            args: manifest_args!(dec!("100")).into(),
                        },
                        InstructionV1::CallFunction {
                            package_address: DynamicPackageAddress::Static(package_address),
                            blueprint_name: package::BLUEPRINT_IDENT.to_owned(),
                            function_name: package::ROYALTY_FUZZ_BLUEPRINT_INSTANTIATE_IDENT
                                .to_owned(),
                            args: to_manifest_value_ignoring_depth(
                                &package::RoyaltyFuzzBlueprintInstantiateInput {
                                    creation_invocation: input.1.creation_invocation,
                                    pre_attachment_invocations: input.1.pre_attachment_invocations,
                                }
                            ),
                        },
                    ],
                    blobs: Default::default(),
                }
            ),
            vec![],
        );
        panic_if_native_vm_trap(&receipt);
        if let Some(address) = map_if_commit_success(&receipt, |_, commit_result, _| {
            *commit_result
                .state_update_summary
                .new_components
                .first()
                .unwrap()
        }) {
            address
        } else {
            return;
        }
    };

    // Perform the method invocations to the royalty module. Each invocation is its own transaction.
    // This is because we would like for a failed invocation not to stop other ones from happening.
    let mut receipts = Vec::new();
    for invocation in input.1.post_attachment_invocations.into_iter() {
        let manifest = continue_if_manifest_is_unpreparable!(TransactionManifestV1 {
            instructions: vec![
                InstructionV1::CallMethod {
                    address: DynamicGlobalAddress::Static(FAUCET.into()),
                    method_name: "lock_fee".to_owned(),
                    args: manifest_args!(dec!("100")).into(),
                },
                InstructionV1::CallMethod {
                    address: DynamicGlobalAddress::Static(component_address.into()),
                    method_name: invocation.method_ident().to_owned(),
                    args: invocation.manifest_value(),
                },
            ],
            blobs: Default::default(),
        });
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        receipts.push(receipt);
    }

    check_invariants(
        package_address,
        component_address,
        &receipts,
        test_runner.substate_db(),
    )
}

fn check_invariants(
    package_address: PackageAddress,
    component_address: ComponentAddress,
    receipts: &[TransactionReceipt],
    substate_database: &InMemorySubstateDatabase,
) {
    let reader = SystemDatabaseReader::new(substate_database);
    let mut component_royalties_checker = ComponentRoyaltyDatabaseChecker::default();
    let mut package_royalties_checker =
        PackageRoyaltyDatabaseChecker::new(|blueprint_id, func_name| {
            reader
                .get_blueprint_definition(blueprint_id)
                .map(|bp_def| bp_def.interface.functions.contains_key(func_name))
                .unwrap_or(false)
        });

    // Component royalties validation
    for (key, value) in reader
        .collection_iter(
            component_address.as_node_id(),
            ModuleId::Royalty,
            ComponentRoyaltyCollection::MethodAmountKeyValue.collection_index(),
        )
        .expect("Impossible case.")
    {
        let SubstateKey::Map(key) = key
            else {
                panic!("Impossible case.")
            };
        component_royalties_checker.on_collection_entry(
            reader
                .get_blueprint_type_target(component_address.as_node_id(), ModuleId::Main)
                .unwrap()
                .blueprint_info,
            component_address.into(),
            ModuleId::Royalty,
            ComponentRoyaltyCollection::MethodAmountKeyValue.collection_index(),
            &key,
            &value,
        )
    }

    // Package royalties validation
    for (key, value) in reader
        .collection_iter(
            package_address.as_node_id(),
            ModuleId::Royalty,
            PackageCollection::BlueprintVersionRoyaltyConfigKeyValue.collection_index(),
        )
        .expect("Impossible case.")
    {
        let SubstateKey::Map(key) = key
            else {
                panic!("Impossible case.")
            };
        package_royalties_checker.on_collection_entry(
            reader
                .get_blueprint_type_target(package_address.as_node_id(), ModuleId::Main)
                .unwrap()
                .blueprint_info,
            package_address.into(),
            ModuleId::Royalty,
            PackageCollection::BlueprintVersionRoyaltyConfigKeyValue.collection_index(),
            &key,
            &value,
        )
    }

    let component_royalties_results = component_royalties_checker.on_finish();
    let package_royalties_results = package_royalties_checker.on_finish();

    if !component_royalties_results.is_empty() || !package_royalties_results.is_empty() {
        panic!("Encountered invalid state in the component or package royalties: {component_royalties_results:#?}, {package_royalties_results:#?}");
    }

    // Verify that none of the transactions panicked the native vm.
    panic_if_native_vm_trap(receipts);
}

#[derive(Arbitrary, Clone, Debug, ScryptoSbor, serde::Serialize, serde::Deserialize)]
struct RoyaltyFuzzerInput(RoyaltyFuzzerPackageInput, RoyaltyFuzzerComponentInput);

#[derive(Arbitrary, Clone, Debug, ScryptoSbor, serde::Serialize, serde::Deserialize)]
struct RoyaltyFuzzerPackageInput {
    /// The configuration to use for package royalties, this may either be valid function names or
    /// completely random.
    royalty_config: RoyaltyFuzzerPackageRoyaltyConfig,
}

#[derive(Arbitrary, Clone, Debug, ScryptoSbor, serde::Serialize, serde::Deserialize)]
struct RoyaltyFuzzerComponentInput {
    /// The invocation made for the creation of the royalty module.
    creation_invocation: ComponentRoyaltyCreateInput,
    /// The method invocations to make to the royalty module before it has been attached to
    /// the component.
    pre_attachment_invocations: Vec<ComponentRoyaltyMethodInvocation>,
    /// The method invocations to make to the royalty module after it has been attached to
    /// the component.
    post_attachment_invocations: Vec<ComponentRoyaltyMethodInvocation>,
}

#[derive(Arbitrary, Clone, Debug, ScryptoSbor, serde::Serialize, serde::Deserialize)]
pub enum RoyaltyFuzzerPackageRoyaltyConfig {
    ValidFunctionNames {
        /// The royalty amount of the `instantiate` function.
        instantiate_fn: RoyaltyAmount,
    },
    CompletelyRandom(PackageRoyaltyConfig),
}

impl RoyaltyFuzzerPackageRoyaltyConfig {
    pub fn package_royalty_config(self) -> PackageRoyaltyConfig {
        match self {
            Self::ValidFunctionNames { instantiate_fn } => {
                PackageRoyaltyConfig::Enabled(indexmap! {
                    package::ROYALTY_FUZZ_BLUEPRINT_INSTANTIATE_IDENT.to_owned() => instantiate_fn
                })
            }
            Self::CompletelyRandom(random) => random,
        }
    }
}

#[derive(
    Arbitrary, Clone, Debug, ScryptoSbor, ManifestSbor, serde::Serialize, serde::Deserialize,
)]
pub enum ComponentRoyaltyMethodInvocation {
    Set(ComponentRoyaltySetInput),
    Lock(ComponentRoyaltyLockInput),
    Claim(ComponentClaimRoyaltiesInput),
}

impl ComponentRoyaltyMethodInvocation {
    pub const fn method_ident(&self) -> &'static str {
        match self {
            Self::Set(..) => COMPONENT_ROYALTY_SET_ROYALTY_IDENT,
            Self::Lock(..) => COMPONENT_ROYALTY_LOCK_ROYALTY_IDENT,
            Self::Claim(..) => COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
        }
    }

    /// Convert the arguments to a [`ManifestValue`]. This method does not adhere to the SBOR depth
    /// limits.
    pub fn manifest_value(&self) -> ManifestValue {
        match self {
            Self::Set(value) => to_manifest_value_ignoring_depth(&value),
            Self::Lock(value) => to_manifest_value_ignoring_depth(&value),
            Self::Claim(value) => to_manifest_value_ignoring_depth(&value),
        }
    }
}

#[derive(Arbitrary, Clone, Debug, ScryptoSbor, serde::Serialize, serde::Deserialize)]
pub enum PackageRoyaltyMethodInvocation {
    Claim(PackageClaimRoyaltiesInput),
}

impl PackageRoyaltyMethodInvocation {
    pub const fn method_ident(&self) -> &'static str {
        match self {
            Self::Claim(..) => COMPONENT_ROYALTY_CLAIM_ROYALTIES_IDENT,
        }
    }

    /// Convert the arguments to a [`ManifestValue`]. This method does not adhere to the SBOR depth
    /// limits.
    pub fn manifest_value(&self) -> ManifestValue {
        match self {
            Self::Claim(value) => to_manifest_value_ignoring_depth(&value),
        }
    }
}

// A module of the package used in this test.
mod package {
    use super::*;

    use native_sdk::modules::metadata::*;
    use native_sdk::modules::royalty::*;

    use radix_engine::errors::RuntimeError;
    use radix_engine::vm::*;
    use radix_engine_interface::prelude::AttachedModuleId;
    use radix_engine_interface::prelude::Bucket;
    use radix_engine_interface::prelude::ClientApi;
    use radix_engine_stores::memory_db::InMemorySubstateDatabase;
    use scrypto_unit::*;

    pub const BLUEPRINT_IDENT: &str = "RoyaltyFuzzBlueprint";
    pub const PACKAGE_CODE_ID: u64 = 1024;

    #[derive(Clone)]
    pub struct TestInvoke;
    impl VmInvoke for TestInvoke {
        fn invoke<Y>(
            &mut self,
            export_name: &str,
            input: &IndexedScryptoValue,
            api: &mut Y,
        ) -> Result<IndexedScryptoValue, RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
        {
            match export_name {
                ROYALTY_FUZZ_BLUEPRINT_INSTANTIATE_IDENT => {
                    let RoyaltyFuzzBlueprintInstantiateInput {
                        creation_invocation,
                        pre_attachment_invocations,
                    } = input
                        .as_typed::<RoyaltyFuzzBlueprintInstantiateInput>()
                        .expect("Failed to decode");
                    RoyaltyFuzzBlueprint::instantiate(
                        creation_invocation,
                        pre_attachment_invocations,
                        api,
                    )
                    .map(|value| IndexedScryptoValue::from_typed(&value))
                }
                _ => {
                    panic!("Invalid method, how did we even get here?")
                }
            }
        }
    }

    pub struct RoyaltyFuzzBlueprint;

    impl RoyaltyFuzzBlueprint {
        pub fn definition() -> PackageDefinition {
            PackageDefinition::new_functions_only_test_definition(
                BLUEPRINT_IDENT,
                vec![(
                    ROYALTY_FUZZ_BLUEPRINT_INSTANTIATE_IDENT,
                    ROYALTY_FUZZ_BLUEPRINT_INSTANTIATE_IDENT,
                    false,
                )],
            )
        }

        fn instantiate<Y>(
            ComponentRoyaltyCreateInput { royalty_config }: ComponentRoyaltyCreateInput,
            pre_attachment_invocations: Vec<ComponentRoyaltyMethodInvocation>,
            api: &mut Y,
        ) -> Result<RoyaltyFuzzBlueprintInstantiateOutput, RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
        {
            let mut royalty = ComponentRoyalty(ComponentRoyalty::create(royalty_config, api)?);
            let metadata = Metadata::create(api)?;
            let node_id = api.new_simple_object(BLUEPRINT_IDENT, Default::default())?;

            /* Perform all of the method invocations */
            let mut buckets = vec![];
            for invocation in pre_attachment_invocations {
                match invocation {
                    ComponentRoyaltyMethodInvocation::Set(ComponentRoyaltySetInput {
                        method,
                        amount,
                    }) => royalty.set_royalty(&method, amount, api)?,
                    ComponentRoyaltyMethodInvocation::Lock(ComponentRoyaltyLockInput {
                        method,
                    }) => royalty.lock_royalty(&method, api)?,
                    ComponentRoyaltyMethodInvocation::Claim(ComponentClaimRoyaltiesInput {}) => {
                        buckets.push(royalty.claim_royalty(api)?)
                    }
                }
            }

            let modules = indexmap! {
                AttachedModuleId::Royalty => royalty.0.0,
                AttachedModuleId::Metadata => metadata.0,
            };

            api.globalize(node_id, modules, None)?;

            Ok(buckets)
        }
    }

    pub const ROYALTY_FUZZ_BLUEPRINT_INSTANTIATE_IDENT: &str = "instantiate";
    #[derive(
        Arbitrary, Clone, Debug, ManifestSbor, ScryptoSbor, serde::Serialize, serde::Deserialize,
    )]
    pub struct RoyaltyFuzzBlueprintInstantiateInput {
        pub creation_invocation: ComponentRoyaltyCreateInput,
        pub pre_attachment_invocations: Vec<ComponentRoyaltyMethodInvocation>,
    }
    pub type RoyaltyFuzzBlueprintInstantiateOutput = Vec<Bucket>;

    /// Creates a new test-runner with this package published to it and ready for use.
    pub fn test_runner() -> TestRunner<OverridePackageCode<TestInvoke>, InMemorySubstateDatabase> {
        TestRunnerBuilder::new()
            .with_custom_extension(OverridePackageCode::new(PACKAGE_CODE_ID, TestInvoke))
            .without_trace()
            .build_from_snapshot(TEST_RUNNER_SNAPSHOT.clone())
    }

    lazy_static::lazy_static! {
        static ref TEST_RUNNER_SNAPSHOT: TestRunnerSnapshot = {
            let test_runner = TestRunnerBuilder::new()
                .with_custom_extension(OverridePackageCode::new(PACKAGE_CODE_ID, TestInvoke))
                .without_trace()
                .build();
            test_runner.create_snapshot()
        };
    }
}

#[cfg(test)]
mod tests {
    use arbitrary::*;
    use rand::{RngCore, SeedableRng};
    use rand_chacha::*;

    use crate::*;

    #[test]
    fn test_royalty_state_generate_fuzz_input_data() {
        for (index, input) in gen_random(7).into_iter().enumerate() {
            std::fs::write(
                format!("royalty_state_{:03?}.raw", index),
                bincode::serialize(&input).unwrap(),
            )
            .unwrap();
        }
    }

    fn gen_random(n: usize) -> Vec<RoyaltyFuzzerInput> {
        let mut vec = Vec::new();

        while vec.len() < n {
            let mut rng = ChaCha8Rng::from_entropy();
            let mut bytes = [0u8; 1024 * 10];
            rng.fill_bytes(&mut bytes);
            let mut unstructured = Unstructured::new(&bytes);
            let input = RoyaltyFuzzerInput::arbitrary(&mut unstructured).unwrap();
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                fuzz_func(input.clone());
            }));
            if result.is_ok() {
                vec.push(input)
            }
        }

        vec
    }
}

macro_rules! assert_can_be_prepared {
    ($else: expr, $manifest: expr) => {{
        let manifest = $manifest;
        if ::transaction::prelude::TestTransaction::new_from_nonce(manifest.clone(), 0)
            .prepare()
            .is_ok()
        {
            manifest
        } else {
            $else;
        }
    }};
}
use assert_can_be_prepared;
