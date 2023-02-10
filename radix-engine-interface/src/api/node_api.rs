use crate::api::types::*;
use sbor::rust::fmt::Debug;

pub trait ClientNodeApi<E: Debug> {
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), E>;
}
