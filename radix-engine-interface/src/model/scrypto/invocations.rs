use radix_engine_interface::model::ComponentAddress;
use crate::api::api::Invocation;
use crate::api::types::{ScryptoFunctionIdent, ScryptoMethodIdent, ScryptoReceiver};
use crate::data::IndexedScryptoValue;
use crate::model::{PackageAddress, SerializedInvocation};
use crate::scrypto;
use crate::wasm::SerializableInvocation;
use sbor::rust::vec::Vec;
use sbor::rust::string::String;
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
    type Output = Vec<u8>;
}

impl SerializableInvocation for ScryptoFunctionInvocation {
    type ScryptoOutput = Vec<u8>;
}

impl Into<SerializedInvocation> for ScryptoFunctionInvocation {
    fn into(self) -> SerializedInvocation {
        SerializedInvocation::Function(self)
    }
}

/// Scrypto function/method invocation.
#[derive(Debug)]
#[scrypto(Categorize, Encode, Decode)]
pub struct ScryptoMethodInvocation {
    pub receiver: ScryptoReceiver,
    pub method_name: String,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoMethodInvocation {
    type Output = Vec<u8>;
}

impl SerializableInvocation for ScryptoMethodInvocation {
    type ScryptoOutput = Vec<u8>;
}

impl Into<SerializedInvocation> for ScryptoMethodInvocation {
    fn into(self) -> SerializedInvocation {
        SerializedInvocation::Method(self)
    }
}

#[derive(Debug)]
pub struct ParsedScryptoFunctionInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub function_name: String,
    pub args: IndexedScryptoValue,
}

impl Invocation for ParsedScryptoFunctionInvocation {
    type Output = IndexedScryptoValue;
}

#[derive(Debug)]
pub struct ParsedScryptoMethodInvocation {
    pub receiver: ScryptoReceiver,
    pub method_name: String,
    pub args: IndexedScryptoValue,
}

impl Invocation for ParsedScryptoMethodInvocation {
    type Output = IndexedScryptoValue;
}
