use crate::api::types::*;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub trait ClientNodeApi<E: Debug> {
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, E>;
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), E>;
    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, E>;
}
