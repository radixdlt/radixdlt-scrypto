use sbor::rust::fmt::Debug;

use crate::api::types::RENodeId;
use crate::api::{api::*, wasm_input::*};
use crate::model::*;
use crate::scrypto;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentSetRoyaltyConfigInvocation {
    pub receiver: RENodeId,
    pub royalty_config: RoyaltyConfig,
}

impl SysInvocation for ComponentSetRoyaltyConfigInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ComponentSetRoyaltyConfigInvocation {}

impl Into<NativeFnInvocation> for ComponentSetRoyaltyConfigInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Component(
            ComponentMethodInvocation::SetRoyaltyConfig(self),
        ))
    }
}
