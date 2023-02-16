use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::math::Decimal;
use crate::*;
use sbor::rust::collections::{BTreeMap, BTreeSet};
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use scrypto_abi::BlueprintAbi;

pub struct AuthZoneAbi;

impl AuthZoneAbi {
    pub fn blueprint_abis() -> BTreeMap<String, BlueprintAbi> {
        BTreeMap::new()
    }
}

pub const AUTH_ZONE_BLUEPRINT: &str = "AuthZone";

pub const AUTH_ZONE_POP_IDENT: &str = "pop";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZonePopInput {}

pub const AUTH_ZONE_PUSH_IDENT: &str = "push";

#[derive(Debug, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZonePushInput {
    pub proof: Proof,
}

impl Clone for AuthZonePushInput {
    fn clone(&self) -> Self {
        Self {
            proof: Proof(self.proof.0),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneCreateProofInvocation {
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AuthZoneStack(AuthZoneStackFn::CreateProof))
    }
}

impl SerializableInvocation for AuthZoneCreateProofInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::AuthZoneStack(AuthZoneStackFn::CreateProof)
    }
}

impl Into<CallTableInvocation> for AuthZoneCreateProofInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::CreateProof(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByAmountInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AuthZoneStack(
            AuthZoneStackFn::CreateProofByAmount,
        ))
    }
}

impl SerializableInvocation for AuthZoneCreateProofByAmountInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::AuthZoneStack(AuthZoneStackFn::CreateProofByAmount)
    }
}

impl Into<CallTableInvocation> for AuthZoneCreateProofByAmountInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::CreateProofByAmount(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub ids: BTreeSet<NonFungibleLocalId>,
    pub resource_address: ResourceAddress,
}

impl Invocation for AuthZoneCreateProofByIdsInvocation {
    type Output = Proof;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AuthZoneStack(AuthZoneStackFn::CreateProofByIds))
    }
}

impl SerializableInvocation for AuthZoneCreateProofByIdsInvocation {
    type ScryptoOutput = Proof;

    fn native_fn() -> NativeFn {
        NativeFn::AuthZoneStack(AuthZoneStackFn::CreateProofByIds)
    }
}

impl Into<CallTableInvocation> for AuthZoneCreateProofByIdsInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::CreateProofByIds(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneClearInvocation {}

impl Invocation for AuthZoneClearInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AuthZoneStack(AuthZoneStackFn::Clear))
    }
}

impl SerializableInvocation for AuthZoneClearInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AuthZoneStack(AuthZoneStackFn::Clear)
    }
}

impl Into<CallTableInvocation> for AuthZoneClearInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::Clear(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneDrainInvocation {}

impl Invocation for AuthZoneDrainInvocation {
    type Output = Vec<Proof>;

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AuthZoneStack(AuthZoneStackFn::Drain))
    }
}

impl SerializableInvocation for AuthZoneDrainInvocation {
    type ScryptoOutput = Vec<Proof>;

    fn native_fn() -> NativeFn {
        NativeFn::AuthZoneStack(AuthZoneStackFn::Drain)
    }
}

impl Into<CallTableInvocation> for AuthZoneDrainInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::Drain(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AuthZoneAssertAccessRuleInvocation {
    pub access_rule: AccessRule,
}

impl Invocation for AuthZoneAssertAccessRuleInvocation {
    type Output = ();

    fn fn_identifier(&self) -> FnIdentifier {
        FnIdentifier::Native(NativeFn::AuthZoneStack(AuthZoneStackFn::AssertAccessRule))
    }
}

impl SerializableInvocation for AuthZoneAssertAccessRuleInvocation {
    type ScryptoOutput = ();

    fn native_fn() -> NativeFn {
        NativeFn::AuthZoneStack(AuthZoneStackFn::AssertAccessRule)
    }
}

impl Into<CallTableInvocation> for AuthZoneAssertAccessRuleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AuthZoneStack(AuthZoneStackInvocation::AssertAuthRule(self)).into()
    }
}
