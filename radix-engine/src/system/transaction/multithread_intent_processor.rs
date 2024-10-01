use crate::blueprints::transaction_processor::{
    IntentProcessor, ResumeResult, MAX_TOTAL_BLOB_SIZE_PER_INVOCATION,
};
use crate::errors::{KernelError, RuntimeError, SystemError};
use crate::internal_prelude::YieldError;
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::actor::{Actor, FunctionActor};
use crate::system::system::SystemService;
use crate::system::system_callback::{System, SystemBasedKernelApi};
use crate::system::system_modules::auth::AuthModule;
use radix_common::constants::{RESOURCE_PACKAGE, TRANSACTION_PROCESSOR_PACKAGE};
use radix_common::prelude::{BlueprintId, GlobalAddressReservation};
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use radix_engine_interface::prelude::{
    ObjectInfo, FUNGIBLE_PROOF_BLUEPRINT, NON_FUNGIBLE_PROOF_BLUEPRINT,
};
use radix_engine_interface::types::IndexedScryptoValue;
use radix_rust::prelude::*;
use radix_transactions::model::{ExecutableTransaction, InstructionV2};
use sbor::prelude::ToString;

/// Multi-thread intent processor for executing multiple subintents
pub struct MultiThreadIntentProcessor {
    pub threads: Vec<(IntentProcessor<InstructionV2>, Vec<usize>)>,
}

impl MultiThreadIntentProcessor {
    pub fn init<Y: SystemBasedKernelApi>(
        executable: ExecutableTransaction,
        global_address_reservations: Vec<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, RuntimeError> {
        let mut txn_processors = vec![];

        // Setup
        for (thread_id, intent) in executable.all_intents().enumerate() {
            api.kernel_switch_stack(thread_id)?;

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
                intent.encoded_instructions.clone(),
                global_address_reservations.clone(),
                intent.blobs.clone(),
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
                    return Err(RuntimeError::SystemError(SystemError::YieldError(
                        YieldError::CannotYieldProof,
                    )));
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn execute<Y: SystemBasedKernelApi>(&mut self, api: &mut Y) -> Result<(), RuntimeError> {
        let mut cur_thread = 0;
        let mut parent_stack = vec![];
        let mut passed_value = None;

        loop {
            api.kernel_switch_stack(cur_thread)?;
            let (txn_thread, children_mapping) = self.threads.get_mut(cur_thread).unwrap();

            let mut system_service = SystemService::new(api);
            match txn_thread.resume(passed_value.take(), &mut system_service)? {
                ResumeResult::YieldToChild(child, value) => {
                    let child = *children_mapping.get(child).unwrap();
                    parent_stack.push(cur_thread);
                    cur_thread = child;
                    passed_value = Some(value);
                }
                ResumeResult::YieldToParent(value) => {
                    cur_thread = parent_stack.pop().unwrap();
                    passed_value = Some(value);
                }
                ResumeResult::RootIntentDone => {
                    if let Some(parent) = parent_stack.pop() {
                        cur_thread = parent;
                    } else {
                        break;
                    }
                }
            }

            // Checked passed values
            if let Some(passed_value) = &passed_value {
                Self::check_yielded_value(passed_value, api)?;
                api.kernel_send_to_stack(cur_thread, passed_value.clone())?;
            }
        }

        assert!(parent_stack.is_empty());

        Ok(())
    }

    pub fn cleanup<Y: SystemBasedKernelApi>(self, api: &mut Y) -> Result<(), RuntimeError> {
        for (thread_id, _intent) in self.threads.iter().enumerate() {
            api.kernel_switch_stack(thread_id)?;

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
                    owned_nodes,
                )));
            }
        }

        Ok(())
    }
}
