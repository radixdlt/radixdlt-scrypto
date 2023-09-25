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

#![cfg_attr(feature = "libfuzzer-sys", no_main)]

use arbitrary::Arbitrary;
#[cfg(feature = "libfuzzer-sys")]
use libfuzzer_sys::fuzz_target;

use radix_engine::blueprints::util::check_name;
use radix_engine::errors::{RejectionReason, RuntimeError, VmError};
use radix_engine::system::attached_modules::role_assignment::*;
use radix_engine::system::system_db_reader::*;
use radix_engine::transaction::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::prelude::*;
use radix_engine_stores::memory_db::*;
use transaction::prelude::*;

#[cfg(feature = "libfuzzer-sys")]
fuzz_target!(|input: RoleAssignmentFuzzerInput| {
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
    // function and stop fuzzing, there is not much to check here. If the transaction is rejected or
    // aborted then we panic as that is not meant to happen.
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
    check_invariants(test_runner.substate_db(), component_address, &[receipt]);

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
    check_invariants(
        test_runner.substate_db(),
        component_address,
        receipts.as_slice(),
    );
});

fn check_invariants(
    substate_store: &InMemorySubstateDatabase,
    component_address: ComponentAddress,
    receipts: &[TransactionReceipt],
) {
    let reader = SystemDatabaseReader::new(substate_store);

    // Verify the owner role information
    {
        let OwnerRoleSubstate {
            owner_role_entry: OwnerRoleEntry { rule, .. },
        } = reader
            .read_object_field(
                component_address.as_node_id(),
                ModuleId::RoleAssignment,
                RoleAssignmentField::Owner.field_index(),
            )
            .expect("Failed to read Owner field of RoleAssignment module.")
            .as_typed::<RoleAssignmentOwnerFieldPayload>()
            .expect("Failed to decode the contents of the owner field")
            .into_latest();
        if RoleAssignmentNativePackage::verify_access_rule(&rule).is_err() {
            panic!("Owner access rule does not respect the access rules max depth and width limits. OwnerRule: {rule:?}");
        }
    }

    // Verify the RoleAssignment collection that stores the ModuleRoleKey -> AccessRule mapping.
    {
        let iter = reader
            .collection_iter(
                component_address.as_node_id(),
                ModuleId::RoleAssignment,
                RoleAssignmentCollection::AccessRuleKeyValue.collection_index(),
            )
            .expect("Failed to read the collection information of the role assignment module");

        for (substate_key, substate_value) in iter {
            let SubstateKey::Map(map_key) = substate_key
        else {
            panic!("Encountered a collection that doesn't have a MapKey!")
        };
            let module_role_key = scrypto_decode::<ModuleRoleKey>(&map_key).unwrap();
            let access_rule =
                scrypto_decode::<RoleAssignmentAccessRuleEntryPayload>(&substate_value)
                    .unwrap()
                    .into_latest();

            let mut error_messages = Vec::<&'static str>::new();
            if RoleAssignmentNativePackage::is_reserved_role_key(&module_role_key.key) {
                error_messages.push("Encountered a reserved role key in the RoleAssignment state");
            }
            if module_role_key.module == ModuleId::RoleAssignment {
                error_messages.push("Encountered a role in a reserved space");
            }
            if module_role_key.key.key.len() > MAX_ROLE_NAME_LEN {
                error_messages.push("Encountered a role with a name longer than allowed");
            }
            if check_name(&module_role_key.key.key).is_err() {
                error_messages
                    .push("Encountered a role name that does not pass the `check_name` checks.");
            }
            if RoleAssignmentNativePackage::verify_access_rule(&access_rule).is_err() {
                error_messages.push(
                "Encountered an access rule that does not comply with the depth and width limits."
            )
            }

            if !error_messages.is_empty() {
                panic!("Messages: {error_messages:?}. ModuleRoleKey: {module_role_key:?}. AccessRule: {access_rule:?}");
            }
        }
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

#[derive(Arbitrary, Clone, Debug)]
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

#[derive(Arbitrary, Clone, Debug, ManifestSbor, ScryptoSbor)]
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
