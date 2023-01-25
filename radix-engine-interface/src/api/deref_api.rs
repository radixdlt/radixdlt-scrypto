use crate::api::types::*;

// TODO: Clean this up
pub trait ClientDerefApi<E> {
    fn deref(&mut self, node_id: RENodeId) -> Result<Option<(RENodeId, LockHandle)>, E>;
}
