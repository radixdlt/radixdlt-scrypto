use radix_engine_interface::wasm::SerializableInvocation;
use sbor::rust::fmt::Debug;
use sbor::*;

use crate::api::{api::*, types::*};
use crate::scrypto;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesAddAccessCheckInvocation {
    pub receiver: RENodeId,
    pub access_rules: AccessRules,
}

impl Invocation for AccessRulesAddAccessCheckInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessRulesAddAccessCheckInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AccessRulesAddAccessCheckInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::AddAccessCheck(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesSetMethodAccessRuleInvocation {
    pub receiver: RENodeId,
    pub index: u32,
    pub key: AccessRuleKey,
    pub rule: AccessRule,
}

impl Invocation for AccessRulesSetMethodAccessRuleInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessRulesSetMethodAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AccessRulesSetMethodAccessRuleInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetMethodAccessRule(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesSetGroupAccessRuleInvocation {
    pub receiver: RENodeId,
    pub index: u32,
    pub name: String,
    pub rule: AccessRule,
}

impl Invocation for AccessRulesSetGroupAccessRuleInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessRulesSetGroupAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AccessRulesSetGroupAccessRuleInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetGroupAccessRule(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesSetMethodMutabilityInvocation {
    pub receiver: RENodeId,
    pub index: u32,
    pub key: AccessRuleKey,
    pub mutability: AccessRule,
}

impl Invocation for AccessRulesSetMethodMutabilityInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessRulesSetMethodMutabilityInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AccessRulesSetMethodMutabilityInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetMethodMutability(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesSetGroupMutabilityInvocation {
    pub receiver: RENodeId,
    pub index: u32,
    pub name: String,
    pub mutability: AccessRule,
}

impl Invocation for AccessRulesSetGroupMutabilityInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessRulesSetGroupMutabilityInvocation {
    type ScryptoOutput = ();
}

impl Into<SerializedInvocation> for AccessRulesSetGroupMutabilityInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetGroupMutability(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesGetLengthInvocation {
    pub receiver: RENodeId,
}

impl Invocation for AccessRulesGetLengthInvocation {
    type Output = u32;
}

impl SerializableInvocation for AccessRulesGetLengthInvocation {
    type ScryptoOutput = u32;
}

impl Into<SerializedInvocation> for AccessRulesGetLengthInvocation {
    fn into(self) -> SerializedInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::GetLength(self)).into()
    }
}
