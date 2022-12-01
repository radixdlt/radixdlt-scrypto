use radix_engine_interface::wasm::ScryptoNativeInvocation;
use sbor::rust::fmt::Debug;

use crate::api::{api::*, types::*};
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesAddAccessCheckInvocation {
    pub receiver: RENodeId,
    pub access_rules: AccessRules,
}

impl Invocation for AccessRulesAddAccessCheckInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AccessRulesAddAccessCheckInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AccessRulesAddAccessCheckInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::AddAccessCheck(self),
        ))
    }
}

#[derive(Debug)]
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

impl ScryptoNativeInvocation for AccessRulesSetMethodAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AccessRulesSetMethodAccessRuleInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::SetMethodAccessRule(self),
        ))
    }
}

#[derive(Debug)]
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

impl ScryptoNativeInvocation for AccessRulesSetGroupAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AccessRulesSetGroupAccessRuleInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::SetGroupAccessRule(self),
        ))
    }
}

#[derive(Debug)]
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

impl ScryptoNativeInvocation for AccessRulesSetMethodMutabilityInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AccessRulesSetMethodMutabilityInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::SetMethodMutability(self),
        ))
    }
}

#[derive(Debug)]
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

impl ScryptoNativeInvocation for AccessRulesSetGroupMutabilityInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AccessRulesSetGroupMutabilityInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::SetGroupMutability(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesGetLengthInvocation {
    pub receiver: RENodeId,
}

impl Invocation for AccessRulesGetLengthInvocation {
    type Output = u32;
}

impl ScryptoNativeInvocation for AccessRulesGetLengthInvocation {
    type ScryptoOutput = u32;
}

impl Into<NativeFnInvocation> for AccessRulesGetLengthInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::GetLength(self),
        ))
    }
}
