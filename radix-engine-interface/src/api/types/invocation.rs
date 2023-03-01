use crate::api::types::*;
use crate::blueprints::resource::MethodKey;
use crate::data::scrypto::model::*;
use crate::data::scrypto::ScryptoValue;
use crate::*;
use sbor::rust::string::String;

// TODO: Remove
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum InvocationDebugIdentifier {
    Transaction,
    Function(FnIdentifier),
    Method(MethodIdentifier),
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FnIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub ident: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoSbor)]
pub struct MethodReceiver(pub RENodeId, pub NodeModuleId);

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodIdentifier(pub RENodeId, pub NodeModuleId, pub String);

impl MethodIdentifier {
    pub fn method_key(&self) -> MethodKey {
        MethodKey::new(self.1, self.2.clone())
    }
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

    pub fn size(&self) -> usize {
        self.blueprint_name.len() + self.ident.len() + self.package_address.size()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FunctionInvocation {
    pub fn_identifier: FnIdentifier,
    pub args: Vec<u8>,
}

impl Invocation for FunctionInvocation {
    type Output = ScryptoValue;

    fn debug_identifier(&self) -> InvocationDebugIdentifier {
        InvocationDebugIdentifier::Function(self.fn_identifier.clone())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodInvocation {
    pub identifier: MethodIdentifier,
    pub args: Vec<u8>,
}

impl Invocation for MethodInvocation {
    type Output = ScryptoValue;

    fn debug_identifier(&self) -> InvocationDebugIdentifier {
        InvocationDebugIdentifier::Method(self.identifier.clone())
    }
}
