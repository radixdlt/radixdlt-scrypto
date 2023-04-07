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
use crate::kernel::interpreters::ScryptoExecutor;

pub trait ExecutableInvocation: Invocation {
    fn resolve<Y: KernelSubstateApi + KernelInternalApi>(
        self,
        api: &mut Y,
    ) -> Result<Box<KernelInvocation>, RuntimeError>;

    fn payload_size(&self) -> usize;
}

pub trait Executor {
    fn execute<Y, W>(
        self,
        args: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<(IndexedScryptoValue, CallFrameUpdate), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + KernelWasmApi<W> + KernelInternalApi + ClientApi<RuntimeError>,
        W: WasmEngine;
}

pub struct KernelInvocation {
    pub executor: ScryptoExecutor,

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
