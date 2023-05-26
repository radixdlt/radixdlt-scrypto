use crate::math::*;
use crate::ManifestSbor;
use crate::ScryptoSbor;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub enum RoyaltyAmount {
    Free,
    Xrd(Decimal),
    Usd(Decimal),
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
