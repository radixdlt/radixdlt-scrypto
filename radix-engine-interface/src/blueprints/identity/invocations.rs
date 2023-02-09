use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::fmt::Debug;

pub const IDENTITY_BLUEPRINT: &str = "Identity";

pub const IDENTITY_CREATE_IDENT: &str = "create";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct IdentityCreateInput {
    pub access_rule: AccessRule,
}
