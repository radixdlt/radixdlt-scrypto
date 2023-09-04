use crate::internal_prelude::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(
    Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub enum RoyaltyAmount {
    Free,
    Xrd(Decimal),
    Usd(Decimal),
}

impl Describe<ScryptoCustomTypeKind> for RoyaltyAmount {
    const TYPE_ID: DefinitionTypeId =
        DefinitionTypeId::WellKnown(well_known_scrypto_custom_types::ROYALTY_AMOUNT_TYPE);

    fn type_data() -> ScryptoTypeData<DefinitionTypeId> {
        well_known_scrypto_custom_types::royalty_amount_type_data()
    }
}

impl RoyaltyAmount {
    pub fn is_zero(&self) -> bool {
        match self {
            RoyaltyAmount::Xrd(x) => x.is_zero(),
            RoyaltyAmount::Usd(x) => x.is_zero(),
            RoyaltyAmount::Free => true,
        }
    }

    pub fn is_non_zero(&self) -> bool {
        !self.is_zero()
    }
}
