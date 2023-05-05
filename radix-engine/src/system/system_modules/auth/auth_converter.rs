use super::authorization::{
    MethodAuthorization,
};
use crate::types::*;
use radix_engine_interface::blueprints::resource::*;

/// Converts an `AccessRule` into a `MethodAuthorization`, with the given context of
/// Scrypto value and schema.
///
/// This method assumes that the value matches with the schema.
pub fn convert(
    method_auth: &AccessRule,
) -> MethodAuthorization {
    match method_auth {
        AccessRule::Protected(auth_rule) => MethodAuthorization::Protected(auth_rule.clone()),
        AccessRule::AllowAll => MethodAuthorization::AllowAll,
        AccessRule::DenyAll => MethodAuthorization::DenyAll,
    }
}
