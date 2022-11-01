use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::scrypto_env::*;
use crate::engine::{api::*, types::*};

use crate::math::Decimal;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePopInvocation {
    pub receiver: AuthZoneId,
}

impl SysInvocation for AuthZonePopInvocation {
    type Output = scrypto::resource::Proof;
}

impl ScryptoNativeInvocation for AuthZonePopInvocation {}

impl Into<NativeFnInvocation> for AuthZonePopInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Pop(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePushInvocation {
    pub receiver: AuthZoneId,
    pub proof: Proof,
}

impl SysInvocation for AuthZonePushInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AuthZonePushInvocation {}

impl Into<NativeFnInvocation> for AuthZonePushInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Push(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInvocation {
    pub receiver: AuthZoneId,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for AuthZoneCreateProofInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofInvocation {}

impl Into<NativeFnInvocation> for AuthZoneCreateProofInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProof(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub receiver: AuthZoneId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByAmountInvocation {}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProofByAmount(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub receiver: AuthZoneId,
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByIdsInvocation {}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByIdsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProofByIds(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneClearInvocation {
    pub receiver: AuthZoneId,
}

impl SysInvocation for AuthZoneClearInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AuthZoneClearInvocation {}

impl Into<NativeFnInvocation> for AuthZoneClearInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Clear(self),
        ))
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneDrainInvocation {
    pub receiver: AuthZoneId,
}

impl SysInvocation for AuthZoneDrainInvocation {
    type Output = Vec<scrypto::resource::Proof>;
}

impl ScryptoNativeInvocation for AuthZoneDrainInvocation {}

impl Into<NativeFnInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Drain(self),
        ))
    }
}

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn sys_drain<Y, E: Debug + TypeId + Decode>(env: &mut Y) -> Result<Vec<Proof>, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneDrainInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneDrainInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_clear<Y, E: Debug + TypeId + Decode>(env: &mut Y) -> Result<(), E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneClearInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneClearInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_pop<Y, E: Debug + TypeId + Decode>(env: &mut Y) -> Result<Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZonePopInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZonePopInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_create_proof<Y, E: Debug + TypeId + Decode>(
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneCreateProofInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneCreateProofInvocation {
            receiver: node_id.into(),
            resource_address,
        })
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + TypeId + Decode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneCreateProofByAmountInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneCreateProofByAmountInvocation {
            receiver: node_id.into(),
            amount,
            resource_address,
        })
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + TypeId + Decode>(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneCreateProofByIdsInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneCreateProofByIdsInvocation {
            receiver: node_id.into(),
            ids: ids.clone(),
            resource_address,
        })
    }

    pub fn sys_push<P: Into<Proof>, Y, E: Debug + TypeId + Decode>(
        proof: P,
        env: &mut Y,
    ) -> Result<(), E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZonePushInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let proof: Proof = proof.into();

        env.sys_invoke(AuthZonePushInvocation {
            receiver: node_id.into(),
            proof,
        })
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use crate::engine::scrypto_env::ScryptoEnv;
    use crate::math::Decimal;
    use crate::resource::*;
    use sbor::rust::collections::BTreeSet;

    impl ComponentAuthZone {
        pub fn pop() -> Proof {
            Self::sys_pop(&mut ScryptoEnv).unwrap()
        }

        pub fn create_proof(resource_address: ResourceAddress) -> Proof {
            Self::sys_create_proof(resource_address, &mut ScryptoEnv).unwrap()
        }

        pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
            Self::sys_create_proof_by_amount(amount, resource_address, &mut ScryptoEnv).unwrap()
        }

        pub fn create_proof_by_ids(
            ids: &BTreeSet<NonFungibleId>,
            resource_address: ResourceAddress,
        ) -> Proof {
            Self::sys_create_proof_by_ids(ids, resource_address, &mut ScryptoEnv).unwrap()
        }

        pub fn push<P: Into<Proof>>(proof: P) {
            Self::sys_push(proof, &mut ScryptoEnv).unwrap()
        }
    }
}
