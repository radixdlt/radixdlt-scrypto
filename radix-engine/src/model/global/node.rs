use crate::model::GlobalAddressSubstate;
use radix_engine_lib::engine::types::RENodeId;

#[derive(Debug)]
pub struct GlobalRENode {
    pub address: GlobalAddressSubstate,
}

impl GlobalRENode {
    pub fn node_deref(&self) -> RENodeId {
        self.address.node_deref()
    }
}
