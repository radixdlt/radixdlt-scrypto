use sbor::rust::string::String;
use sbor::*;
use scrypto::component::ComponentAddress;
use scrypto::core::{NativeFnIdentifier, Receiver};

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum MethodIdentifier {
    Scrypto {
        component_address: ComponentAddress,
        ident: String,
    },
    Native {
        receiver: Receiver,
        native_fn_identifier: NativeFnIdentifier,
    },
}