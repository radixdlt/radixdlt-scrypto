use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{ClientNativeInvokeApi, ClientNodeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use sbor::rust::fmt::Debug;

pub trait SysProof {
    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>;
    fn sys_drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        sys_calls: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E>;
}

impl SysProof for Proof {
    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        sys_calls.call_native(ProofCloneInvocation { receiver: self.0 })
    }

    fn sys_drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        sys_calls: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E>,
    {
        sys_calls.sys_drop_node(RENodeId::Proof(self.0))
    }
}
