use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{ClientNodeApi, ClientObjectApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;

pub trait SysProof {
    fn sys_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientObjectApi<E>;

    fn sys_non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientObjectApi<E>;

    fn sys_resource_address<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>
    where
        Y: ClientObjectApi<E>;

    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientObjectApi<E>;

    fn sys_drop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        self,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E>;
}

impl SysProof for Proof {
    fn sys_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Decimal, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Proof(self.0),
            PROOF_GET_AMOUNT_IDENT,
            scrypto_encode(&ProofGetAmountInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_non_fungible_local_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<BTreeSet<NonFungibleLocalId>, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Proof(self.0),
            PROOF_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT,
            scrypto_encode(&ProofGetNonFungibleLocalIdsInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_resource_address<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<ResourceAddress, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Proof(self.0),
            PROOF_GET_RESOURCE_ADDRESS_IDENT,
            scrypto_encode(&ProofGetResourceAddressInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    fn sys_clone<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Proof(self.0),
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
