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
use radix_engine::errors::{RejectionReason, RuntimeError, VmError};
use radix_engine::system::checkers::*;
use radix_engine::transaction::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::prelude::*;
use radix_engine_stores::memory_db::*;
use transaction::prelude::*;

fuzz_template!(|input: RoleAssignmentFuzzerInput| {
    // Getting the roles that we would have if the creation invocation is valid and the creation tx
    // succeeds
    let role_keys = input.initial_role_keys();

    // Creating a new test-runner. This test runner has the appropriate test package published and
    // ready for us to use in fuzzing
    let (mut test_runner, package_address) = package::test_runner();

    // Instantiate a new role-assignment test component and get the component address.
    let receipt = test_runner.execute_manifest(
        ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                package_address,
                package::BLUEPRINT_IDENT,
                package::ROLE_ASSIGNMENT_FUZZ_BLUEPRINT_INSTANTIATE_IDENT,
                package::RoleAssignmentFuzzBlueprintInstantiateInput {
                    creation_invocation: input.creation_invocation,
                    pre_attachment_invocations: input.pre_attachment_invocations,
                },
            )
            .build(),
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
    check_invariants(test_runner.substate_db(), &[receipt], role_keys.clone());

    // Perform the method invocations to the role-assignment module. Each invocation is its own
    // transaction. This is because we would like for a failed invocation not to stop other ones
    // from happening.
    let receipts = input
        .post_attachment_invocations
        .into_iter()
        .map(|invocation| {
            let manifest = ManifestBuilder::new()
                .lock_fee_from_faucet()
                .call_role_assignment_method(
                    component_address,
                    invocation.method_ident(),
                    ManifestArgs::new_from_tuple_or_panic(invocation.manifest_value()),
                )
                .build();
            test_runner.execute_manifest(manifest, vec![])
        })
        .collect::<Vec<_>>();

    // Do the second check of the invariants
    check_invariants(test_runner.substate_db(), receipts.as_slice(), role_keys);
});

fn check_invariants(
    substate_database: &InMemorySubstateDatabase,
    receipts: &[TransactionReceipt],
    initial_roles: Vec<ModuleRoleKey>,
) {
    // Verifying the role-assignment substates
    let mut checker =
        SystemDatabaseChecker::new(RoleAssignmentDatabaseChecker::new(Some(initial_roles)));
    let (_, results) = checker
        .check_db(substate_database)
        .expect("Should not fail!");

    if !results.is_empty() {
        panic!("Found violations in the database: {:?}", results);
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
    #[derive(Arbitrary, Clone, Debug, ManifestSbor, ScryptoSbor)]
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
            .build();
        let package_address = test_runner
            .publish_native_package(PACKAGE_CODE_ID, RoleAssignmentFuzzBlueprint::definition());
        (test_runner, package_address)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_role_assignment_generate_fuzz_input_data() {
        let example_inputs = vec![
            RoleAssignmentFuzzerInput {
                creation_invocation: RoleAssignmentCreateInput {
                    owner_role: OwnerRoleEntry {
                        rule: rule!(allow_all),
                        updater: OwnerRoleUpdater::None,
                    },
                    roles: Default::default(),
                },
                pre_attachment_invocations: Default::default(),
                post_attachment_invocations: Default::default(),
            },
            RoleAssignmentFuzzerInput {
                creation_invocation: RoleAssignmentCreateInput {
                    owner_role: OwnerRoleEntry {
                        rule: rule!(allow_all),
                        updater: OwnerRoleUpdater::None,
                    },
                    roles: Default::default(),
                },
                pre_attachment_invocations: Default::default(),
                post_attachment_invocations: Default::default(),
            },
        ];

        for (index, input) in example_inputs.into_iter().enumerate() {
            std::fs::write(
                format!("role_assignment_{:03?}.raw", index),
                bincode::serialize(&input).unwrap(),
            )
            .unwrap();
        }
    }
}
