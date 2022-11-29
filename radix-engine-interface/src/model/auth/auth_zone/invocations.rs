use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

use crate::api::api::*;
use crate::api::types::AuthZoneId;
use crate::math::Decimal;
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZonePopInvocation {
    pub receiver: AuthZoneId,
}

impl Invocation for AuthZonePopInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZonePopInvocation {
    type ScryptoOutput = Proof;
}

impl Into<NativeFnInvocation> for AuthZonePopInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Pop(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZonePushInvocation {
    pub receiver: AuthZoneId,
    pub proof: Proof,
}

impl Invocation for AuthZonePushInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AuthZonePushInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AuthZonePushInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Push(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInvocation {
    pub receiver: AuthZoneId,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofInvocation {
    type ScryptoOutput = Proof;
}

impl Into<NativeFnInvocation> for AuthZoneCreateProofInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProof(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub receiver: AuthZoneId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;
}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByAmountInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProofByAmount(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub receiver: AuthZoneId,
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;
}

impl ScryptoNativeInvocation for AuthZoneCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;
}

impl Into<NativeFnInvocation> for AuthZoneCreateProofByIdsInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::CreateProofByIds(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneClearInvocation {
    pub receiver: AuthZoneId,
}

impl Invocation for AuthZoneClearInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AuthZoneClearInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AuthZoneClearInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Clear(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneDrainInvocation {
    pub receiver: AuthZoneId,
}

impl Invocation for AuthZoneDrainInvocation {
    type Output = Vec<Proof>;
}

impl ScryptoNativeInvocation for AuthZoneDrainInvocation {
    type ScryptoOutput = Vec<Proof>;
}

impl Into<NativeFnInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::Drain(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneAssertAccessRuleInvocation {
    pub receiver: AuthZoneId,
    pub access_rule: AccessRule,
}

impl Invocation for AuthZoneAssertAccessRuleInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AuthZoneAssertAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AuthZoneAssertAccessRuleInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AuthZone(
            AuthZoneMethodInvocation::AssertAuthRule(self),
        ))
    }
}
