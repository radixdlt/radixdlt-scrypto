use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::ScryptoReceiver;
use radix_engine_interface::api::{
    ClientApi, ClientNativeInvokeApi, ClientNodeApi, ClientSubstateApi,
};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{
    scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode,
};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn sys_drain<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        env: &mut Y,
    ) -> Result<Vec<Proof>, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        env.call_native(AuthZoneDrainInvocation {})
    }

    pub fn sys_clear<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        env.call_native(AuthZoneClearInvocation {})
    }

    pub fn sys_pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(api: &mut Y) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::AuthZoneStack,
            AUTH_ZONE_POP_IDENT,
            scrypto_encode(&AuthZonePopInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::AuthZoneStack,
            AUTH_ZONE_CREATE_PROOF_IDENT,
            scrypto_encode(&AuthZoneCreateProofInput { resource_address, }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::AuthZoneStack,
            AUTH_ZONE_CREATE_PROOF_BY_AMOUNT_IDENT,
            scrypto_encode(&AuthZoneCreateProofByAmountInput { resource_address, amount, }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        api: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            ScryptoReceiver::AuthZoneStack,
            AUTH_ZONE_CREATE_PROOF_BY_IDS_IDENT,
            scrypto_encode(&AuthZoneCreateProofByIdsInput { resource_address, ids: ids.clone(), }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_push<P: Into<Proof>, Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        proof: P,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let proof: Proof = proof.into();

        let _rtn = api.call_method(
            ScryptoReceiver::AuthZoneStack,
            AUTH_ZONE_PUSH_IDENT,
            scrypto_encode(&AuthZonePushInput { proof }).unwrap(),
        )?;

        Ok(())
    }

    pub fn sys_assert_access_rule<Y, E>(access_rule: AccessRule, env: &mut Y) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientNativeInvokeApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        env.call_native(AuthZoneAssertAccessRuleInvocation { access_rule })
    }
}
