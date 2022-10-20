use crate::engine::types::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash, TypeId, Encode, Decode)]
pub struct ScryptoFunctionIdent {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub function_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub struct ScryptoMethodIdent {
    pub receiver: ScryptoReceiver,
    pub method_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq, TypeId, Encode, Decode)]
pub enum ScryptoReceiver {
    Global(ComponentAddress),
    Component(ComponentId),
}
