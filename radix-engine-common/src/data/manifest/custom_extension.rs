use super::*;
use crate::data::scrypto::*;
use crate::*;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ManifestCustomExtension {}

impl CustomExtension for ManifestCustomExtension {
    const MAX_DEPTH: usize = MANIFEST_SBOR_V1_MAX_DEPTH;
    const PAYLOAD_PREFIX: u8 = MANIFEST_SBOR_V1_PAYLOAD_PREFIX;

    type CustomValueKind = ManifestCustomValueKind;
    type CustomTraversal = ManifestCustomTraversal;
    // NOTE: ManifestSbor is actually validated against Scrypto schemas
    type CustomSchema = ScryptoCustomSchema;

    fn custom_value_kind_matches_type_kind<L: SchemaTypeLink>(
        custom_value_kind: Self::CustomValueKind,
        type_kind: &TypeKind<<Self::CustomSchema as CustomSchema>::CustomTypeKind<L>, L>,
    ) -> bool {
        match custom_value_kind {
            ManifestCustomValueKind::Address => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            ),
            ManifestCustomValueKind::Bucket => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Own))
            }
            ManifestCustomValueKind::Proof => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Own))
            }
            // Vec<Own> buckets - will check more in the payload_validator
            ManifestCustomValueKind::Expression => matches!(type_kind, TypeKind::Array { .. }),
            // Vec<u8> - will check more in the payload_validator
            ManifestCustomValueKind::Blob => matches!(type_kind, TypeKind::Array { .. }),
            ManifestCustomValueKind::Decimal => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Decimal))
            }
            ManifestCustomValueKind::PreciseDecimal => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal)
            ),
            ManifestCustomValueKind::NonFungibleLocalId => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId)
            ),
        }
    }

    fn custom_type_kind_matches_non_custom_value_kind<L: SchemaTypeLink>(
        _: &<Self::CustomSchema as CustomSchema>::CustomTypeKind<L>,
        _: ValueKind<Self::CustomValueKind>,
    ) -> bool {
        // No custom type kinds can match non-custom value kinds
        false
    }
}
