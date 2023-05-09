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
        self.0.len() + self.1.len()
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
    pub blueprint: Blueprint,
    pub ident: FnIdent,
}
