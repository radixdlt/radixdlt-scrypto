use crate::api::types::*;
use crate::blueprints::resource::MethodKey;
use crate::data::scrypto::model::*;
use crate::*;
use sbor::rust::prelude::*;
use sbor::rust::string::String;

// TODO: Remove
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum InvocationDebugIdentifier {
    Function(FunctionIdentifier),
    Method(MethodIdentifier),
    VirtualLazyLoad,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ScryptoSbor)]
pub struct MethodReceiver(pub RENodeId, pub NodeModuleId);

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodIdentifier(pub RENodeId, pub NodeModuleId, pub String);

impl MethodIdentifier {
    pub fn method_key(&self) -> MethodKey {
        MethodKey::new(self.1, self.2.as_str())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FunctionIdentifier(pub PackageAddress, pub String, pub String);

impl FunctionIdentifier {
    pub fn new(package_address: PackageAddress, blueprint_name: String, ident: String) -> Self {
        Self(package_address, blueprint_name, ident)
    }

    pub fn package_address(&self) -> PackageAddress {
        self.0
    }

    pub fn blueprint_name(&self) -> &String {
        &self.1
    }

    pub fn size(&self) -> usize {
        self.1.len() + self.2.len() + self.0.size()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum FnIdent {
    Application(String),
    System(u8),
}

impl FnIdent {
    pub fn len(&self) -> usize {
        match self {
            FnIdent::System(..) => 1,
            FnIdent::Application(ident) => ident.len(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FnIdentifier {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub ident: FnIdent,
}

impl FnIdentifier {
    pub fn new(package_address: PackageAddress, blueprint_name: String, ident: String) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident: FnIdent::Application(ident),
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
    pub identifier: FunctionIdentifier,
    pub args: Vec<u8>,
}

impl Invocation for FunctionInvocation {
    type Output = IndexedScryptoValue;

    fn debug_identifier(&self) -> InvocationDebugIdentifier {
        InvocationDebugIdentifier::Function(self.identifier.clone())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodInvocation {
    pub identifier: MethodIdentifier,
    pub args: Vec<u8>,
}

impl Invocation for MethodInvocation {
    type Output = IndexedScryptoValue;

    fn debug_identifier(&self) -> InvocationDebugIdentifier {
        InvocationDebugIdentifier::Method(self.identifier.clone())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct VirtualLazyLoadInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub virtual_func_id: u8,
    pub args: [u8; 26],
}

impl Invocation for VirtualLazyLoadInvocation {
    type Output = IndexedScryptoValue;

    fn debug_identifier(&self) -> InvocationDebugIdentifier {
        InvocationDebugIdentifier::VirtualLazyLoad
    }
}
