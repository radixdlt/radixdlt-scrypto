use radix_engine_interface::data::ScryptoValue;
use crate::api::api::Invocation;
use crate::api::types::Receiver;
use crate::model::{PackageAddress, CallTableInvocation};
use crate::scrypto;
use crate::wasm::SerializableInvocation;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

/// Scrypto function/method invocation.
#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ScryptoFunctionInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub function_name: String,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoFunctionInvocation {
    type Output = ScryptoValue;
}

impl SerializableInvocation for ScryptoFunctionInvocation {
    type ScryptoOutput = ScryptoValue;
}

impl Into<CallTableInvocation> for ScryptoFunctionInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::ScryptoFunction(self)
    }
}

/// Scrypto function/method invocation.
#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ScryptoMethodInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub method_name: String,
    pub receiver: Option<Receiver>,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoMethodInvocation {
    type Output = ScryptoValue;
}

impl SerializableInvocation for ScryptoMethodInvocation {
    type ScryptoOutput = ScryptoValue;
}

impl Into<CallTableInvocation> for ScryptoMethodInvocation {
    fn into(self) -> CallTableInvocation {
        CallTableInvocation::ScryptoMethod(self)
    }
}
