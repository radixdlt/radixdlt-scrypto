use crate::types::*;
use radix_engine_interface::blueprints::resource::AccessRuleNode;

/// Authorization of a method call
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum MethodAuthorization {
    AllowAll,
    DenyAll,
    Protected(AccessRuleNode),
}
