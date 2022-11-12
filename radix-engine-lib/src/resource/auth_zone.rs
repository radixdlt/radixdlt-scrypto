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
    type Output = Proof;
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
    type Output = Vec<Proof>;
}

impl ScryptoNativeInvocation for AuthZoneDrainInvocation {}

impl Into<NativeFnInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Drain(self),
        ))
    }
}
