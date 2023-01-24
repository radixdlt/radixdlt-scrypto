use radix_engine_interface::api::blueprints::resource::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{EngineSubstateApi, Invokable};
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use sbor::rust::fmt::Debug;

pub trait SysProof {
    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineSubstateApi<E> + Invokable<ProofCloneInvocation, E>;
    fn sys_drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        sys_calls: &mut Y,
    ) -> Result<(), E>
    where
        Y: EngineSubstateApi<E>;
}

impl SysProof for Proof {
    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineSubstateApi<E> + Invokable<ProofCloneInvocation, E>,
    {
        sys_calls.invoke(ProofCloneInvocation { receiver: self.0 })
    }

    fn sys_drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        sys_calls: &mut Y,
    ) -> Result<(), E>
    where
        Y: EngineSubstateApi<E>,
    {
        sys_calls.sys_drop_node(RENodeId::Proof(self.0))
    }
}
