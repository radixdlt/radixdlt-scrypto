use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::{AccessRules, Royalty};
use crate::prelude::well_known_scrypto_custom_types::{reference_type_data, REFERENCE_ID};
use crate::prelude::{scrypto_encode, ScryptoSbor};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientObjectApi};
use radix_engine_interface::data::scrypto::{
    scrypto_decode, ScryptoCustomTypeKind, ScryptoCustomValueKind, ScryptoDecode, ScryptoEncode,
};
use radix_engine_interface::types::*;
use sbor::rust::ops::Deref;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ObjectStubHandle {
    Own(Own),
    Global(GlobalAddress),
}

impl ObjectStubHandle {
    pub fn as_node_id(&self) -> &NodeId {
        match self {
            ObjectStubHandle::Own(own) => own.as_node_id(),
            ObjectStubHandle::Global(address) => address.as_node_id(),
        }
    }
}

pub trait ObjectStub {
    fn new(handle: ObjectStubHandle) -> Self;

    fn handle(&self) -> &ObjectStubHandle;

    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(self.handle().as_node_id(), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn call_ignore_rtn(&self, method: &str, args: Vec<u8>) {
        ScryptoEnv
            .call_method(self.handle().as_node_id(), method, args)
            .unwrap();
    }

    fn blueprint(&self) -> Blueprint {
        ScryptoEnv
            .get_object_info(self.handle().as_node_id())
            .unwrap()
            .blueprint
    }
}
