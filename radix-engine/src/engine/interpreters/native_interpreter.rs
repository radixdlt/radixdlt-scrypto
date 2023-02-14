use crate::engine::*;
use radix_engine_interface::api::types::RENodeId;

pub fn deref_and_update<D: ResolverApi>(
    receiver: RENodeId,
    call_frame_update: &mut CallFrameUpdate,
    deref: &mut D,
) -> Result<ResolvedReceiver, RuntimeError> {
    // TODO: Move this logic into kernel
    let resolved_receiver = if let Some((derefed, derefed_lock)) = deref.deref(receiver)? {
        ResolvedReceiver::derefed(derefed, receiver, derefed_lock)
    } else {
        ResolvedReceiver::new(receiver)
    };
    let resolved_node_id = resolved_receiver.receiver;
    call_frame_update.node_refs_to_copy.insert(resolved_node_id);

    Ok(resolved_receiver)
}
