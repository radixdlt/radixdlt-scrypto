use super::FeeTable;
use crate::kernel::actor::Actor;
use crate::kernel::kernel_callback_api::{
    CloseSubstateEvent, CreateNodeEvent, DrainSubstatesEvent, DropNodeEvent, MoveModuleEvent,
    OpenSubstateEvent, ReadSubstateEvent, RemoveSubstateEvent, ScanKeysEvent,
    ScanSortedSubstatesEvent, SetSubstateEvent, WriteSubstateEvent,
};
use crate::system::system_modules::transaction_runtime::Event;
use crate::track::interface::StoreCommit;
use crate::types::*;
use radix_engine_interface::*;

#[derive(Debug, IntoStaticStr)]
pub enum ExecutionCostingEntry<'a> {
    /* verify signature */
    VerifySignature {
        num_signatures: usize,
    },

    /* run code */
    RunNativeCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        input_size: usize,
    },
    RunWasmCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        wasm_execution_units: u32,
    },
    PrepareWasmCode {
        size: usize,
    },

    /* invoke */
    BeforeInvoke {
        actor: &'a Actor,
        input_size: usize,
    },
    AfterInvoke {
        output_size: usize,
    },

    /* node */
    AllocateNodeId,
    CreateNode {
        event: &'a CreateNodeEvent<'a>,
    },
    DropNode {
        event: &'a DropNodeEvent<'a>,
    },
    MoveModule {
        event: &'a MoveModuleEvent<'a>,
    },
    OpenSubstate {
        event: &'a OpenSubstateEvent<'a>,
    },
    ReadSubstate {
        event: &'a ReadSubstateEvent<'a>,
    },
    WriteSubstate {
        event: &'a WriteSubstateEvent<'a>,
    },
    CloseSubstate {
        event: &'a CloseSubstateEvent,
    },

    /* unstable node apis */
    SetSubstate {
        event: &'a SetSubstateEvent<'a>,
    },
    RemoveSubstate {
        event: &'a RemoveSubstateEvent<'a>,
    },
    ScanKeys {
        event: &'a ScanKeysEvent<'a>,
    },
    ScanSortedSubstates {
        event: &'a ScanSortedSubstatesEvent<'a>,
    },
    DrainSubstates {
        event: &'a DrainSubstatesEvent<'a>,
    },

    /* system */
    LockFee,
    QueryFeeReserve,
    QueryActor,
    QueryAuthZone,
    QueryTransactionHash,
    GenerateRuid,
    EmitEvent {
        size: usize,
    },
    EmitLog {
        size: usize,
    },
    Panic {
        size: usize,
    },
}

