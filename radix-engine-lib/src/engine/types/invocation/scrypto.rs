use crate::component::*;
use crate::engine::types::*;
use crate::scrypto;

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ScryptoFunctionIdent {
    pub package: ScryptoPackage,
    pub blueprint_name: String,
    pub function_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ScryptoMethodIdent {
    pub receiver: ScryptoReceiver,
    pub method_name: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ScryptoPackage {
    Global(PackageAddress),
    /* The following variant is commented out because all packages are globalized upon instantiation. */
    // Package(PackageId),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[scrypto(TypeId, Encode, Decode)]
pub enum ScryptoReceiver {
    Global(ComponentAddress),
    Component(ComponentId),
}
