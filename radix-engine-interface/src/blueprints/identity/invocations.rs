use crate::blueprints::resource::*;
use crate::*;
use sbor::rust::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct IdentityCreateInput {
    pub access_rule: AccessRule,
}
