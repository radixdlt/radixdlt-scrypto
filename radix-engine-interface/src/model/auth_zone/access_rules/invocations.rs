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
    pub key: AccessRuleKey,
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
            AccessRulesMethodInvocation::SetAccessRule(self)
        ))
    }
}
