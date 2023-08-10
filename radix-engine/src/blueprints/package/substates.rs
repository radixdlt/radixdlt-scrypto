use crate::blueprints::macros::*;
use super::*;
use crate::types::*;

declare_native_blueprint_state!{
    blueprint_ident: Package,
    blueprint_snake_case: package,
    instance_schema_types: [],
    fields: {
        royalty:  {
            ident: RoyaltyAccumulator,
            field_type: StaticSingleVersioned,
            condition: Condition::Always,
        }
    },
    collections: {
        blueprint_definitions: KeyValue {
            entry_ident: BlueprintDefinition,
            key_type: Static {
                the_type: BlueprintVersionKey,
            },
            value_type: StaticSingleVersioned,
            can_own: false,
        },
    }
}

//-------------
// Field models
//-------------

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct PackageRoyaltyAccumulatorFieldV1 {
    /// The vault for collecting package royalties.
    pub royalty_vault: Vault,
}

//-------------
// Collection models
//-------------

pub type PackageBlueprintDefinitionValueV1 = BlueprintDefinition;