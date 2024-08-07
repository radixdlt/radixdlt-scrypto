use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ScryptoCustomExtension {}

impl CustomExtension for ScryptoCustomExtension {
    const PAYLOAD_PREFIX: u8 = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;

    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTraversal = ScryptoCustomTraversal;
    type CustomSchema = ScryptoCustomSchema;

    fn custom_value_kind_matches_type_kind(
        _: &Schema<Self::CustomSchema>,
        custom_value_kind: Self::CustomValueKind,
        type_kind: &LocalTypeKind<Self::CustomSchema>,
    ) -> bool {
        match custom_value_kind {
            ScryptoCustomValueKind::Reference => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            ),
            ScryptoCustomValueKind::Own => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Own))
            }
            ScryptoCustomValueKind::Decimal => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Decimal))
            }
            ScryptoCustomValueKind::PreciseDecimal => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal)
            ),
            ScryptoCustomValueKind::NonFungibleLocalId => matches!(
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
        // It's not possible for a custom type kind to match a non-custom value kind
        false
    }
}
