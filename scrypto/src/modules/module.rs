use crate::engine::scrypto_env::ScryptoEnv;
use crate::prelude::ScryptoEncode;
use crate::runtime::*;
use crate::*;
use radix_engine_derive::ScryptoSbor;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::api::{ClientActorApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::NodeId;
use radix_engine_interface::types::*;
use sbor::rust::marker::PhantomData;
use sbor::rust::ops::Deref;
use sbor::rust::prelude::*;
use scrypto::prelude::ScryptoDecode;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum ModuleHandle {
    Own(Own),
    Attached(GlobalAddress, ObjectModuleId),
    SELF(ObjectModuleId),
}

impl ModuleHandle {
    pub fn as_node_id(&self) -> &NodeId {
        match self {
            ModuleHandle::Own(own) => own.as_node_id(),
            ModuleHandle::SELF(..) | ModuleHandle::Attached(..) => panic!("invalid"),
        }
    }
}

pub struct Attached<'a, O>(pub O, pub PhantomData<&'a ()>);

impl<'a, O> Deref for Attached<'a, O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, O> Attached<'a, O> {
    pub fn new(o: O) -> Self {
        Attached(o, PhantomData::default())
    }
}

pub trait Attachable: Sized {
    const MODULE_ID: ObjectModuleId;

    fn attached(address: GlobalAddress) -> Self {
        Self::new(ModuleHandle::Attached(address, Self::MODULE_ID))
    }

    fn self_attached() -> Self {
        Self::new(ModuleHandle::SELF(Self::MODULE_ID))
    }

    fn new(handle: ModuleHandle) -> Self;

    fn handle(&self) -> &ModuleHandle;

    fn call<A: ScryptoEncode, T: ScryptoDecode>(&self, method: &str, args: &A) -> T {
        let args = scrypto_encode(args).unwrap();
        scrypto_decode(&self.call_raw(method, args)).unwrap()
    }

    fn call_raw(&self, method: &str, args: Vec<u8>) -> Vec<u8> {
        match self.handle() {
            ModuleHandle::Own(own) => {
                let output = ScryptoEnv
                    .call_method(own.as_node_id(), method, args)
                    .unwrap();
                output
            }
            ModuleHandle::Attached(address, module_id) => {
                let output = ScryptoEnv
                    .call_method_advanced(
                        address.as_node_id(),
                        false,
                        module_id.clone(),
                        method,
                        args,
                    )
                    .unwrap();
                output
            }
            ModuleHandle::SELF(module_id) => {
                let output = ScryptoEnv
                    .actor_call_module_method(OBJECT_HANDLE_SELF, *module_id, method, args)
                    .unwrap();
                output
            }
        }
    }

    fn call_ignore_rtn<A: ScryptoEncode>(&self, method: &str, args: &A) {
        let args = scrypto_encode(args).unwrap();
        match self.handle() {
            ModuleHandle::Own(own) => {
                ScryptoEnv
                    .call_method(own.as_node_id(), method, args)
                    .unwrap();
            }
            ModuleHandle::Attached(address, module_id) => {
                ScryptoEnv
                    .call_method_advanced(
                        address.as_node_id(),
                        false,
                        module_id.clone(),
                        method,
                        args,
                    )
                    .unwrap();
            }
            ModuleHandle::SELF(module_id) => {
                ScryptoEnv
                    .actor_call_module_method(OBJECT_HANDLE_SELF, *module_id, method, args)
                    .unwrap();
            }
        }
    }
}
