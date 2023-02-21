use crate::api::package::PackageAddress;
use crate::api::types::*;
use crate::data::ScryptoValue;
use crate::*;
use sbor::rust::string::String;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum InvocationIdentifier {
    Transaction, // TODO: Remove
    Function(FnIdentifier),
    Method(MethodReceiver, String),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FnIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub ident: String,
}

impl FnIdentifier {
    pub fn new(package_address: PackageAddress, blueprint_name: String, ident: String) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident,
        }
    }

    pub fn package_address(&self) -> PackageAddress {
        self.package_address
    }

    pub fn blueprint_name(&self) -> &String {
        &self.blueprint_name
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FunctionInvocation {
    pub fn_identifier: FnIdentifier,
    pub args: Vec<u8>,
}

impl Invocation for FunctionInvocation {
    type Output = ScryptoValue;

    fn identifier(&self) -> InvocationIdentifier {
        InvocationIdentifier::Function(self.fn_identifier.clone())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoSbor)]
pub struct MethodReceiver(pub RENodeId, pub NodeModuleId);

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodInvocation {
    pub receiver: MethodReceiver,
    pub fn_name: String,
    pub args: Vec<u8>,
}

impl Invocation for MethodInvocation {
    type Output = ScryptoValue;

    fn identifier(&self) -> InvocationIdentifier {
        InvocationIdentifier::Method(self.receiver, self.fn_name.clone())
    }
}
