use crate::kernel::actor::Actor;
use crate::track::interface::StoreAccessInfo;
use crate::types::*;
use radix_engine_interface::*;

use super::FeeTable;

#[derive(Debug, IntoStaticStr)]
pub enum CostingEntry<'a> {
    // FIXME: Add test to verify each entry

    /* TX */
    TxBaseCost,
    TxPayloadCost {
        size: usize,
    },
    TxSignatureVerification {
        num_signatures: usize,
    },

    /* execution */
    RunNativeCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
    },
    RunWasmCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        gas: u32,
    },

    /* invoke */
    Invoke {
        actor: &'a Actor,
        input_size: usize,
    },

    /* node */
    AllocateNodeId,
    CreateNode {
        node_id: &'a NodeId,
        total_substate_size: usize,
        store_access: &'a StoreAccessInfo,
    },
    DropNode {
        total_substate_size: usize,
    },
    MoveModules,
    OpenSubstate {
        value_size: usize,
        store_access: &'a StoreAccessInfo,
    },
    ReadSubstate {
        value_size: usize,
        store_access: &'a StoreAccessInfo,
    },
    WriteSubstate {
        value_size: usize,
        store_access: &'a StoreAccessInfo,
    },
    CloseSubstate {
        store_access: &'a StoreAccessInfo,
    },

    /* unstable node apis */
    SetSubstate {
        store_access: &'a StoreAccessInfo,
    },
    RemoveSubstate {
        store_access: &'a StoreAccessInfo,
    },
    ScanSortedSubstates {
        store_access: &'a StoreAccessInfo,
    },
    ScanSubstates {
        store_access: &'a StoreAccessInfo,
    },
    TakeSubstate {
        store_access: &'a StoreAccessInfo,
    },

    /* system */
    LockFee,
    QueryFeeReserve,
    QueryActor,
    QueryAuthZone,
    AssertAccessRule,
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

    /* system modules */
    RoyaltyModule {
        direct_charge: u32,
    },
    AuthModule {
        direct_charge: u32,
    },
}

#[inline]
pub fn checked_mul_or_max(base: u32, multiplier: usize) -> u32 {
    u32::try_from(multiplier)
        .ok()
        .and_then(|x| base.checked_mul(x))
        .unwrap_or(u32::MAX)
}

impl<'a> CostingEntry<'a> {
    pub fn to_cost_units(&self, ft: &FeeTable) -> u32 {
        match self {
            CostingEntry::TxBaseCost => ft.tx_base_cost(),
            CostingEntry::TxPayloadCost { size } => {
                checked_mul_or_max(ft.tx_payload_cost_per_byte(), *size)
            }
            CostingEntry::TxSignatureVerification { num_signatures } => {
                checked_mul_or_max(ft.tx_signature_verification_cost_per_sig(), *num_signatures)
            }
            CostingEntry::RunNativeCode {
                package_address,
                export_name,
            } => ft.run_native_code_cost(package_address, export_name),
            CostingEntry::RunWasmCode {
                package_address,
                export_name,
                gas,
            } => ft.run_wasm_code_cost(package_address, export_name, *gas),
            CostingEntry::Invoke { actor, input_size } => ft.invoke_cost(actor, *input_size),
            CostingEntry::AllocateNodeId => ft.allocate_node_id_cost(),
            CostingEntry::CreateNode {
                node_id,
                total_substate_size,
                store_access,
            } => ft.create_node_cost(node_id, *total_substate_size, store_access),
            CostingEntry::DropNode {
                total_substate_size,
            } => ft.drop_node_cost(*total_substate_size),
            CostingEntry::MoveModules => ft.move_modules_cost(),
            CostingEntry::OpenSubstate {
                value_size,
                store_access,
            } => ft.open_substate_cost(*value_size, store_access),
            CostingEntry::ReadSubstate {
                value_size,
                store_access,
            } => ft.read_substate_cost(*value_size, store_access),
            CostingEntry::WriteSubstate {
                value_size,
                store_access,
            } => ft.write_substate_cost(*value_size, store_access),
            CostingEntry::CloseSubstate { store_access } => ft.close_substate_cost(store_access),
            CostingEntry::SetSubstate { store_access } => ft.set_substate_cost(store_access),
            CostingEntry::RemoveSubstate { store_access } => ft.remove_substate_cost(store_access),
            CostingEntry::ScanSortedSubstates { store_access } => {
                ft.scan_sorted_substates_cost(store_access)
            }
            CostingEntry::ScanSubstates { store_access } => ft.scan_substates_cost(store_access),
            CostingEntry::TakeSubstate { store_access } => ft.take_substates_cost(store_access),
            CostingEntry::LockFee => ft.lock_fee_cost(),
            CostingEntry::QueryFeeReserve => ft.query_fee_reserve_cost(),
            CostingEntry::QueryActor => ft.query_actor_cost(),
            CostingEntry::QueryAuthZone => ft.query_auth_zone_cost(),
            CostingEntry::AssertAccessRule => ft.assert_access_rule_cost(),
            CostingEntry::QueryTransactionHash => ft.query_transaction_hash_cost(),
            CostingEntry::GenerateRuid => ft.generate_ruid_cost(),
            CostingEntry::EmitEvent { size } => ft.emit_event_cost(*size),
            CostingEntry::EmitLog { size } => ft.emit_log_cost(*size),
            CostingEntry::Panic { size } => ft.panic_cost(*size),
            CostingEntry::RoyaltyModule { direct_charge } => *direct_charge,
            CostingEntry::AuthModule { direct_charge } => *direct_charge,
        }
    }
}
