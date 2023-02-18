use crate::api::package::PackageAddress;
use crate::api::types::*;
use crate::data::ScryptoValue;
use crate::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ScryptoInvocation {
    pub package_address: PackageAddress,
    pub blueprint_name: String,
    pub fn_name: String,
    pub receiver: Option<(ScryptoReceiver, NodeModuleId)>,
    pub args: Vec<u8>,
}

impl Invocation for ScryptoInvocation {
    type Output = ScryptoValue;

    fn identifier(&self) -> InvocationIdentifier {
        InvocationIdentifier::Function(self.package_address, self.blueprint_name.to_string(), self.fn_name.clone())
    }
}