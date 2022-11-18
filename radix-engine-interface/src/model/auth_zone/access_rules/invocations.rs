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
        NativeFnInvocation::Method(NativeMethodInvocation::Component(
            ComponentMethodInvocation::AddAccessCheck(self),
        ))
    }
}
