use sbor::rust::fmt::Debug;

use crate::api::{api::*, types::*, wasm_input::*};
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentAddAccessCheckInvocation {
    pub receiver: ComponentId,
    pub access_rules: AccessRules,
}

impl SysInvocation for ComponentAddAccessCheckInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ComponentAddAccessCheckInvocation {}

impl Into<NativeFnInvocation> for ComponentAddAccessCheckInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Component(
            ComponentMethodInvocation::AddAccessCheck(self),
        ))
    }
}
