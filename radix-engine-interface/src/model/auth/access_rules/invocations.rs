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
pub struct AccessRulesSetAccessRuleInvocation {
    pub receiver: RENodeId,
    pub index: u32,
    pub selector: AccessRuleSelector,
    pub rule: AccessRule,
}

impl Invocation for AccessRulesSetAccessRuleInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AccessRulesSetAccessRuleInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AccessRulesSetAccessRuleInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::SetAccessRule(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesSetMutabilityInvocation {
    pub receiver: RENodeId,
    pub index: u32,
    pub selector: AccessRuleSelector,
    pub mutability: AccessRule,
}

impl Invocation for AccessRulesSetMutabilityInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AccessRulesSetMutabilityInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for AccessRulesSetMutabilityInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::SetMutability(self),
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
