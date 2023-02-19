use crate::errors::*;
use crate::kernel::actor::ResolvedReceiver;
use crate::kernel::call_frame::CallFrameUpdate;
use radix_engine_interface::api::types::MethodReceiver;
use radix_engine_interface::api::ClientDerefApi;

pub fn deref_and_update<D: ClientDerefApi<RuntimeError>>(
    receiver: MethodReceiver,
    call_frame_update: &mut CallFrameUpdate,
    deref: &mut D,
) -> Result<ResolvedReceiver, RuntimeError> {
    // TODO: Move this logic into kernel
    let resolved_receiver = if let Some((derefed, derefed_lock)) = deref.deref(receiver.0)? {
        ResolvedReceiver::derefed(MethodReceiver(derefed, receiver.1), receiver.0, derefed_lock)
    } else {
        ResolvedReceiver::new(receiver)
    };
    let resolved_node_id = resolved_receiver.receiver;
    call_frame_update
        .node_refs_to_copy
        .insert(resolved_node_id.0);

    Ok(resolved_receiver)
}
