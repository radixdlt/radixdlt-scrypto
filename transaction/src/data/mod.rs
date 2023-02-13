mod custom_value;
mod custom_value_kind;
pub mod model;

pub use custom_value::*;
pub use custom_value_kind::*;

use sbor::*;

pub const MANIFEST_SBOR_V1_PAYLOAD_PREFIX: u8 = 77; // [M] ASCII code
pub const MANIFEST_SBOR_V1_MAX_DEPTH: u8 = 16;

pub type ManifestEncoder<'a> = VecEncoder<'a, ManifestCustomValueKind, MANIFEST_SBOR_V1_MAX_DEPTH>;
pub type ManifestDecoder<'a> = VecDecoder<'a, ManifestCustomValueKind, MANIFEST_SBOR_V1_MAX_DEPTH>;
pub type ManifestValueKind = ValueKind<ManifestCustomValueKind>;
pub type ManifestValue = Value<ManifestCustomValueKind, ManifestCustomValue>;

pub trait ManifestCategorize: Categorize<ManifestCustomValueKind> {}
impl<T: Categorize<ManifestCustomValueKind> + ?Sized> ManifestCategorize for T {}

pub trait ManifestDecode: for<'a> Decode<ManifestCustomValueKind, ManifestDecoder<'a>> {}
impl<T: for<'a> Decode<ManifestCustomValueKind, ManifestDecoder<'a>>> ManifestDecode for T {}

pub trait ManifestEncode: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> {}
impl<T: for<'a> Encode<ManifestCustomValueKind, ManifestEncoder<'a>> + ?Sized> ManifestEncode
    for T
{
}

pub fn manifest_encode<T: ManifestEncode + ?Sized>(value: &T) -> Result<Vec<u8>, EncodeError> {
    let mut buf = Vec::with_capacity(512);
    let encoder = ManifestEncoder::new(&mut buf);
    encoder.encode_payload(value, MANIFEST_SBOR_V1_PAYLOAD_PREFIX)?;
    Ok(buf)
}

pub fn manifest_decode<T: ManifestDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    ManifestDecoder::new(buf).decode_payload(MANIFEST_SBOR_V1_PAYLOAD_PREFIX)
}

#[cfg(test)]
mod tests {
    use super::model::*;
    use crate::*;

    #[derive(ManifestCategorize, ManifestEncode, ManifestDecode)]
    struct TestStruct {
        a: ManifestAddress,
        d: ManifestBucket,
        e: ManifestProof,
        f: ManifestExpression,
        g: ManifestBlobRef,
        h: ManifestDecimal,
        i: ManifestPreciseDecimal,
        j: ManifestNonFungibleLocalId,
        k: ManifestNonFungibleGlobalId,
    }

    #[test]
    fn test_encode_and_decode() {}
}
