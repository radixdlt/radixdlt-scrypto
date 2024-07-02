use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ManifestCustomExtension {}

impl CustomExtension for ManifestCustomExtension {
    const PAYLOAD_PREFIX: u8 = MANIFEST_SBOR_V1_PAYLOAD_PREFIX;

    type CustomValueKind = ManifestCustomValueKind;
    type CustomTraversal = ManifestCustomTraversal;
    // NOTE: ManifestSbor is actually validated against Scrypto schemas
    type CustomSchema = ScryptoCustomSchema;

    fn custom_value_kind_matches_type_kind(
        schema: &Schema<Self::CustomSchema>,
        custom_value_kind: Self::CustomValueKind,
        type_kind: &LocalTypeKind<Self::CustomSchema>,
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
            ManifestCustomValueKind::AddressReservation => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Own))
            }
            // An Expression can only be a Vec<Proof> or Vec<Manifest> at the moment
            // - in other words they're both a Vec<Own> at the TypeKind level
            ManifestCustomValueKind::Expression => matches!(
                type_kind,
                TypeKind::Array { element_type }
                    if match schema.resolve_type_kind(*element_type) {
                        Some(TypeKind::Custom(ScryptoCustomTypeKind::Own)) => true,
                        _ => false,
                    }
            ),
            ManifestCustomValueKind::Blob => matches!(
                type_kind,
                TypeKind::Array { element_type }
                    if match schema.resolve_type_kind(*element_type) {
                        Some(TypeKind::U8) => true,
                        _ => false,
                    }
            ),
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

    fn custom_type_kind_matches_non_custom_value_kind(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomSchema as CustomSchema>::CustomLocalTypeKind,
        _: ValueKind<Self::CustomValueKind>,
    ) -> bool {
        // No custom type kinds can match non-custom value kinds
        false
    }
}
