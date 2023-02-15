use radix_engine_interface::api::types::{RENodeId, ScryptoReceiver};
use radix_engine_interface::api::{ClientComponentApi, ClientNodeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use sbor::rust::fmt::Debug;

pub trait SysProof {
    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientComponentApi<E>;
    fn sys_drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E>;
}

impl SysProof for Proof {
    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientComponentApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Proof(self.0),
            PROOF_CLONE_IDENT,
            scrypto_encode(&ProofCloneInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(self, api: &mut Y) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E>,
    {
        api.sys_drop_node(RENodeId::Proof(self.0))
    }
}
