use crate::api::types::NodeModuleId;
use crate::blueprints::resource::*;
use crate::*;
use radix_engine_common::data::scrypto::model::{ComponentAddress, Own};
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;

pub const IDENTITY_BLUEPRINT: &str = "Identity";

pub const IDENTITY_CREATE_ADVANCED_IDENT: &str = "create_advanced";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct IdentityCreateAdvancedInput {
    pub access_rule: AccessRule,
    pub mutability: AccessRule,
}

pub type IdentityCreateOutput = ComponentAddress;

pub const IDENTITY_SECURIFY_TO_SINGLE_BADGE_IDENT: &str = "securify_to_single_badge";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct IdentitySecurifyToSingleBadgeInput {}

pub type IdentitySecurifyToSingleBadgeOutput = Bucket;

pub const IDENTITY_CREATE_VIRTUAL_ECDSA_256K1_IDENT: &str = "create_virtual_ecdsa_256k1";
pub const IDENTITY_CREATE_VIRTUAL_EDDSA_25519_IDENT: &str = "create_virtual_eddsa_25519";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct VirtualLazyLoadInput {
    pub id: [u8; 26],
}

pub type VirtualLazyLoadOutput = (Own, BTreeMap<NodeModuleId, Own>);
