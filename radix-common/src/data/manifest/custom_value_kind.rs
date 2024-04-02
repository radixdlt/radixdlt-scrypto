use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

pub const MANIFEST_VALUE_KIND_ADDRESS: u8 = 0x80;
pub const MANIFEST_VALUE_KIND_BUCKET: u8 = 0x81;
pub const MANIFEST_VALUE_KIND_PROOF: u8 = 0x82;
pub const MANIFEST_VALUE_KIND_EXPRESSION: u8 = 0x83;
pub const MANIFEST_VALUE_KIND_BLOB: u8 = 0x84;
pub const MANIFEST_VALUE_KIND_DECIMAL: u8 = 0x85;
pub const MANIFEST_VALUE_KIND_PRECISE_DECIMAL: u8 = 0x86;
pub const MANIFEST_VALUE_KIND_NON_FUNGIBLE_LOCAL_ID: u8 = 0x87;
pub const MANIFEST_VALUE_KIND_ADDRESS_RESERVATION: u8 = 0x88;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
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
    AddressReservation,
}

impl From<ManifestCustomValueKind> for ValueKind<ManifestCustomValueKind> {
    fn from(custom_value_kind: ManifestCustomValueKind) -> Self {
        ValueKind::Custom(custom_value_kind)
    }
}

impl CustomValueKind for ManifestCustomValueKind {
    fn as_u8(&self) -> u8 {
        match self {
            Self::Address => MANIFEST_VALUE_KIND_ADDRESS,
            Self::Bucket => MANIFEST_VALUE_KIND_BUCKET,
            Self::Proof => MANIFEST_VALUE_KIND_PROOF,
            Self::Expression => MANIFEST_VALUE_KIND_EXPRESSION,
            Self::Blob => MANIFEST_VALUE_KIND_BLOB,
            Self::Decimal => MANIFEST_VALUE_KIND_DECIMAL,
            Self::PreciseDecimal => MANIFEST_VALUE_KIND_PRECISE_DECIMAL,
            Self::NonFungibleLocalId => MANIFEST_VALUE_KIND_NON_FUNGIBLE_LOCAL_ID,
            Self::AddressReservation => MANIFEST_VALUE_KIND_ADDRESS_RESERVATION,
        }
    }

    fn from_u8(id: u8) -> Option<Self> {
        match id {
            MANIFEST_VALUE_KIND_ADDRESS => Some(ManifestCustomValueKind::Address),
            MANIFEST_VALUE_KIND_BUCKET => Some(ManifestCustomValueKind::Bucket),
            MANIFEST_VALUE_KIND_PROOF => Some(ManifestCustomValueKind::Proof),
            MANIFEST_VALUE_KIND_EXPRESSION => Some(ManifestCustomValueKind::Expression),
            MANIFEST_VALUE_KIND_BLOB => Some(ManifestCustomValueKind::Blob),
            MANIFEST_VALUE_KIND_DECIMAL => Some(ManifestCustomValueKind::Decimal),
            MANIFEST_VALUE_KIND_PRECISE_DECIMAL => Some(ManifestCustomValueKind::PreciseDecimal),
            MANIFEST_VALUE_KIND_NON_FUNGIBLE_LOCAL_ID => {
                Some(ManifestCustomValueKind::NonFungibleLocalId)
            }
            MANIFEST_VALUE_KIND_ADDRESS_RESERVATION => {
                Some(ManifestCustomValueKind::AddressReservation)
            }
            _ => None,
        }
    }
}
