use crate::engine::scrypto_env::ScryptoVmV1Api;
use crate::modules::ModuleHandle;
use crate::runtime::*;
use crate::*;
use radix_common::constants::METADATA_MODULE_PACKAGE;
use radix_common::data::scrypto::*;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::api::AttachedModuleId;
use radix_engine_interface::object_modules::metadata::*;
use sbor::rust::prelude::*;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::*;
use scrypto::modules::Attachable;

pub trait HasMetadata {
    fn set_metadata<K: AsRef<str>, V: MetadataVal>(&self, name: K, value: V);
    fn get_metadata<K: ToString, V: MetadataVal>(
        &self,
        name: K,
    ) -> Result<Option<V>, MetadataConversionError>;
    fn remove_metadata<K: ToString>(&self, name: K) -> bool;
    fn lock_metadata<K: ToString>(&self, name: K);
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Metadata(pub ModuleHandle);

impl Attachable for Metadata {
    const MODULE_ID: AttachedModuleId = AttachedModuleId::Metadata;

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
        let rtn = ScryptoVmV1Api::blueprint_call(
            METADATA_MODULE_PACKAGE,
            METADATA_BLUEPRINT,
            METADATA_CREATE_IDENT,
            scrypto_encode(&MetadataCreateInput {}).unwrap(),
        );
        let metadata: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(metadata))
    }

    pub fn new_with_data(data: MetadataInit) -> Self {
        let rtn = ScryptoVmV1Api::blueprint_call(
            METADATA_MODULE_PACKAGE,
            METADATA_BLUEPRINT,
            METADATA_CREATE_WITH_DATA_IDENT,
            scrypto_encode(&MetadataCreateWithDataInput { data }).unwrap(),
        );
        let metadata: Own = scrypto_decode(&rtn).unwrap();
        Self(ModuleHandle::Own(metadata))
    }

    pub fn set<K: AsRef<str>, V: MetadataVal>(&self, name: K, value: V) {
        // Manual encoding to avoid large code size
        // TODO: to replace with EnumVariant when it's ready
        let mut buffer = Vec::new();
        let mut encoder =
            VecEncoder::<ScryptoCustomValueKind>::new(&mut buffer, SCRYPTO_SBOR_V1_MAX_DEPTH);
        encoder
            .write_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        encoder.write_value_kind(ValueKind::Tuple).unwrap();
        encoder.write_size(2).unwrap();
        encoder.encode(name.as_ref()).unwrap();
        encoder.write_value_kind(ValueKind::Enum).unwrap();
        encoder.write_discriminator(V::DISCRIMINATOR).unwrap();
        encoder.write_size(1).unwrap();
        encoder.encode(&value).unwrap();

        self.call_raw(METADATA_SET_IDENT, buffer);
    }

    pub fn get<K: ToString, V: MetadataVal>(
        &self,
        name: K,
    ) -> Result<Option<V>, MetadataConversionError> {
        let rtn = self.call_raw(
            METADATA_GET_IDENT,
            scrypto_encode(&MetadataGetInput {
                key: name.to_string(),
            })
            .unwrap(),
        );

        // Manual decoding of Option<MetadataValue> to avoid large code size
        // TODO: to replace with EnumVariant when it's ready
        let mut decoder =
            VecDecoder::<ScryptoCustomValueKind>::new(&rtn, SCRYPTO_SBOR_V1_MAX_DEPTH);
        decoder
            .read_and_check_payload_prefix(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
            .unwrap();
        decoder.read_and_check_value_kind(ValueKind::Enum).unwrap();
        match decoder.read_discriminator().unwrap() {
            OPTION_VARIANT_NONE => {
                return Ok(None);
            }
            OPTION_VARIANT_SOME => {
                decoder.read_and_check_size(1).unwrap();
                decoder.read_and_check_value_kind(ValueKind::Enum).unwrap();
                let id = decoder.read_discriminator().unwrap();
                if id == V::DISCRIMINATOR {
                    decoder.read_and_check_size(1).unwrap();
                    let v: V = decoder.decode().unwrap();
                    return Ok(Some(v));
                } else {
                    return Err(MetadataConversionError::UnexpectedType {
                        expected_type_id: V::DISCRIMINATOR,
                        actual_type_id: id,
                    });
                }
            }
            _ => unreachable!(),
        }
    }

    pub fn get_string<K: ToString>(
        &self,
        name: K,
    ) -> Result<Option<String>, MetadataConversionError> {
        self.get(name)
    }

    pub fn remove<K: ToString>(&self, name: K) -> bool {
        let rtn = self.call(
            METADATA_REMOVE_IDENT,
            &MetadataRemoveInput {
                key: name.to_string(),
            },
        );

        rtn
    }

    pub fn lock<K: ToString>(&self, name: K) {
        let _: () = self.call(
            METADATA_LOCK_IDENT,
            &MetadataLockInput {
                key: name.to_string(),
            },
        );
    }
}
