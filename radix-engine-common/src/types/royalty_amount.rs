use crate::math::*;
use crate::ScryptoSbor;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum RoyaltyAmount {
    Xrd(Decimal),
    Usd(Decimal),
}
