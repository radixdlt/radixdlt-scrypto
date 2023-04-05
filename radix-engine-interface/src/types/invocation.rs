use crate::blueprints::resource::MethodKey;
use crate::types::*;
use crate::*;
use radix_engine_common::types::*;
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
pub struct MethodReceiver(pub NodeId, pub TypedModuleId);

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodIdentifier(pub NodeId, pub TypedModuleId, pub String);

impl MethodIdentifier {
    pub fn method_key(&self) -> MethodKey {
        MethodKey::new(self.1, self.2.as_str())
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FunctionIdentifier(pub Blueprint, pub String);

impl FunctionIdentifier {
    pub fn new(blueprint: Blueprint, ident: String) -> Self {
        Self(blueprint, ident)
    }

    pub fn size(&self) -> usize {
        self.0.size() + self.1.len()
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

    pub fn to_debug_string(&self) -> String {
        match self {
            FnIdent::Application(x) => x.clone(),
            FnIdent::System(x) => x.to_string(),
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
    pub fn application_ident(
        package_address: PackageAddress,
        blueprint_name: String,
        ident: String,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident: FnIdent::Application(ident),
        }
    }

    pub fn system_ident(
        package_address: PackageAddress,
        blueprint_name: String,
        ident: u8,
    ) -> Self {
        Self {
            package_address,
            blueprint_name,
            ident: FnIdent::System(ident),
        }
    }

    pub fn package_address(&self) -> PackageAddress {
        self.package_address
    }

    pub fn blueprint_name(&self) -> &String {
        &self.blueprint_name
    }

    pub fn size(&self) -> usize {
        self.blueprint_name.len() + self.ident.len() + self.package_address.as_ref().len()
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
    pub blueprint: Blueprint,
    pub virtual_func_id: u8,
    pub args: [u8; 26],
}

impl Invocation for VirtualLazyLoadInvocation {
    type Output = IndexedScryptoValue;

    fn debug_identifier(&self) -> InvocationDebugIdentifier {
        InvocationDebugIdentifier::VirtualLazyLoad
    }
}
