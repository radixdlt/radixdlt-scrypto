use crate::blueprints::resource::*;
use crate::*;
use radix_engine_common::types::ComponentAddress;
use sbor::rust::fmt::Debug;

pub const IDENTITY_BLUEPRINT: &str = "Identity";

pub const IDENTITY_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct IdentityCreateInput {
    pub access_rule: AccessRule,
}

pub type IdentityCreateOutput = ComponentAddress;
