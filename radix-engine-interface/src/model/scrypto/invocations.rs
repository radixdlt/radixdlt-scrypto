use crate::api::types::ComponentId;
use crate::api::wasm::SerializableInvocation;
use crate::api::Invocation;
use crate::model::{CallTableInvocation, ComponentAddress, PackageAddress};
use crate::*;
use radix_engine_interface::data::ScryptoValue;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Copy, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum ScryptoReceiver {
    Global(ComponentAddress),
    Component(ComponentId),
}

/// Scrypto function/method invocation.
#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ScryptoInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub fn_name: String,
    pub receiver: Option<ScryptoReceiver>,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoInvocation {
    type Output = ScryptoValue;
}

impl SerializableInvocation for ScryptoInvocation {
    type ScryptoOutput = ScryptoValue;
}

impl Into<CallTableInvocation> for ScryptoInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::Scrypto(self)
    }
}
