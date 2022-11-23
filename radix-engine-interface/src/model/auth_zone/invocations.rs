use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

use crate::api::api::*;
use crate::api::types::AuthZoneStackId;
use crate::api::wasm_input::*;
use crate::math::Decimal;
use crate::model::*;
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZonePopInvocation {
    pub receiver: AuthZoneStackId,
}

impl SysInvocation for AuthZonePopInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZonePopInvocation {}

impl Into<NativeFnInvocation> for AuthZonePopInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Pop(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZonePushInvocation {
    pub receiver: AuthZoneStackId,
    pub proof: Proof,
}

impl SysInvocation for AuthZonePushInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AuthZonePushInvocation {}

impl Into<NativeFnInvocation> for AuthZonePushInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Push(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInvocation {
    pub receiver: AuthZoneStackId,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for AuthZoneCreateProofInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofInvocation {}

impl Into<NativeFnInvocation> for AuthZoneCreateProofInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::CreateProof(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub receiver: AuthZoneStackId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByAmountInvocation {}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::CreateProofByAmount(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub receiver: AuthZoneStackId,
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl SysInvocation for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByIdsInvocation {}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByIdsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::CreateProofByIds(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneClearInvocation {
    pub receiver: AuthZoneStackId,
}

impl SysInvocation for AuthZoneClearInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AuthZoneClearInvocation {}

impl Into<NativeFnInvocation> for AuthZoneClearInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Clear(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneDrainInvocation {
    pub receiver: AuthZoneStackId,
}

impl SysInvocation for AuthZoneDrainInvocation {
    type Output = Vec<Proof>;
}

impl ScryptoNativeInvocation for AuthZoneDrainInvocation {}

impl Into<NativeFnInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZoneStack(
            AuthZoneStackMethodInvocation::Drain(self),
        ))
    }
}
