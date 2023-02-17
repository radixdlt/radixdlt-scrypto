use sbor::*;

pub const VALUE_KIND_ADDRESS: u8 = 0x80;
pub const VALUE_KIND_OWN: u8 = 0x81;
pub const VALUE_KIND_DECIMAL: u8 = 0x85;
pub const VALUE_KIND_PRECISE_DECIMAL: u8 = 0x86;
pub const VALUE_KIND_NON_FUNGIBLE_LOCAL_ID: u8 = 0x87;
pub const VALUE_KIND_PUBLIC_KEY: u8 = 0x88;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValueKind {
    Address,
    Own,

    /* Uninterpreted, but so common; consistent with manifest model */
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
    PublicKey,
}

impl From<ScryptoCustomValueKind> for ValueKind<ScryptoCustomValueKind> {
    fn from(custom_value_kind: ScryptoCustomValueKind) -> Self {
        ValueKind::Custom(custom_value_kind)
    }
}

impl CustomValueKind for ScryptoCustomValueKind {
    fn as_u8(&self) -> u8 {
        match self {
            Self::Address => VALUE_KIND_ADDRESS,
            Self::Own => VALUE_KIND_OWN,
            Self::Decimal => VALUE_KIND_DECIMAL,
            Self::PreciseDecimal => VALUE_KIND_PRECISE_DECIMAL,
            Self::NonFungibleLocalId => VALUE_KIND_NON_FUNGIBLE_LOCAL_ID,
            Self::PublicKey => VALUE_KIND_PUBLIC_KEY,
        }
    }

    fn from_u8(id: u8) -> Option<Self> {
        match id {
            VALUE_KIND_ADDRESS => Some(ScryptoCustomValueKind::Address),
            VALUE_KIND_OWN => Some(ScryptoCustomValueKind::Own),
            VALUE_KIND_DECIMAL => Some(ScryptoCustomValueKind::Decimal),
            VALUE_KIND_PRECISE_DECIMAL => Some(ScryptoCustomValueKind::PreciseDecimal),
            VALUE_KIND_NON_FUNGIBLE_LOCAL_ID => Some(ScryptoCustomValueKind::NonFungibleLocalId),
            VALUE_KIND_PUBLIC_KEY => Some(ScryptoCustomValueKind::PublicKey),
            _ => None,
        }
    }
}
