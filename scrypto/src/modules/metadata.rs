use crate::engine::scrypto_env::ScryptoEnv;
use crate::runtime::*;
use crate::*;
use radix_engine_interface::api::node_modules::metadata::*;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientBlueprintApi;
use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::constants::METADATA_MODULE_PACKAGE;
use radix_engine_interface::data::scrypto::ScryptoCustomValueKind;
use radix_engine_interface::data::scrypto::SCRYPTO_SBOR_V1_MAX_DEPTH;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::types::NodeId;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;
use sbor::Decoder;
use sbor::Encoder;
use sbor::ValueKind;
use sbor::VecDecoder;
use sbor::VecEncoder;
use sbor::OPTION_VARIANT_NONE;
use sbor::OPTION_VARIANT_SOME;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Metadata(pub Own);

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
        Self(metadata)
    }
}

impl MetadataObject for Metadata {
    fn self_id(&self) -> (&NodeId, ObjectModuleId) {
        (self.0.as_node_id(), ObjectModuleId::Main)
    }
}

#[derive(PartialEq, Eq, Hash)]
pub struct AttachedMetadata(pub GlobalAddress);

impl MetadataObject for AttachedMetadata {
    fn self_id(&self) -> (&NodeId, ObjectModuleId) {
        (self.0.as_node_id(), ObjectModuleId::Metadata)
    }
}

pub trait MetadataObject {
    fn self_id(&self) -> (&NodeId, ObjectModuleId);

    fn set<K: AsRef<str>, V: MetadataVal>(&self, name: K, value: V) {
        let (node_id, module_id) = self.self_id();

        let mut buffer = Vec::new();
        let mut encoder =
            VecEncoder::<ScryptoCustomValueKind>::new(&mut buffer, SCRYPTO_SBOR_V1_MAX_DEPTH);
        encoder.write_value_kind(ValueKind::Tuple).unwrap();
        encoder.write_size(2).unwrap();
        encoder.encode(name.as_ref()).unwrap();
        encoder.write_value_kind(ValueKind::Enum).unwrap();
        encoder.write_discriminator(V::TYPE_ID).unwrap();
        encoder.encode(&value).unwrap();

        ScryptoEnv
            .call_method_advanced(node_id, false, module_id, METADATA_SET_IDENT, buffer)
            .unwrap();
    }

    fn get<K: AsRef<str>, V: MetadataVal>(&self, name: K) -> Result<V, MetadataError> {
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

        // Option<MetadataValue>
        let mut decoder =
            VecDecoder::<ScryptoCustomValueKind>::new(&rtn, SCRYPTO_SBOR_V1_MAX_DEPTH);
        decoder.read_and_check_value_kind(ValueKind::Enum).unwrap();
        match decoder.read_discriminator().unwrap() {
            OPTION_VARIANT_NONE => {
                return Err(MetadataError::NotFound);
            }
            OPTION_VARIANT_SOME => {
                decoder.read_and_check_value_kind(ValueKind::Enum).unwrap();
                let id = decoder.read_discriminator().unwrap();

                if id == V::TYPE_ID {
                    let v: V = decoder.decode().unwrap();
                    return Ok(v);
                } else {
                    return Err(MetadataError::UnexpectedType {
                        expected_type_id: V::TYPE_ID,
                        actual_type_id: id,
                    });
                }
            }
            _ => unreachable!(),
        }
    }

    fn get_string<K: AsRef<str>>(&self, name: K) -> Result<String, MetadataError> {
        self.get(name)
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
