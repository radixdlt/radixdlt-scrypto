use sbor::rust::fmt::Debug;

use crate::api::{api::*, types::*, wasm_input::*};
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesAddAccessCheckInvocation {
    pub receiver: RENodeId,
    pub access_rules: AccessRules,
}

impl SysInvocation for AccessRulesAddAccessCheckInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AccessRulesAddAccessCheckInvocation {}

impl Into<NativeFnInvocation> for AccessRulesAddAccessCheckInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::AddAccessCheck(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesUpdateAuthInvocation {
    pub receiver: RENodeId,
    pub access_rules_index: usize,
    pub method: AccessRulesMethodIdent,
    pub access_rule: AccessRule,
}

impl SysInvocation for AccessRulesUpdateAuthInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AccessRulesUpdateAuthInvocation {}

impl Into<NativeFnInvocation> for AccessRulesUpdateAuthInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::UpdateAuth(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AccessRulesLockAuthInvocation {
    pub receiver: RENodeId,
    pub access_rules_index: usize,
    pub method: AccessRulesMethodIdent,
}

impl SysInvocation for AccessRulesLockAuthInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for AccessRulesLockAuthInvocation {}

impl Into<NativeFnInvocation> for AccessRulesLockAuthInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::AccessRules(
            AccessRulesMethodInvocation::LockAuth(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub enum AccessRulesMethodIdent {
    Method(String),
    Default,
}

impl From<String> for AccessRulesMethodIdent {
    fn from(value: String) -> AccessRulesMethodIdent {
        AccessRulesMethodIdent::Method(value)
    }
}
