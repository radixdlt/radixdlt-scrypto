use crate::kernel::actor::Actor;
use crate::track::interface::StoreAccessInfo;
use crate::types::*;
use radix_engine_interface::*;

use super::FeeTable;

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
    },

    /* invoke */
    Invoke {
        actor: &'a Actor,
        input_size: usize,
    },

    /* node */
    AllocateNodeId,
    AllocateGlobalAddress,
    CreateNode {
        node_id: &'a NodeId,
        db_access: &'a StoreAccessInfo,
    },
    DropNode,
    OpenSubstate {
        db_access: &'a StoreAccessInfo,
    },
    ReadSubstate {
        db_access: &'a StoreAccessInfo,
    },
    WriteSubstate {
        db_access: &'a StoreAccessInfo,
    },
    CloseSubstate {
        db_access: &'a StoreAccessInfo,
    },

    /* unstable node apis */
    SetSubstate {
        db_access: &'a StoreAccessInfo,
    },
    RemoveSubstate {
        db_access: &'a StoreAccessInfo,
    },
    ScanSortedSubstates {
        db_access: &'a StoreAccessInfo,
    },
    ScanSubstates {
        db_access: &'a StoreAccessInfo,
    },
    TakeSubstate {
        db_access: &'a StoreAccessInfo,
    },

    /* system */
    Event {
        size: usize,
    },
    Log {
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
    DropLock {},
}

impl<'a> CostingEntry<'a> {
    pub fn to_cost_units(&self, fee_table: &FeeTable) -> u32 {
        todo!()
    }
}
