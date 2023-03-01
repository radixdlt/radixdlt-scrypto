use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::fmt::Debug;
use scrypto_schema::PackageSchema;

pub struct IdentityAbi;

impl IdentityAbi {
    pub fn schema() -> PackageSchema {
        PackageSchema::default()
    }
}

pub const IDENTITY_BLUEPRINT: &str = "Identity";

pub const IDENTITY_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct IdentityCreateInput {
    pub access_rule: AccessRule,
}
