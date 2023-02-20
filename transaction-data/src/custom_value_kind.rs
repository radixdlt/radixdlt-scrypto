use sbor::*;

pub const VALUE_KIND_ADDRESS: u8 = 0x80;
pub const VALUE_KIND_BUCKET: u8 = 0x81;
pub const VALUE_KIND_PROOF: u8 = 0x82;
pub const VALUE_KIND_EXPRESSION: u8 = 0x83;
pub const VALUE_KIND_BLOB: u8 = 0x84;
pub const VALUE_KIND_DECIMAL: u8 = 0x85;
pub const VALUE_KIND_PRECISE_DECIMAL: u8 = 0x86;
pub const VALUE_KIND_NON_FUNGIBLE_LOCAL_ID: u8 = 0x87;

#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(tag = "type")
)]
#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub enum ManifestCustomValueKind {
    Address,
    Bucket,
    Proof,
    Expression,
    Blob,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

impl From<ManifestCustomValueKind> for ValueKind<ManifestCustomValueKind> {
    fn from(custom_value_kind: ManifestCustomValueKind) -> Self {
        ValueKind::Custom(custom_value_kind)
    }
}

impl CustomValueKind for ManifestCustomValueKind {
    fn as_u8(&self) -> u8 {
        match self {
            Self::Address => VALUE_KIND_ADDRESS,
            Self::Bucket => VALUE_KIND_BUCKET,
            Self::Proof => VALUE_KIND_PROOF,
            Self::Expression => VALUE_KIND_EXPRESSION,
            Self::Blob => VALUE_KIND_BLOB,
            Self::Decimal => VALUE_KIND_DECIMAL,
            Self::PreciseDecimal => VALUE_KIND_PRECISE_DECIMAL,
            Self::NonFungibleLocalId => VALUE_KIND_NON_FUNGIBLE_LOCAL_ID,
        }
    }

    fn from_u8(id: u8) -> Option<Self> {
        match id {
            VALUE_KIND_ADDRESS => Some(ManifestCustomValueKind::Address),
            VALUE_KIND_BUCKET => Some(ManifestCustomValueKind::Bucket),
            VALUE_KIND_PROOF => Some(ManifestCustomValueKind::Proof),
            VALUE_KIND_EXPRESSION => Some(ManifestCustomValueKind::Expression),
            VALUE_KIND_BLOB => Some(ManifestCustomValueKind::Blob),
            VALUE_KIND_DECIMAL => Some(ManifestCustomValueKind::Decimal),
            VALUE_KIND_PRECISE_DECIMAL => Some(ManifestCustomValueKind::PreciseDecimal),
            VALUE_KIND_NON_FUNGIBLE_LOCAL_ID => Some(ManifestCustomValueKind::NonFungibleLocalId),
            _ => None,
        }
    }
}
