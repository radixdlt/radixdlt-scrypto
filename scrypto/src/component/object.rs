use crate::engine::scrypto_env::ScryptoEnv;
use crate::prelude::{scrypto_encode, ScryptoEncode, ScryptoSbor};
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::data::scrypto::{scrypto_decode, ScryptoDecode};
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
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

pub trait ObjectStub: Copy {
    type AddressType: TryFrom<[u8; NodeId::LENGTH]>;

    fn new(handle: ObjectStubHandle) -> Self;

    fn handle(&self) -> &ObjectStubHandle;

    fn call<A: ScryptoEncode, T: ScryptoDecode>(&self, method: &str, args: &A) -> T {
        let output = ScryptoEnv
            .call_method(
                self.handle().as_node_id(),
                method,
                scrypto_encode(args).unwrap(),
            )
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn call_ignore_rtn<A: ScryptoEncode>(&self, method: &str, args: &A) {
        ScryptoEnv
            .call_method(
                self.handle().as_node_id(),
                method,
                scrypto_encode(args).unwrap(),
            )
            .unwrap();
    }

    fn call_raw<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        let output = ScryptoEnv
            .call_method(self.handle().as_node_id(), method, args)
            .unwrap();
        scrypto_decode(&output).unwrap()
    }

    fn blueprint(&self) -> BlueprintId {
        ScryptoEnv
            .get_object_info(self.handle().as_node_id())
            .unwrap()
            .blueprint_id
    }
}
