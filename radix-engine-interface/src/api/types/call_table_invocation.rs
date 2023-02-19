use crate::api::types::*;
use crate::data::ScryptoValue;
use crate::*;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
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

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct MethodInvocation {
    pub receiver: (RENodeId, NodeModuleId),
    pub fn_name: String,
    pub args: Vec<u8>,
}

impl Invocation for MethodInvocation {
    type Output = ScryptoValue;

    fn identifier(&self) -> InvocationIdentifier {
        InvocationIdentifier::Method(self.receiver.0, self.receiver.1, self.fn_name.clone())
    }
}
