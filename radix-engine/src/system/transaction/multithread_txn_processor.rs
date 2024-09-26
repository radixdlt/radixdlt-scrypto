use crate::blueprints::transaction_processor::{
    ResumeResult, TxnProcessorThread, MAX_TOTAL_BLOB_SIZE_PER_INVOCATION,
};
use crate::errors::{KernelError, RuntimeError};
use crate::kernel::kernel_callback_api::KernelCallbackObject;
use crate::system::actor::{Actor, FunctionActor};
use crate::system::system::SystemService;
use crate::system::system_callback::{System, SystemBasedKernelApi};
use crate::system::system_modules::auth::AuthModule;
use radix_common::constants::TRANSACTION_PROCESSOR_PACKAGE;
use radix_common::prelude::{BlueprintId, GlobalAddressReservation};
use radix_engine_interface::blueprints::transaction_processor::{
    TRANSACTION_PROCESSOR_BLUEPRINT, TRANSACTION_PROCESSOR_RUN_IDENT,
};
use radix_rust::prelude::*;
use radix_transactions::model::{ExecutableTransaction, InstructionV2};
use sbor::prelude::ToString;

/// Multi-thread transaction processor for executing multiple subintents
pub struct MultiThreadedTxnProcessor {
    pub threads: Vec<(TxnProcessorThread<InstructionV2>, Vec<usize>)>,
}

impl MultiThreadedTxnProcessor {
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
            let virtual_resources = intent
                .auth_zone_init
                .simulate_every_proof_under_resources
                .clone();
            let virtual_non_fungibles =
                intent.auth_zone_init.initial_non_fungible_id_proofs.clone();
            let auth_zone = AuthModule::create_auth_zone(
                &mut system_service,
                None,
                virtual_resources,
                virtual_non_fungibles,
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
            let txn_processor = TxnProcessorThread::<InstructionV2>::init(
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

    pub fn execute<Y: SystemBasedKernelApi>(&mut self, api: &mut Y) -> Result<(), RuntimeError> {
        let mut cur_thread = 0;
        let mut parent_stack = vec![];

        loop {
            api.kernel_switch_stack(cur_thread)?;
            let (txn_thread, children_mapping) = self.threads.get_mut(cur_thread).unwrap();

            let mut system_service = SystemService::new(api);
            match txn_thread.resume(&mut system_service)? {
                ResumeResult::YieldToChild(child) => {
                    let child = *children_mapping.get(child).unwrap();
                    parent_stack.push(cur_thread);
                    cur_thread = child;
                }
                ResumeResult::YieldToParent => {
                    cur_thread = parent_stack.pop().unwrap();
                }
                ResumeResult::Done => {
                    if let Some(parent) = parent_stack.pop() {
                        cur_thread = parent;
                    } else {
                        break;
                    }
                }
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
