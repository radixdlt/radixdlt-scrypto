use super::call_frame::CallFrameUpdate;
use super::kernel_api::KernelNodeApi;
use super::kernel_api::KernelSubstateApi;
use super::kernel_api::KernelWasmApi;
use crate::errors::*;
use crate::kernel::actor::Actor;
use crate::kernel::kernel_api::KernelInternalApi;
use crate::types::*;
use crate::wasm::WasmEngine;
use radix_engine_interface::api::*;
use crate::system::invoke::ScryptoExecutor;

#[derive(Debug)]
pub struct KernelInvocation {
    pub executor: ScryptoExecutor,

    // TODO: Remove
    pub payload_size: usize,

    // TODO: Make these two RENodes / Substates
    pub resolved_actor: Actor,
    pub args: IndexedScryptoValue,
}

impl KernelInvocation {
    pub fn get_update(&self) -> CallFrameUpdate {
        let nodes_to_move = self.args.owned_node_ids().clone();
        let mut node_refs_to_copy = self.args.references().clone();
        match self.resolved_actor {
            Actor::Method { node_id, .. } => {
                node_refs_to_copy.insert(node_id);
            }
            Actor::Function { .. } | Actor::VirtualLazyLoad { .. } => {
            }
        }

        CallFrameUpdate {
            nodes_to_move,
            node_refs_to_copy
        }
    }
}
