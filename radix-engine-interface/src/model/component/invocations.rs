use sbor::rust::fmt::Debug;

use crate::api::api::*;
use crate::api::types::{ComponentId, RENodeId};
use crate::model::*;
use crate::scrypto;
use crate::wasm::*;

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentGlobalizeWithOwnerInvocation {
    pub component_id: ComponentId,
}

impl Invocation for ComponentGlobalizeWithOwnerInvocation {
    type Output = (ComponentAddress, Bucket);
}

impl ScryptoNativeInvocation for ComponentGlobalizeWithOwnerInvocation {
    type ScryptoOutput = (ComponentAddress, Bucket);
}

impl Into<NativeFnInvocation> for ComponentGlobalizeWithOwnerInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Component(
            ComponentFunctionInvocation::GlobalizeWithOwner(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentGlobalizeNoOwnerInvocation {
    pub component_id: ComponentId,
}

impl Invocation for ComponentGlobalizeNoOwnerInvocation {
    type Output = ComponentAddress;
}

impl ScryptoNativeInvocation for ComponentGlobalizeNoOwnerInvocation {
    type ScryptoOutput = ComponentAddress;
}

impl Into<NativeFnInvocation> for ComponentGlobalizeNoOwnerInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Function(NativeFunctionInvocation::Component(
            ComponentFunctionInvocation::GlobalizeNoOwner(self),
        ))
    }
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ComponentSetRoyaltyConfigInvocation {
    /// Either global or local component
    pub receiver: RENodeId,
    pub royalty_config: RoyaltyConfig,
}

impl Invocation for ComponentSetRoyaltyConfigInvocation {
    type Output = ();
}

impl ScryptoNativeInvocation for ComponentSetRoyaltyConfigInvocation {
    type ScryptoOutput = ();
}

impl Into<NativeFnInvocation> for ComponentSetRoyaltyConfigInvocation {
    fn into(self) -> NativeFnInvocation {
        NativeFnInvocation::Method(NativeMethodInvocation::Component(
            ComponentMethodInvocation::SetRoyaltyConfig(self),
        ))
    }
}
