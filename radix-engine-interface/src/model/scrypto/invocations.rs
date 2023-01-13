use radix_engine_interface::data::ScryptoValue;
use crate::api::api::Invocation;
use crate::model::{PackageAddress, CallTableInvocation, ComponentAddress};
use crate::scrypto;
use crate::wasm::SerializableInvocation;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use crate::api::types::ComponentId;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[scrypto(Categorize, Encode, Decode)]
pub enum ScryptoReceiver {
    Global(ComponentAddress),
    Component(ComponentId),
}

/// Scrypto function/method invocation.
#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
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
