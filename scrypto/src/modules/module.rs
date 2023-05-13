use std::marker::PhantomData;
use std::ops::Deref;
use radix_engine_derive::ScryptoSbor;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientActorApi, ClientBlueprintApi, OBJECT_HANDLE_SELF};
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::constants::METADATA_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};
use radix_engine_interface::types::NodeId;
use radix_engine_interface::types::*;
use sbor::rust::prelude::ToOwned;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
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

    fn call<T: ScryptoDecode>(&self, method: &str, args: Vec<u8>) -> T {
        match self.handle() {
            ModuleHandle::Own(own) => {
                let output = ScryptoEnv
                    .call_method(own.as_node_id(), method, args)
                    .unwrap();
                scrypto_decode(&output).unwrap()
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
                scrypto_decode(&output).unwrap()
            }
            ModuleHandle::SELF(module_id) => {
                let output = ScryptoEnv
                    .actor_call_module_method(
                        OBJECT_HANDLE_SELF,
                        *module_id,
                        method,
                        args,
                    )
                    .unwrap();
                scrypto_decode(&output).unwrap()
            }
        }
    }

    fn call_ignore_rtn(&self, method: &str, args: Vec<u8>) {
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
                    .actor_call_module_method(
                        OBJECT_HANDLE_SELF,
                        *module_id,
                        method,
                        args,
                    )
                    .unwrap();
            }
        }
    }
}