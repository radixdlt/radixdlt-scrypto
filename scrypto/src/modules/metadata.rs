use std::marker::PhantomData;
use std::ops::Deref;
use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::constants::METADATA_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};
use radix_engine_interface::types::NodeId;
use radix_engine_interface::types::*;
use sbor::rust::prelude::ToOwned;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use scrypto::modules::Attachable;
use crate::modules::{Attached, ModuleHandle};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Metadata(pub ModuleHandle);

impl Attachable for Metadata {
    fn attached(address: GlobalAddress) -> Self {
        Metadata(ModuleHandle::Attached(address, ObjectModuleId::Metadata))
    }

    fn new(handle: ModuleHandle) -> Self {
        Metadata(handle)
    }

    fn handle(&self) -> &ModuleHandle {
        &self.handle()
    }
}

impl Metadata {
    pub fn new() -> Self {
        let rtn = ScryptoEnv
            .call_function(
                METADATA_MODULE_PACKAGE,
                METADATA_BLUEPRINT,
                METADATA_CREATE_IDENT,
                scrypto_encode(&MetadataCreateInput {}).unwrap(),
            )
            .unwrap();
        let metadata: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(metadata))
    }

    pub fn set_list<K: AsRef<str>>(&self, name: K, list: Vec<MetadataValue>) {
        let value: ScryptoValue =
            scrypto_decode(&scrypto_encode(&MetadataEntry::List(list)).unwrap()).unwrap();
        self.call_ignore_rtn(METADATA_SET_IDENT, scrypto_encode(&MetadataSetInput {
            key: name.as_ref().to_owned(),
            value,
        }).unwrap());
    }

    pub fn set<K: AsRef<str>, V: MetadataVal>(&self, name: K, value: V) {
        self.call_ignore_rtn(METADATA_SET_IDENT, scrypto_encode(&MetadataSetInput {
            key: name.as_ref().to_owned(),
            value: value.to_metadata_entry(),
        }).unwrap());
    }

    pub fn get_string<K: AsRef<str>>(&self, name: K) -> Result<String, MetadataError> {
        let value: Option<ScryptoValue> = self.call(METADATA_GET_IDENT, scrypto_encode(&MetadataGetInput {
            key: name.as_ref().to_owned(),
        }).unwrap());

        match value {
            None => Err(MetadataError::EmptyEntry),
            Some(value) => String::from_metadata_entry(value),
        }
    }

    pub fn remove<K: AsRef<str>>(&self, name: K) -> bool {
        let rtn = self.call(METADATA_REMOVE_IDENT, scrypto_encode(&MetadataRemoveInput {
            key: name.as_ref().to_owned(),
        }).unwrap());

        rtn
    }
}
