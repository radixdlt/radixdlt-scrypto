use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

#[cfg_attr(
    feature = "fuzzing",
    derive(Arbitrary, serde::Serialize, serde::Deserialize)
)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, ManifestSbor, ScryptoCategorize, ScryptoEncode, ScryptoDecode,
)]
pub enum RoyaltyAmount {
    Free,
    Xrd(Decimal),
    Usd(Decimal),
}

impl Describe<ScryptoCustomTypeKind> for RoyaltyAmount {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::ROYALTY_AMOUNT_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
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

    pub fn is_negative(&self) -> bool {
        match self {
            Self::Free => false,
            Self::Usd(amount) | Self::Xrd(amount) => amount.is_negative(),
        }
    }
}
