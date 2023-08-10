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
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::Always,
        }
    },
    collections: {
        blueprint_version_definitions: KeyValue {
            entry_ident: BlueprintVersionDefinition,
            key_type: {
                kind: Static,
                the_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            can_own: false,
        },
        blueprint_version_dependencies: KeyValue {
            entry_ident: BlueprintVersionDependencies,
            key_type: {
                kind: Static,
                the_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            can_own: false,
        },
        schemas: KeyValue {
            entry_ident: Schema,
            key_type: {
                kind: Static,
                the_type: SchemaHash,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            can_own: false,
        },
        blueprint_version_royalty_configs: KeyValue {
            entry_ident: BlueprintVersionRoyaltyConfig,
            key_type: {
                kind: Static,
                the_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            can_own: false,
        },
        blueprint_version_auth_configs: KeyValue {
            entry_ident: BlueprintVersionAuthConfig,
            key_type: {
                kind: Static,
                the_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            can_own: false,
        },
        code_vm_type: KeyValue {
            entry_ident: CodeVmType,
            key_type: {
                kind: Static,
                the_type: CodeHash,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            can_own: false,
        },
        code_original_code: KeyValue {
            entry_ident: CodeOriginalCode,
            key_type: {
                kind: Static,
                the_type: CodeHash,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            can_own: false,
        },
        code_instrumented_code: KeyValue {
            entry_ident: CodeInstrumentedCode,
            key_type: {
                kind: Static,
                the_type: CodeHash,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
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

//---------------------------------------
// Collection models - Schemas
//---------------------------------------

define_wrapped_hash!(
    /// Represents a particular schema under a package
    SchemaHash
);

// TODO(David): Change to VersionedSchema when can define a type as not-implicitly-versioned
// TODO: Move to Schema partition when we have it
pub type PackageSchemaValueV1 = ScryptoSchema;

//---------------------------------------
// Collection models - By BlueprintVersion
//---------------------------------------

pub type PackageBlueprintVersionDefinitionValueV1 = BlueprintDefinition;
pub type PackageBlueprintVersionDependenciesValueV1 = BlueprintDependencies;
pub type PackageBlueprintVersionRoyaltyConfigValueV1 = PackageRoyaltyConfig;
pub type PackageBlueprintVersionAuthConfigValueV1 = AuthConfig;

//---------------------------------------
// Collection models - By Code
//---------------------------------------

define_wrapped_hash!(
    /// Represents a particular instance of code under a package
    CodeHash
);

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct PackageCodeVmTypeValueV1 {
    pub vm_type: VmType,
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct PackageCodeOriginalCodeValueV1 {
    pub code: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct PackageCodeInstrumentedCodeValueV1 {
    pub code: Vec<u8>,
}