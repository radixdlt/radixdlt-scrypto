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

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum ObjectType {
    Own(Own),
    Attached(GlobalAddress),
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Metadata(pub ObjectType);

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
        Self(ObjectType::Own(metadata))
    }

    pub(crate) fn attached<'a>(address: GlobalAddress) -> Attached<'a, Metadata> {
        Attached(Metadata(ObjectType::Attached(address)), PhantomData::default())
    }

    pub(crate) fn to_owned(self) -> Own {
        match self.0 {
            ObjectType::Own(own) => own,
            _ => panic!("oops"),

        }
    }
}

impl MetadataObject for Metadata {
    fn self_id(&self) -> (&NodeId, ObjectModuleId) {
        match &self.0 {
            ObjectType::Own(own) => {
                (own.as_node_id(), ObjectModuleId::Main)
            }
            ObjectType::Attached(global_address) => {
                (global_address.as_node_id(), ObjectModuleId::Metadata)
            }
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Attached<'a, O>(O, PhantomData<&'a ()>);

impl<'a, O> Deref for Attached<'a, O> {
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait MetadataObject {
    fn self_id(&self) -> (&NodeId, ObjectModuleId);

    fn set_list<K: AsRef<str>>(&self, name: K, list: Vec<MetadataValue>) {
        let (node_id, module_id) = self.self_id();

        let value: ScryptoValue =
            scrypto_decode(&scrypto_encode(&MetadataEntry::List(list)).unwrap()).unwrap();

        let _rtn = ScryptoEnv
            .call_method_advanced(
                node_id,
                false,
                module_id,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: name.as_ref().to_owned(),
                    value,
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn set<K: AsRef<str>, V: MetadataVal>(&self, name: K, value: V) {
        let (node_id, module_id) = self.self_id();

        let _rtn = ScryptoEnv
            .call_method_advanced(
                node_id,
                false,
                module_id,
                METADATA_SET_IDENT,
                scrypto_encode(&MetadataSetInput {
                    key: name.as_ref().to_owned(),
                    value: value.to_metadata_entry(),
                })
                .unwrap(),
            )
            .unwrap();
    }

    fn get_string<K: AsRef<str>>(&self, name: K) -> Result<String, MetadataError> {
        let (node_id, module_id) = self.self_id();

        let rtn = ScryptoEnv
            .call_method_advanced(
                node_id,
                false,
                module_id,
                METADATA_GET_IDENT,
                scrypto_encode(&MetadataGetInput {
                    key: name.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();

        let value: Option<ScryptoValue> = scrypto_decode(&rtn).unwrap();

        match value {
            None => Err(MetadataError::EmptyEntry),
            Some(value) => String::from_metadata_entry(value),
        }
    }

    fn remove<K: AsRef<str>>(&self, name: K) -> bool {
        let (node_id, module_id) = self.self_id();

        let rtn = ScryptoEnv
            .call_method_advanced(
                node_id,
                false,
                module_id,
                METADATA_REMOVE_IDENT,
                scrypto_encode(&MetadataRemoveInput {
                    key: name.as_ref().to_owned(),
                })
                .unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }
}
