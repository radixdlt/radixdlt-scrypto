use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{ClientNodeApi, ClientSubstateApi, Invokable};
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
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + Invokable<AuthZoneDrainInvocation, E>,
    {
        env.invoke(AuthZoneDrainInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
        })
    }

    pub fn sys_clear<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + Invokable<AuthZoneClearInvocation, E>,
    {
        env.invoke(AuthZoneClearInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
        })
    }

    pub fn sys_pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + Invokable<AuthZonePopInvocation, E>,
    {
        env.invoke(AuthZonePopInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
        })
    }

    pub fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + Invokable<AuthZoneCreateProofInvocation, E>,
    {
        env.invoke(AuthZoneCreateProofInvocation {
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
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + Invokable<AuthZoneCreateProofByAmountInvocation, E>,
    {
        env.invoke(AuthZoneCreateProofByAmountInvocation {
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
        Y: ClientNodeApi<E>
            + ClientSubstateApi<E>
            + Invokable<AuthZoneCreateProofByIdsInvocation, E>,
    {
        env.invoke(AuthZoneCreateProofByIdsInvocation {
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
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + Invokable<AuthZonePushInvocation, E>,
    {
        let proof: Proof = proof.into();

        env.invoke(AuthZonePushInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            proof,
        })
    }

    pub fn sys_assert_access_rule<Y, E>(access_rule: AccessRule, env: &mut Y) -> Result<(), E>
    where
        Y: ClientNodeApi<E> + Invokable<AuthZoneAssertAccessRuleInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        env.invoke(AuthZoneAssertAccessRuleInvocation {
            receiver: RENodeId::AuthZoneStack.into(),
            access_rule,
        })
    }
}
