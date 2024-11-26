use crate::blueprints::resource::AuthZone;
use crate::blueprints::transaction_processor::{
    IntentProcessor, ResumeResult, MAX_TOTAL_BLOB_SIZE_PER_INVOCATION,
};
use crate::errors::{KernelError, RuntimeError, SystemError};
use crate::internal_prelude::*;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::actor::{Actor, FunctionActor};
use crate::system::node_init::type_info_partition;
use crate::system::system::SystemService;
use crate::system::system_callback::{System, SystemBasedKernelApi};
use crate::system::system_modules::auth::{AuthModule, Authorization, AuthorizationCheckResult};
use crate::system::type_info::TypeInfoSubstate;
use radix_common::constants::{RESOURCE_PACKAGE, TRANSACTION_PROCESSOR_PACKAGE};
use radix_common::prelude::{BlueprintId, EntityType, GlobalAddressReservation, Reference};
use radix_common::types::{GlobalCaller, NodeId};
use radix_engine_interface::blueprints::package::BlueprintVersion;
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use radix_engine_interface::prelude::{
    AccessRule, AuthZoneField, BlueprintInfo, ObjectInfo, ObjectType, OuterObjectInfo,
    AUTH_ZONE_BLUEPRINT, FUNGIBLE_PROOF_BLUEPRINT, MAIN_BASE_PARTITION,
    NON_FUNGIBLE_PROOF_BLUEPRINT, TYPE_INFO_FIELD_PARTITION,
};
use radix_rust::prelude::*;
use radix_transactions::model::{ExecutableTransaction, InstructionV2};
use sbor::prelude::ToString;

/// Multi-thread intent processor for executing multiple subintents
pub struct MultiThreadIntentProcessor<'e> {
    pub threads: Vec<(IntentProcessor<'e, InstructionV2>, Vec<usize>)>,
}

impl<'e> MultiThreadIntentProcessor<'e> {
    pub fn init<Y: SystemBasedKernelApi>(
        executable: &'e ExecutableTransaction,
        global_address_reservations: &[GlobalAddressReservation],
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        let mut txn_processors = vec![];

        // Setup
        for (thread_id, intent) in executable.all_intents().enumerate() {
            api.kernel_switch_stack(thread_id)?;

            // We create the auth zone for the transaction processor.
            // This needs a `SystemService` to resolve the current actor (Root) to get the global caller (None),
            // and also to create the node and substates.
            let mut system_service = SystemService::new(api);
            let simulate_every_proof_under_resources = intent
                .auth_zone_init
                .simulate_every_proof_under_resources
                .clone();
            let initial_non_fungible_id_proofs =
                intent.auth_zone_init.initial_non_fungible_id_proofs.clone();
            let auth_zone = AuthModule::create_auth_zone(
                &mut system_service,
                None,
                simulate_every_proof_under_resources,
                initial_non_fungible_id_proofs,
            )?;

            api.kernel_set_call_frame_data(Actor::Function(FunctionActor {
                blueprint_id: BlueprintId::new(
                    &TRANSACTION_PROCESSOR_PACKAGE,
                    TRANSACTION_PROCESSOR_BLUEPRINT,
                ),
                ident: TRANSACTION_PROCESSOR_RUN_IDENT.to_string(),
                auth_zone,
            }))?;

            let mut system_service = SystemService::new(api);
            let txn_processor = IntentProcessor::<InstructionV2>::init(
                intent.encoded_instructions.as_ref(),
                global_address_reservations,
                &intent.blobs,
                MAX_TOTAL_BLOB_SIZE_PER_INVOCATION,
                &mut system_service,
            )?;

            txn_processors.push((
                txn_processor,
                intent
                    .children_subintent_indices
                    .iter()
                    .map(|index| {
                        let thread_index = index.0 + 1;
                        thread_index
                    })
                    .collect::<Vec<_>>(),
            ));
        }
        Ok(Self {
            threads: txn_processors,
        })
    }

