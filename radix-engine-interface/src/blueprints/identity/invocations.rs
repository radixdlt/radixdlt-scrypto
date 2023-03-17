use std::collections::BTreeMap;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_common::data::scrypto::model::{ComponentAddress, Own};
use sbor::rust::fmt::Debug;
use crate::api::types::NodeModuleId;

pub const IDENTITY_BLUEPRINT: &str = "Identity";

pub const IDENTITY_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct IdentityCreateInput {
    pub access_rule: AccessRule,
}

pub type IdentityCreateOutput = ComponentAddress;


pub const IDENTITY_CREATE_VIRTUAL_ECDSA_IDENT: &str = "create_virtual_ecdsa";
pub const IDENTITY_CREATE_VIRTUAL_EDDSA_IDENT: &str = "create_virtual_eddsa";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VirtualLazyLoadInput {
    pub id: [u8; 26],
}

pub type VirtualLazyLoadOutput = (Own, BTreeMap<NodeModuleId, Own>);