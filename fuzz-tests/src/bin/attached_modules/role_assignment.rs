//! This binary has fuzz tests for the role-assignment node module. This fuzzer creates random
//! invocations that can be made to a role-assignment module object (or to the role-assignment
//! package as well) meaning that it tests both methods and functions. After each fuzz test we
//! check the database to ensure the following:
//!
//! 1. No reserved roles (roles that begin with an underscore `_`) are defined for any module.
//! 2. No roles in reserved spaces (roles for the role-assignment module itself) are defined.
//! 3. The names of the roles respect the rules we have (e.g., no utf-8 characters that can't be
//! displayed)
//! 4. None of the roles have names exceeding the length.
//! 5. The access rules all respect the depth and width limits.
//! 6. Transactions involving the role-assignment module do not panic. We check the receipt to
//! ensure that if they've resulted in an error, it's not a native vm trap.
//! 7. No roles may be set after the creation of the module.

#![cfg_attr(feature = "libfuzzer-sys", no_main)]

use arbitrary::Arbitrary;

use fuzz_tests::fuzz_template;
use radix_engine::errors::*;
use radix_engine::system::attached_modules::role_assignment::*;
use radix_engine::system::checkers::*;
use radix_engine::system::system_db_reader::*;
use radix_engine::transaction::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::prelude::*;
use radix_engine_stores::memory_db::*;
use transaction::prelude::*;

fuzz_template!(|input: RoleAssignmentFuzzerInput| { fuzz_func(input) });

fn fuzz_func(input: RoleAssignmentFuzzerInput) {
    // Getting the roles that we would have if the creation invocation is valid and the creation tx
    // succeeds
    let role_keys = input.initial_role_keys();

    // Creating a new test-runner. This test runner has the appropriate test package published and
    // ready for us to use in fuzzing
    let (mut test_runner, package_address) = package::test_runner();

    // Instantiate a new role-assignment test component and get the component address.
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
                        function_name: package::ROLE_ASSIGNMENT_FUZZ_BLUEPRINT_INSTANTIATE_IDENT
                            .to_owned(),
                        args: manifest_decode_with_depth_limit(
                            &manifest_encode_with_depth_limit(
                                &package::RoleAssignmentFuzzBlueprintInstantiateInput {
                                    creation_invocation: input.creation_invocation,
                                    pre_attachment_invocations: input.pre_attachment_invocations,
                                },
                                usize::MAX,
                            )
                            .unwrap(),
                            usize::MAX,
                        )
                        .unwrap(),
                    },
                ],
                blobs: Default::default(),
            }
        ),
        vec![],
    );

    // Depending on the result of the above transaction we do different things. If it was successful
    // we get the component address, if it was a committed failure then we just return from this
    // function and stop fuzzing, there is not much to check here.
    let component_address = match receipt.result {
        TransactionResult::Commit(CommitResult {
            outcome: TransactionOutcome::Success(..),
            ref state_update_summary,
            ..
        }) => *state_update_summary.new_components.first().unwrap(),
        TransactionResult::Commit(CommitResult {
            outcome: TransactionOutcome::Failure(..),
            ..
        })
        | TransactionResult::Abort(..)
        | TransactionResult::Reject(..) => return,
    };

    // Do the first check of the invariants
    check_invariants(
        component_address,
        test_runner.substate_db(),
        &[receipt],
        role_keys.clone(),
    );

    // Perform the method invocations to the role-assignment module. Each invocation is its own
    // transaction. This is because we would like for a failed invocation not to stop other ones
    // from happening.
    let mut receipts = Vec::new();
    for invocation in input.post_attachment_invocations.into_iter() {
        let manifest = assert_can_be_prepared!(
            continue,
            TransactionManifestV1 {
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
            }
        );
        let receipt = test_runner.execute_manifest(manifest, vec![]);
        receipts.push(receipt);
    }

    // Do the second check of the invariants
    check_invariants(
        component_address,
        test_runner.substate_db(),
        receipts.as_slice(),
        role_keys,
    );
}

