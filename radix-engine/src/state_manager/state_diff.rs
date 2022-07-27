use sbor::rust::collections::*;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::buffer::scrypto_encode;
use scrypto::crypto::hash;

use crate::engine::Address;
use crate::ledger::*;
use crate::state_manager::CommitReceipt;

#[derive(Debug, Clone, Hash, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct VirtualSubstateId(pub Address, pub Vec<u8>);

#[derive(Debug, TypeId, Encode, Decode)]
pub struct StateDiff {
    pub down_virtual_substates: Vec<VirtualSubstateId>,
    pub up_substates: BTreeMap<Address, OutputValue>,
    pub down_substates: Vec<OutputId>,
}

impl StateDiff {
    pub fn new() -> Self {
        Self {
            up_substates: BTreeMap::new(),
            down_virtual_substates: Vec::new(),
            down_substates: Vec::new(),
        }
    }

    /// Applies the state changes to some substate store.
    pub fn commit<S: WriteableSubstateStore>(&self, store: &mut S) -> CommitReceipt {
        let mut receipt = CommitReceipt::new();

        for virtual_substate_id in &self.down_virtual_substates {
            receipt.virtual_down(virtual_substate_id.clone());
        }

        for output_id in &self.down_substates {
            receipt.down(output_id.clone());
        }
        for (address, output_value) in &self.up_substates {
            let output_id = OutputId {
                address: address.clone(),
                substate_hash: hash(scrypto_encode(&output_value.substate)),
                version: output_value.version,
            };
            receipt.up(output_id);
            store.put_substate(address.clone(), output_value.clone());
        }
        receipt
    }
}
