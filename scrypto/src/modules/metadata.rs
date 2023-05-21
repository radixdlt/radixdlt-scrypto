use crate::engine::scrypto_env::ScryptoEnv;
use crate::modules::ModuleHandle;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::constants::METADATA_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode, ScryptoValue};
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::modules::Attachable;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Metadata(pub ModuleHandle);

impl Attachable for Metadata {
    const MODULE_ID: ObjectModuleId = ObjectModuleId::Metadata;

    fn new(handle: ModuleHandle) -> Self {
        Metadata(handle)
    }

    fn handle(&self) -> &ModuleHandle {
        &self.0
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata::new()
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

    pub fn set_list<S: ToString>(&self, name: S, list: Vec<MetadataValue>) {
        let value: ScryptoValue =
            scrypto_decode(&scrypto_encode(&MetadataEntry::List(list)).unwrap()).unwrap();
        self.call_ignore_rtn(
            METADATA_SET_IDENT,
            &MetadataSetInput {
                key: name.to_string(),
                value,
            },
        );
    }

    pub fn set<S: ToString, V: MetadataVal>(&self, name: S, value: V) {
        self.call_ignore_rtn(
            METADATA_SET_IDENT,
            &MetadataSetInput {
                key: name.to_string(),
                value: value.to_metadata_entry(),
            },
        );
    }

    pub fn get_string<S: ToString>(&self, name: S) -> Result<String, MetadataError> {
        let value: Option<ScryptoValue> = self.call(
            METADATA_GET_IDENT,
            &MetadataGetInput {
                key: name.to_string(),
            },
        );

        match value {
            None => Err(MetadataError::EmptyEntry),
            Some(value) => String::from_metadata_entry(value),
        }
    }

    pub fn remove<S: ToString>(&self, name: S) -> bool {
        let rtn = self.call(
            METADATA_REMOVE_IDENT,
            &MetadataRemoveInput {
                key: name.to_string(),
            },
        );

        rtn
    }
}