fn check_invariants(
    component_address: ComponentAddress,
    substate_database: &InMemorySubstateDatabase,
    receipts: &[TransactionReceipt],
    initial_roles: Vec<ModuleRoleKey>,
) {
    // We're doing something quite un-orthodox here, we're reading the particular fields of the db
    // that we're interested in and then calling the `RoleAssignmentDatabaseChecker` with their
    // data. We do this to improve performance so that we're not iterating over everything in the
    // database. We still use the RoleAssignmentDatabaseChecker as we would like the checking logic
    // to all exist in one place so that it can be reused in an actual DB checker.
    let reader = SystemDatabaseReader::new(substate_database);
    let mut checker =
        RoleAssignmentDatabaseChecker::new(initial_roles, component_address.into_node_id());
    let blueprint_info = reader
        .get_blueprint_type_target(component_address.as_node_id(), ModuleId::Main)
        .unwrap()
        .blueprint_info;

    {
        let owner_role_substate = reader
            .read_object_field(
                component_address.as_node_id(),
                ModuleId::RoleAssignment,
                RoleAssignmentField::Owner.field_index(),
            )
            .expect("Impossible case.");

        checker.on_field(
            blueprint_info.clone(),
            component_address.into_node_id(),
            ModuleId::RoleAssignment,
            RoleAssignmentField::Owner.field_index(),
            owner_role_substate.as_vec_ref(),
        );
    }

    {
        let iter = reader
            .collection_iter(
                component_address.as_node_id(),
                ModuleId::RoleAssignment,
                RoleAssignmentCollection::AccessRuleKeyValue.collection_index(),
            )
            .expect("Impossible case.");

        for (substate_key, substate_value) in iter {
            let SubstateKey::Map(map_key) = substate_key
            else {
                panic!("Impossible case.")
            };

            checker.on_collection_entry(
                blueprint_info.clone(),
                component_address.into_node_id(),
                ModuleId::RoleAssignment,
                RoleAssignmentCollection::AccessRuleKeyValue.collection_index(),
                &map_key,
                &substate_value,
            )
        }
    }

    let results = checker.on_finish();
    if !results.is_empty() {
        panic!("Found violations in the database: {results:#?}");
    }

    // Verify that none of the transactions panicked.
    {
        for (i, receipt) in receipts.iter().enumerate() {
            if let Some(error) = get_error_from_receipt(receipt) {
                if matches!(
                    error,
                    RuntimeError::VmError(VmError::Native(
                        radix_engine::errors::NativeRuntimeError::Trap { .. }
                    ))
                ) {
                    panic!("An panic was encountered in the transaction. Receipt index: {i}. Error: {error:?}")
                }
            }
        }
    }
}

fn get_error_from_receipt(receipt: &TransactionReceipt) -> Option<&RuntimeError> {
    match receipt.result {
        TransactionResult::Commit(CommitResult {
            outcome: TransactionOutcome::Failure(ref error),
            ..
        })
        | TransactionResult::Reject(RejectResult {
            reason: RejectionReason::ErrorBeforeLoanAndDeferredCostsRepaid(ref error),
        }) => Some(error),
        TransactionResult::Commit(CommitResult {
            outcome: TransactionOutcome::Success(..),
            ..
        })
        | TransactionResult::Abort(..)
        | TransactionResult::Reject(RejectResult { .. }) => None,
    }
}

#[derive(Arbitrary, Clone, Debug, ScryptoSbor, serde::Serialize, serde::Deserialize)]
struct RoleAssignmentFuzzerInput {
    /// The invocation made for the creation of the role-assignment module
    creation_invocation: RoleAssignmentCreateInput,
    /// The method invocations to make to the role-assignment module before it has been attached to
    /// the component.
    pre_attachment_invocations: Vec<RoleAssignmentMethodInvocation>,
    /// The method invocations to make to the role-assignment module after it has been attached to
    /// the component.
    post_attachment_invocations: Vec<RoleAssignmentMethodInvocation>,
}