#[derive(Debug, IntoStaticStr)]
pub enum FinalizationCostingEntry<'a> {
    TransactionBase,
    TransactionPayload { size: usize },
    CommitStates { store_commit: &'a StoreCommit },
    CommitEvents { events: &'a Vec<Event> },
    CommitLogs { logs: &'a Vec<(Level, String)> },
}

impl<'a> ExecutionCostingEntry<'a> {
    pub fn to_execution_cost_units(&self, ft: &FeeTable) -> u32 {
        match self {
            ExecutionCostingEntry::VerifySignature { num_signatures } => {
                ft.tx_signature_verification_cost(*num_signatures)
            }
            ExecutionCostingEntry::RunNativeCode {
                package_address,
                export_name,
                input_size,
            } => ft.run_native_code_cost(package_address, export_name, input_size),
            ExecutionCostingEntry::RunWasmCode {
                package_address,
                export_name,
                wasm_execution_units,
            } => ft.run_wasm_code_cost(package_address, export_name, *wasm_execution_units),
            ExecutionCostingEntry::PrepareWasmCode { size } => ft.instantiate_wasm_code_cost(*size),
            ExecutionCostingEntry::BeforeInvoke { actor, input_size } => {
                ft.before_invoke_cost(actor, *input_size)
            }
            ExecutionCostingEntry::AfterInvoke { output_size } => {
                ft.after_invoke_cost(*output_size)
            }
            ExecutionCostingEntry::AllocateNodeId => ft.allocate_node_id_cost(),
            ExecutionCostingEntry::CreateNode { event } => ft.create_node_cost(event),
            ExecutionCostingEntry::DropNode { event } => ft.drop_node_cost(event),
            ExecutionCostingEntry::MoveModule { event } => ft.move_module_cost(event),
            ExecutionCostingEntry::OpenSubstate { event } => ft.open_substate_cost(event),
            ExecutionCostingEntry::ReadSubstate { event } => ft.read_substate_cost(event),
            ExecutionCostingEntry::WriteSubstate { event } => ft.write_substate_cost(event),
            ExecutionCostingEntry::CloseSubstate { event } => ft.close_substate_cost(event),
            ExecutionCostingEntry::SetSubstate { event } => ft.set_substate_cost(event),
            ExecutionCostingEntry::RemoveSubstate { event } => ft.remove_substate_cost(event),
            ExecutionCostingEntry::ScanKeys { event } => ft.scan_keys_cost(event),
            ExecutionCostingEntry::DrainSubstates { event } => ft.drain_substates_cost(event),
            ExecutionCostingEntry::ScanSortedSubstates { event } => {
                ft.scan_sorted_substates_cost(event)
            }
            ExecutionCostingEntry::LockFee => ft.lock_fee_cost(),
            ExecutionCostingEntry::QueryFeeReserve => ft.query_fee_reserve_cost(),
            ExecutionCostingEntry::QueryActor => ft.query_actor_cost(),
            ExecutionCostingEntry::QueryAuthZone => ft.query_auth_zone_cost(),
            ExecutionCostingEntry::QueryTransactionHash => ft.query_transaction_hash_cost(),
            ExecutionCostingEntry::GenerateRuid => ft.generate_ruid_cost(),
            ExecutionCostingEntry::EmitEvent { size } => ft.emit_event_cost(*size),
            ExecutionCostingEntry::EmitLog { size } => ft.emit_log_cost(*size),
            ExecutionCostingEntry::Panic { size } => ft.panic_cost(*size),
        }
    }
}

impl<'a> FinalizationCostingEntry<'a> {
    pub fn to_finalization_cost_units(&self, ft: &FeeTable) -> u32 {
        match self {
            FinalizationCostingEntry::TransactionBase => ft.transaction_base_cost(),
            FinalizationCostingEntry::TransactionPayload { size } => {
                ft.transaction_payload_cost(*size)
            }
            FinalizationCostingEntry::CommitStates { store_commit } => {
                ft.commit_states_cost(store_commit)
            }
            FinalizationCostingEntry::CommitEvents { events } => ft.commit_events_cost(events),
            FinalizationCostingEntry::CommitLogs { logs } => ft.commit_logs_cost(logs),
        }
    }
}

impl<'a> ExecutionCostingEntry<'a> {
    pub fn to_trace_key(&self) -> String {
        match self {
            ExecutionCostingEntry::RunNativeCode { export_name, .. } => {
                format!("RunNativeCode::{}", export_name)
            }
            ExecutionCostingEntry::RunWasmCode { export_name, .. } => {
                format!("RunWasmCode::{}", export_name)
            }
            ExecutionCostingEntry::OpenSubstate {
                event: OpenSubstateEvent::End { node_id, .. },
                ..
            } => {
                format!(
                    "OpenSubstate::{}",
                    node_id.entity_type().map(|x| x.into()).unwrap_or("?")
                )
            }
            x => Into::<&'static str>::into(x).to_string(),
        }
    }
}

impl<'a> FinalizationCostingEntry<'a> {
    pub fn to_trace_key(&self) -> String {
        match self {
            FinalizationCostingEntry::CommitStates { store_commit } => {
                format!(
                    "CommitStates::{}",
                    store_commit
                        .node_id()
                        .entity_type()
                        .map(|x| x.into())
                        .unwrap_or("?")
                )
            }
            x => Into::<&'static str>::into(x).to_string(),
        }
    }
}
