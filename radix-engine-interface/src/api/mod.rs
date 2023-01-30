pub mod blueprints;
pub mod component;
pub mod kernel_modules; // TODO: remove
pub mod node_modules;
pub mod package;
pub mod scrypto_invocation;
pub mod serialize;
pub mod static_link;
pub mod types;

// re-export
pub use static_link::{Invocation, Invokable, InvokableModel};

use crate::api::scrypto_invocation::ScryptoReceiver;
use crate::data::IndexedScryptoValue;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use types::*;

pub trait EngineApi<E: Debug> {
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, E>;
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), E>;
    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, E>;
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, E>;
    fn sys_read(&mut self, lock_handle: LockHandle) -> Result<Vec<u8>, E>;
    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), E>;
    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), E>;
}

pub trait ActorApi<E: Debug> {
    fn fn_identifier(&mut self) -> Result<FnIdentifier, E>;
}

pub trait ComponentApi<E> {
    fn invoke_method(
        &mut self,
        receiver: ScryptoReceiver,
        method_name: &str,
        args: Vec<u8>,
    ) -> Result<IndexedScryptoValue, E>;
}
