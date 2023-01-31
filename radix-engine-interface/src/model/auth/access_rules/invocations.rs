use sbor::rust::fmt::Debug;

use crate::api::types::*;
use crate::api::wasm::*;
use crate::api::*;
use crate::*;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for AccessRulesAddAccessCheckInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::AddAccessCheck(self)).into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessRulesSetMethodAccessRuleInvocation {
    pub receiver: RENodeId,
    pub index: u32,
    pub key: AccessRuleKey,
    pub rule: AccessRuleEntry,
}

impl Invocation for AccessRulesSetMethodAccessRuleInvocation {
    type Output = ();
}

impl SerializableInvocation for AccessRulesSetMethodAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<CallTableInvocation> for AccessRulesSetMethodAccessRuleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetMethodAccessRule(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for AccessRulesSetGroupAccessRuleInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetGroupAccessRule(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for AccessRulesSetMethodMutabilityInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetMethodMutability(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

impl Into<CallTableInvocation> for AccessRulesSetGroupMutabilityInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::SetGroupMutability(self))
            .into()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessRulesGetLengthInvocation {
    pub receiver: RENodeId,
}

impl Invocation for AccessRulesGetLengthInvocation {
    type Output = u32;
}

impl SerializableInvocation for AccessRulesGetLengthInvocation {
    type ScryptoOutput = u32;
}

impl Into<CallTableInvocation> for AccessRulesGetLengthInvocation {
    fn into(self) -> CallTableInvocation {
        NativeInvocation::AccessRulesChain(AccessRulesChainInvocation::GetLength(self)).into()
    }
}
