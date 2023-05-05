use crate::types::*;
use radix_engine_interface::blueprints::resource::ProofRule;

#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum HardAuthRule {
    ProofRule(ProofRule),
    AnyOf(Vec<HardAuthRule>),
    AllOf(Vec<HardAuthRule>),
}

/// Authorization of a method call
#[derive(Debug, Clone, PartialEq, Eq, Hash, ScryptoSbor)]
pub enum MethodAuthorization {
    AllowAll,
    DenyAll,
    Protected(HardAuthRule),
}