    pub fn execute<Y: SystemBasedKernelApi>(&mut self, api: &mut Y) -> Result<(), RuntimeError> {
        let mut cur_thread = 0;
        let mut parent_stack = vec![];
        let mut passed_value = None;

        enum PostExecution {
            SwitchThread(usize, IndexedScryptoValue, bool),
            VerifyParent(AccessRule),
            RootIntentDone,
        }

        loop {
            api.kernel_switch_stack(cur_thread)?;
            let (txn_thread, children_mapping) = self.threads.get_mut(cur_thread).unwrap();

            let mut system_service = SystemService::new(api);
            let post_exec = match txn_thread.resume(passed_value.take(), &mut system_service)? {
                ResumeResult::YieldToChild(child, value) => {
                    let child = *children_mapping
                        .get(child)
                        .ok_or(RuntimeError::SystemError(SystemError::IntentError(
                            IntentError::InvalidIntentIndex(child),
                        )))?;
                    parent_stack.push(cur_thread);
                    PostExecution::SwitchThread(child, value, false)
                }
                ResumeResult::YieldToParent(value) => {
                    let parent = parent_stack.pop().ok_or(RuntimeError::SystemError(
                        SystemError::IntentError(IntentError::NoParentToYieldTo),
                    ))?;
                    PostExecution::SwitchThread(parent, value, false)
                }
                ResumeResult::VerifyParent(rule) => PostExecution::VerifyParent(rule),
                ResumeResult::DoneAndYieldToParent(value) => {
                    let parent = parent_stack.pop().ok_or(RuntimeError::SystemError(
                        SystemError::IntentError(IntentError::NoParentToYieldTo),
                    ))?;
                    PostExecution::SwitchThread(parent, value, true)
                }
                ResumeResult::Done => PostExecution::RootIntentDone,
            };

            match post_exec {
                PostExecution::SwitchThread(next_thread, value, intent_done) => {
                    // Checked passed values
                    Self::check_yielded_value(&value, api)?;
                    api.kernel_send_to_stack(next_thread, &value)?;
                    passed_value = Some(value);

                    // Cleanup stack if intent is done. This must be done after the above kernel_send_to_stack.
                    if intent_done {
                        Self::cleanup_stack(api)?;
                    }

                    cur_thread = next_thread;
                }
                PostExecution::VerifyParent(rule) => {
                    let save_cur_thread = cur_thread;
                    let system_version = api.system_service().system().versioned_system_logic;

                    let parent = if system_version.use_root_for_verify_parent_instruction() {
                        parent_stack.first()
                    } else {
                        // As of CuttlefishPart2, we use the direct parent intent for the VERIFY_PARENT instruction
                        parent_stack.last()
                    };
                    let parent_thread_index = parent.cloned().ok_or(RuntimeError::SystemError(
                        SystemError::IntentError(IntentError::CannotVerifyParentOnRoot),
                    ))?;
                    api.kernel_switch_stack(parent_thread_index)?;

                    // Create a temporary authzone with the current authzone as the global caller since
                    // check_authorization_against_access_rule tests against the global caller authzones.
                    // Run assert_access_rule against this authzone
                    {
                        let auth_zone = Self::create_temp_child_auth_zone_for_verify_parent(api)?;
                        let mut system_service = SystemService::new(api);
                        let auth_result = Authorization::check_authorization_against_access_rule(
                            &mut system_service,
                            &auth_zone,
                            &rule,
                        )?;
                        match auth_result {
                            AuthorizationCheckResult::Authorized => {}
                            AuthorizationCheckResult::Failed(..) => {
                                return Err(RuntimeError::SystemError(SystemError::IntentError(
                                    IntentError::VerifyParentFailed,
                                )))
                            }
                        }
                        api.kernel_drop_node(&auth_zone)?;
                    }

                    api.kernel_switch_stack(save_cur_thread)?;
                }
                PostExecution::RootIntentDone => {
                    Self::cleanup_stack(api)?;
                    break;
                }
            }
        }

        assert!(parent_stack.is_empty());

        Ok(())
    }

    fn create_temp_child_auth_zone_for_verify_parent<Y: SystemBasedKernelApi>(
        api: &mut Y,
    ) -> Result<NodeId, RuntimeError> {
        let actor = api.kernel_get_system_state().current_call_frame;
        let auth_zone = actor.self_auth_zone().unwrap();
        let blueprint_id = actor.blueprint_id().unwrap();
        let auth_zone = AuthZone::new(
            vec![],
            Default::default(),
            Default::default(),
            None,
            Some((
                GlobalCaller::PackageBlueprint(blueprint_id),
                Reference(auth_zone),
            )),
            None,
        );

        let new_auth_zone = api.kernel_allocate_node_id(EntityType::InternalGenericComponent)?;

        api.kernel_create_node(
            new_auth_zone,
            btreemap!(
                MAIN_BASE_PARTITION => btreemap!(
                    AuthZoneField::AuthZone.into() => IndexedScryptoValue::from_typed(&FieldSubstate::new_unlocked_field(auth_zone))
                ),
                TYPE_INFO_FIELD_PARTITION => type_info_partition(TypeInfoSubstate::Object(ObjectInfo {
                    blueprint_info: BlueprintInfo {
                        blueprint_id: BlueprintId::new(&RESOURCE_PACKAGE, AUTH_ZONE_BLUEPRINT),
                        blueprint_version: BlueprintVersion::default(),
                        outer_obj_info: OuterObjectInfo::default(),
                        features: indexset!(),
                        generic_substitutions: vec![],
                    },
                    object_type: ObjectType::Owned,
                }))
            ),
        )?;
        api.kernel_pin_node(new_auth_zone)?;

        Ok(new_auth_zone)
    }

    fn check_yielded_value<Y: SystemBasedKernelApi>(
        value: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(), RuntimeError> {
        let mut system_service = SystemService::new(api);
        for node_id in value.owned_nodes() {
            let object_info: ObjectInfo = system_service.get_object_info(node_id)?;

            let blueprint_id = object_info.blueprint_info.blueprint_id;
            match (
                blueprint_id.package_address,
                blueprint_id.blueprint_name.as_str(),
            ) {
                (RESOURCE_PACKAGE, FUNGIBLE_PROOF_BLUEPRINT | NON_FUNGIBLE_PROOF_BLUEPRINT) => {
                    return Err(RuntimeError::SystemError(SystemError::IntentError(
                        IntentError::CannotYieldProof,
                    )));
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn cleanup_stack<Y: SystemBasedKernelApi>(api: &mut Y) -> Result<(), RuntimeError> {
        let owned_nodes = api.kernel_get_owned_nodes()?;
        System::auto_drop(owned_nodes, api)?;

        let actor = api.kernel_get_system_state().current_call_frame;
        match actor {
            Actor::Function(FunctionActor { auth_zone, .. }) => {
                let auth_zone = auth_zone.clone();
                let mut system_service = SystemService::new(api);
                AuthModule::teardown_auth_zone(&mut system_service, auth_zone)?;
            }
            _ => {
                panic!("unexpected");
            }
        }
        let owned_nodes = api.kernel_get_owned_nodes()?;
        if !owned_nodes.is_empty() {
            return Err(RuntimeError::KernelError(KernelError::OrphanedNodes(
                owned_nodes
                    .into_iter()
                    .map(|node_id| node_id.into())
                    .collect(),
            )));
        }

        Ok(())
    }
}
