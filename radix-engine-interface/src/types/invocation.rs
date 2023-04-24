use crate::api::ObjectModuleId;
use crate::blueprints::resource::MethodKey;
use crate::types::*;
use crate::*;
use radix_engine_common::types::*;
use sbor::rust::prelude::*;
use sbor::rust::string::String;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct MethodIdentifier(pub NodeId, pub ObjectModuleId, pub String);

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
