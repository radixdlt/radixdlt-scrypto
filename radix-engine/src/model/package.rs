use sbor::*;
use scrypto::rust::vec::Vec;
use scrypto::rust::string::String;
use wasmi::{MemoryRef, ModuleRef};
use crate::engine::{instantiate_module, parse_module};

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, TypeId, Encode, Decode)]
pub struct Package {
    code: Vec<u8>,
    blueprints: Vec<String>,
}

pub enum PackageError {
    BlueprintNotFound(String)
}

impl Package {
    pub fn new(blueprints: Vec<String>, code: Vec<u8>) -> Self {
        Self { blueprints, code }
    }

    pub fn code(&self) -> &[u8] {
        &self.code
    }

    pub fn load_blueprint(&self, blueprint_name: String) -> Result<(ModuleRef, MemoryRef), PackageError> {
        if !self.blueprints.iter().any(|s| s.eq(&blueprint_name)) {
            return Err(PackageError::BlueprintNotFound(blueprint_name));
        }

        let module = parse_module(&self.code).unwrap();
        let inst = instantiate_module(&module).unwrap();
        Ok(inst)
    }
}
