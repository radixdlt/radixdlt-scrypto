use super::*;
use crate::internal_prelude::*;

declare_native_blueprint_state! {
    blueprint_ident: Package,
    blueprint_snake_case: package,
    features: {
        package_royalty: {
            ident: PackageRoyalty,
            description: "Enables the package royalty substate",
        }
    },
    fields: {
        royalty:  {
            ident: RoyaltyAccumulator,
            field_type: {
                kind: StaticSingleVersioned,
            },
            condition: Condition::if_feature(PackageFeature::PackageRoyalty),
        }
    },
    collections: {
        blueprint_version_definitions: KeyValue {
            entry_ident: BlueprintVersionDefinition,
            key_type: {
                kind: Static,
                content_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
        blueprint_version_dependencies: KeyValue {
            entry_ident: BlueprintVersionDependencies,
            key_type: {
                kind: Static,
                content_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
        schemas: KeyValue {
            entry_ident: Schema,
            mapped_physical_partition: SCHEMAS_PARTITION,
            key_type: {
                kind: Static,
                content_type: SchemaHash,
            },
            value_type: {
                kind: Static,
                content_type: VersionedScryptoSchema,
            },
            allow_ownership: false,
        },
        blueprint_version_royalty_configs: KeyValue {
            entry_ident: BlueprintVersionRoyaltyConfig,
            key_type: {
                kind: Static,
                content_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
        blueprint_version_auth_configs: KeyValue {
            entry_ident: BlueprintVersionAuthConfig,
            key_type: {
                kind: Static,
                content_type: BlueprintVersionKey,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
        code_vm_type: KeyValue {
            entry_ident: CodeVmType,
            key_type: {
                kind: Static,
                content_type: CodeHash,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
        code_original_code: KeyValue {
            entry_ident: CodeOriginalCode,
            key_type: {
                kind: Static,
                content_type: CodeHash,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
        code_instrumented_code: KeyValue {
            entry_ident: CodeInstrumentedCode,
            key_type: {
                kind: Static,
                content_type: CodeHash,
            },
            value_type: {
                kind: StaticSingleVersioned,
            },
            allow_ownership: false,
        },
    }
}

//-------------
// Field models
//-------------

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
pub struct PackageRoyaltyAccumulatorV1 {
    /// The vault for collecting package royalties.
    pub royalty_vault: Vault,
}

//---------------------------------------
// Collection models - By BlueprintVersion
//---------------------------------------

pub type PackageBlueprintVersionDefinitionV1 = BlueprintDefinition;
pub type PackageBlueprintVersionDependenciesV1 = BlueprintDependencies;
pub type PackageBlueprintVersionRoyaltyConfigV1 = PackageRoyaltyConfig;
pub type PackageBlueprintVersionAuthConfigV1 = AuthConfig;

//---------------------------------------
// Collection models - By Code
//---------------------------------------

#[derive(Debug, PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct PackageCodeVmTypeV1 {
    pub vm_type: VmType,
}

#[derive(PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct PackageCodeOriginalCodeV1 {
    pub code: Vec<u8>,
}

impl Debug for PackageCodeOriginalCodeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageCodeOriginalCodeValueV1")
            .field("len", &self.code.len())
            .finish()
    }
}

#[derive(PartialEq, Eq, ScryptoSbor)]
#[sbor(transparent)]
pub struct PackageCodeInstrumentedCodeV1 {
    pub instrumented_code: Vec<u8>,
}

impl Debug for PackageCodeInstrumentedCodeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PackageCodeInstrumentedCodeValueV1")
            .field("len", &self.instrumented_code.len())
            .finish()
    }
}