impl RoleAssignmentFuzzerInput {
    pub fn initial_role_keys(&self) -> Vec<ModuleRoleKey> {
        self.creation_invocation
            .roles
            .iter()
            .flat_map(|(module_id, RoleAssignmentInit { data: init })| {
                init.keys()
                    .map(|role_key| ModuleRoleKey::new(*module_id, role_key.clone()))
            })
            .collect()
    }
}

#[derive(
    Arbitrary, Clone, Debug, ManifestSbor, ScryptoSbor, serde::Serialize, serde::Deserialize,
)]
pub enum RoleAssignmentMethodInvocation {
    Get(RoleAssignmentGetInput),
    Set(RoleAssignmentSetInput),
    SetOwner(RoleAssignmentSetOwnerInput),
    LockOwner(RoleAssignmentLockOwnerInput),
}

impl RoleAssignmentMethodInvocation {
    pub const fn method_ident(&self) -> &'static str {
        match self {
            Self::Get(..) => ROLE_ASSIGNMENT_GET_IDENT,
            Self::Set(..) => ROLE_ASSIGNMENT_SET_IDENT,
            Self::SetOwner(..) => ROLE_ASSIGNMENT_SET_OWNER_IDENT,
            Self::LockOwner(..) => ROLE_ASSIGNMENT_LOCK_OWNER_IDENT,
        }
    }

    /// Convert the arguments to a [`ManifestValue`]. This method does not adhere to the SBOR depth
    /// limits.
    pub fn manifest_value(&self) -> ManifestValue {
        let encoded = match self {
            Self::Get(value) => manifest_encode_with_depth_limit(&value, usize::MAX).unwrap(),
            Self::Set(value) => manifest_encode_with_depth_limit(&value, usize::MAX).unwrap(),
            Self::SetOwner(value) => manifest_encode_with_depth_limit(&value, usize::MAX).unwrap(),
            Self::LockOwner(value) => manifest_encode_with_depth_limit(&value, usize::MAX).unwrap(),
        };
        manifest_decode_with_depth_limit(&encoded, usize::MAX).unwrap()
    }
}

/// A module of the package used in this test.
mod package {
    use super::*;

    use native_sdk::modules::metadata::*;
    use native_sdk::modules::role_assignment::*;

    use radix_engine::vm::*;
    use radix_engine_interface::blueprints::package::*;
    use scrypto_unit::*;

