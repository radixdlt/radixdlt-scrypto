use super::*;
use crate::internal_prelude::*;
use radix_common::data::scrypto::model::Own;
use radix_rust::rust::fmt::Debug;
use radix_rust::rust::prelude::*;

pub const METADATA_BLUEPRINT: &str = "Metadata";

pub const METADATA_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct MetadataCreateInput {}

pub type MetadataCreateOutput = Own;

pub const METADATA_CREATE_WITH_DATA_IDENT: &str = "create_with_data";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct MetadataCreateWithDataInput {
    pub data: MetadataInit,
}

pub type MetadataCreateWithDataOutput = Own;

pub const METADATA_SET_IDENT: &str = "set";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct MetadataSetInput {
    pub key: String,
    pub value: MetadataValue,
}

pub type MetadataSetOutput = ();

pub const METADATA_LOCK_IDENT: &str = "lock";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct MetadataLockInput {
    pub key: String,
}

pub type MetadataLockOutput = ();

pub const METADATA_GET_IDENT: &str = "get";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct MetadataGetInput {
    pub key: String,
}

pub type MetadataGetOutput = Option<MetadataValue>;

pub const METADATA_REMOVE_IDENT: &str = "remove";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct MetadataRemoveInput {
    pub key: String,
}

pub type MetadataRemoveOutput = bool;
