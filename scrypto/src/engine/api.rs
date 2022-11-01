use crate::buffer::scrypto_encode;
use crate::engine::utils::call_engine;
use crate::resource::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::{Decode, Encode, TypeId};
use scrypto::core::*;
use scrypto::engine::types::*;
use scrypto::resource::{
    AuthZoneCreateProofByIdsInput, AuthZonePopInput, ProofCloneInput, VaultGetAmountInput,
};

use super::types::*;

pub trait SysInvocation: Encode {
    type Output: Debug + Decode;

    fn native_method() -> NativeMethod;
}

pub trait SysInvokable<I, E>
where
    I: SysInvocation,
{
    fn sys_invoke(&mut self, input: I) -> Result<I::Output, E>;
}

pub trait ScryptoSyscalls<E: Debug> {
    fn sys_invoke_scrypto_function<ARGS: Encode, V: Decode>(
        &mut self,
        fn_ident: ScryptoFunctionIdent,
        args: &ARGS,
    ) -> Result<V, E>;
    fn sys_invoke_scrypto_method<ARGS: Encode, V: Decode>(
        &mut self,
        method_ident: ScryptoMethodIdent,
        args: &ARGS,
    ) -> Result<V, E>;
    fn sys_create_node(&mut self, node: ScryptoRENode) -> Result<RENodeId, E>;
    fn sys_drop_node(&mut self, node_id: RENodeId) -> Result<(), E>;
    fn sys_get_visible_nodes(&mut self) -> Result<Vec<RENodeId>, E>;
    fn sys_lock_substate(
        &mut self,
        node_id: RENodeId,
        offset: SubstateOffset,
        mutable: bool,
    ) -> Result<LockHandle, E>;
    fn sys_read<V: Decode>(&mut self, lock_handle: LockHandle) -> Result<V, E>;
    fn sys_write(&mut self, lock_handle: LockHandle, buffer: Vec<u8>) -> Result<(), E>;
    fn sys_drop_lock(&mut self, lock_handle: LockHandle) -> Result<(), E>;
    fn sys_get_actor(&mut self) -> Result<ScryptoActor, E>;
    fn sys_generate_uuid(&mut self) -> Result<u128, E>;
    fn sys_emit_log(&mut self, level: Level, message: String) -> Result<(), E>;
}

pub trait SysInvokableNative<E>:
    SysInvokable<AuthZonePopInput, E>
    + SysInvokable<AuthZonePushInput, E>
    + SysInvokable<AuthZoneCreateProofInput, E>
    + SysInvokable<AuthZoneCreateProofByAmountInput, E>
    + SysInvokable<AuthZoneCreateProofByIdsInput, E>
    + SysInvokable<ResourceManagerCreateBucketInput, E>
    + SysInvokable<ResourceManagerBurnInput, E>
    + SysInvokable<BucketCreateProofInput, E>
    + SysInvokable<ProofCloneInput, E>
    + SysInvokable<VaultGetAmountInput, E>
{
}
