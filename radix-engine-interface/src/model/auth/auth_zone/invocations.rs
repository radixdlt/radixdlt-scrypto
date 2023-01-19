use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

use crate::api::types::AuthZoneStackId;
use crate::api::wasm::*;
use crate::api::*;
use crate::math::Decimal;
use crate::model::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZonePopInvocation {
    pub receiver: AuthZoneStackId,
}

impl Invocation for AuthZonePopInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZonePopInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AuthZonePopInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::Pop(self)).into()
    }
}

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZonePushInvocation {
    pub receiver: AuthZoneStackId,
    pub proof: Proof,
}

impl Clone for AuthZonePushInvocation {
    fn clone(&self) -> Self {
        Self {
            receiver: self.receiver,
            proof: Proof(self.proof.0),
        }
    }
}

impl Invocation for AuthZonePushInvocation {
    type Output = ();
}

impl SerializableInvocation for AuthZonePushInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AuthZonePushInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::Push(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneCreateProofInvocation {
    pub receiver: AuthZoneStackId,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZoneCreateProofInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AuthZoneCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::CreateProof(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub receiver: AuthZoneStackId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZoneCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AuthZoneCreateProofByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::CreateProofByAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub receiver: AuthZoneStackId,
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;
}

impl SerializableInvocation for AuthZoneCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;
}

impl Into<CallTableInvocation> for AuthZoneCreateProofByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::CreateProofByIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneClearInvocation {
    pub receiver: AuthZoneStackId,
}

impl Invocation for AuthZoneClearInvocation {
    type Output = ();
}

impl SerializableInvocation for AuthZoneClearInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AuthZoneClearInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::Clear(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneDrainInvocation {
    pub receiver: AuthZoneStackId,
}

impl Invocation for AuthZoneDrainInvocation {
    type Output = Vec<Proof>;
}

impl SerializableInvocation for AuthZoneDrainInvocation {
    type ScryptoOutput = Vec<Proof>;
}

impl Into<CallTableInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::Drain(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneAssertAccessRuleInvocation {
    pub receiver: AuthZoneStackId,
    pub access_rule: AccessRule,
}

impl Invocation for AuthZoneAssertAccessRuleInvocation {
    type Output = ();
}

impl SerializableInvocation for AuthZoneAssertAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AuthZoneAssertAccessRuleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::AssertAuthRule(self)).into()
    }
}