    pub const BLUEPRINT_IDENT: &str = "RoleAssignmentFuzzBlueprint";
    const PACKAGE_CODE_ID: u64 = 1024;

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
                ROLE_ASSIGNMENT_FUZZ_BLUEPRINT_INSTANTIATE_IDENT => {
                    let RoleAssignmentFuzzBlueprintInstantiateInput {
                        creation_invocation,
                        pre_attachment_invocations,
                    } = input
                        .as_typed::<RoleAssignmentFuzzBlueprintInstantiateInput>()
                        .expect("Failed to decode");
                    RoleAssignmentFuzzBlueprint::instantiate(
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

    struct RoleAssignmentFuzzBlueprint;

    impl RoleAssignmentFuzzBlueprint {
        pub fn definition() -> PackageDefinition {
            PackageDefinition::new_functions_only_test_definition(
                BLUEPRINT_IDENT,
                vec![(
                    ROLE_ASSIGNMENT_FUZZ_BLUEPRINT_INSTANTIATE_IDENT,
                    ROLE_ASSIGNMENT_FUZZ_BLUEPRINT_INSTANTIATE_IDENT,
                    false,
                )],
            )
        }

        fn instantiate<Y>(
            RoleAssignmentCreateInput { owner_role, roles }: RoleAssignmentCreateInput,
            pre_attachment_invocations: Vec<RoleAssignmentMethodInvocation>,
            api: &mut Y,
        ) -> Result<RoleAssignmentFuzzBlueprintInstantiateOutput, RuntimeError>
        where
            Y: ClientApi<RuntimeError>,
        {
            let role_assignment = RoleAssignment::create(owner_role, roles, api)?;
            let metadata = Metadata::create(api)?;
            let node_id = api.new_simple_object(BLUEPRINT_IDENT, Default::default())?;

            /* Perform all of the method invocations */
            for invocation in pre_attachment_invocations {
                match invocation {
                    RoleAssignmentMethodInvocation::Get(RoleAssignmentGetInput {
                        module,
                        role_key,
                    }) => role_assignment.get_role(module, role_key, api)?,
                    RoleAssignmentMethodInvocation::Set(RoleAssignmentSetInput {
                        rule,
                        module,
                        role_key,
                    }) => role_assignment.set_role(module, role_key, rule, api)?,
                    RoleAssignmentMethodInvocation::SetOwner(RoleAssignmentSetOwnerInput {
                        rule,
                    }) => role_assignment.set_owner_role(rule, api)?,
                    RoleAssignmentMethodInvocation::LockOwner(RoleAssignmentLockOwnerInput {}) => {
                        role_assignment.lock_owner_role(api)?
                    }
                }
            }

            let modules = indexmap! {
                AttachedModuleId::RoleAssignment => role_assignment.0.0,
                AttachedModuleId::Metadata => metadata.0,
            };

            api.globalize(node_id, modules, None)?;

            Ok(())
        }
    }

    pub const ROLE_ASSIGNMENT_FUZZ_BLUEPRINT_INSTANTIATE_IDENT: &str = "instantiate";
    #[derive(
        Arbitrary, Clone, Debug, ManifestSbor, ScryptoSbor, serde::Serialize, serde::Deserialize,
    )]
    pub struct RoleAssignmentFuzzBlueprintInstantiateInput {
        pub creation_invocation: RoleAssignmentCreateInput,
        pub pre_attachment_invocations: Vec<RoleAssignmentMethodInvocation>,
    }
    pub type RoleAssignmentFuzzBlueprintInstantiateOutput = ();

    /// Creates a new test-runner with this package published to it and ready for use.
    pub fn test_runner() -> (
        TestRunner<OverridePackageCode<TestInvoke>, InMemorySubstateDatabase>,
        PackageAddress,
    ) {
        let mut test_runner = TestRunnerBuilder::new()
            .with_custom_extension(OverridePackageCode::new(PACKAGE_CODE_ID, TestInvoke))
            .without_trace()
            .build_from_snapshot(TEST_RUNNER_SNAPSHOT.0.clone());
        let package_address = test_runner
            .publish_native_package(PACKAGE_CODE_ID, RoleAssignmentFuzzBlueprint::definition());
        (test_runner, package_address)
    }

    lazy_static::lazy_static! {
        static ref TEST_RUNNER_SNAPSHOT: (TestRunnerSnapshot, PackageAddress) = {
            let mut test_runner = TestRunnerBuilder::new()
                .with_custom_extension(OverridePackageCode::new(PACKAGE_CODE_ID, TestInvoke))
                .without_trace()
                .build();
            let package_address = test_runner
                .publish_native_package(PACKAGE_CODE_ID, RoleAssignmentFuzzBlueprint::definition());
            (test_runner.create_snapshot(), package_address)
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
    fn test_role_assignment_generate_fuzz_input_data() {
        for (index, input) in gen_random(7).into_iter().enumerate() {
            std::fs::write(
                format!("role_assignment_{:03?}.raw", index),
                bincode::serialize(&input).unwrap(),
            )
            .unwrap();
        }
    }

    fn gen_random(n: usize) -> Vec<RoleAssignmentFuzzerInput> {
        let mut vec = Vec::new();

        while vec.len() < n {
            let mut rng = ChaCha8Rng::from_entropy();
            let mut bytes = [0u8; 1024 * 10];
            rng.fill_bytes(&mut bytes);
            let mut unstructured = Unstructured::new(&bytes);
            let input = RoleAssignmentFuzzerInput::arbitrary(&mut unstructured).unwrap();
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
        if TestTransaction::new_from_nonce(manifest.clone(), 0)
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
