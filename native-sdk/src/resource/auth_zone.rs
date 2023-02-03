use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{ClientNativeInvokeApi, ClientNodeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
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
        env.call_native(AuthZoneDrainInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
        })
    }

    pub fn sys_clear<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        env.call_native(AuthZoneClearInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
        })
    }

    pub fn sys_pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        env.call_native(AuthZonePopInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
        })
    }

    pub fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        env.call_native(AuthZoneCreateProofInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            resource_address,
        })
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        env.call_native(AuthZoneCreateProofByAmountInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            amount,
            resource_address,
        })
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        env.call_native(AuthZoneCreateProofByIdsInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            ids: ids.clone(),
            resource_address,
        })
    }

    pub fn sys_push<P: Into<Proof>, Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        proof: P,
        env: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
    {
        let proof: Proof = proof.into();

        env.call_native(AuthZonePushInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            proof,
        })
    }

    pub fn sys_assert_access_rule<Y, E>(access_rule: AccessRule, env: &mut Y) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientNativeInvokeApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        env.call_native(AuthZoneAssertAccessRuleInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            access_rule,
        })
    }
}
