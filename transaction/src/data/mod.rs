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
    use radix_engine_interface::{
        api::types::{ComponentAddress, PackageAddress},
        blueprints::resource::{NonFungibleLocalId, ResourceAddress},
        crypto::{EcdsaSecp256k1PublicKey, PublicKey},
        math::{Decimal, PreciseDecimal},
    };

    use super::model::*;
    use crate::data::{manifest_decode, manifest_encode};
    use crate::*;

    #[derive(ManifestCategorize, ManifestEncode, ManifestDecode, PartialEq, Eq, Debug)]
    struct TestStruct {
        a: ManifestAddress,
        b: ManifestAddress,
        c: ManifestAddress,
        d: ManifestBucket,
        e: ManifestProof,
        f: ManifestExpression,
        g: ManifestBlobRef,
        h: ManifestDecimal,
        i: ManifestPreciseDecimal,
        j: ManifestNonFungibleLocalId,
        k: ManifestPublicKey,
    }

    #[test]
    fn test_encode_and_decode() {
        let t = TestStruct {
            a: ManifestAddress(
                PackageAddress::Normal([1u8; 26])
                    .to_vec()
                    .try_into()
                    .unwrap(),
            ),
            b: ManifestAddress(
                ComponentAddress::Normal([2u8; 26])
                    .to_vec()
                    .try_into()
                    .unwrap(),
            ),
            c: ManifestAddress(
                ResourceAddress::Normal([3u8; 26])
                    .to_vec()
                    .try_into()
                    .unwrap(),
            ),
            d: ManifestBucket(4),
            e: ManifestProof(5),
            f: ManifestExpression::EntireAuthZone,
            g: ManifestBlobRef([6u8; 32]),
            h: ManifestDecimal(Decimal::from(7u32)),
            i: ManifestPreciseDecimal(PreciseDecimal::from(8u32)),
            j: ManifestNonFungibleLocalId(NonFungibleLocalId::String("abc".to_owned())),
            k: ManifestPublicKey(PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PublicKey(
                [10u8; 33],
            ))),
        };

        let bytes = manifest_encode(&t).unwrap();
        assert_eq!(
            bytes,
            vec![
                77, // prefix
                33, // struct
                11, // field length
                128, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                1, // address
                128, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                2, // address
                128, 0, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
                3, // address
                129, 4, 0, 0, 0, // bucket
                130, 5, 0, 0, 0, // proof
                131, 1, // expression
                132, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, // blob
                133, 0, 0, 188, 147, 233, 254, 36, 97, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, // decimal
                134, 0, 0, 0, 0, 0, 0, 0, 0, 8, 248, 80, 251, 37, 107, 199, 113, 107, 191, 60, 213,
                166, 207, 255, 73, 31, 120, 194, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // precise decimal
                135, 0, 3, 97, 98, 99, // non-fungible local id
                136, 0, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
                10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10 // public key
            ]
        );
        let decoded: TestStruct = manifest_decode(&bytes).unwrap();
        assert_eq!(decoded, t);
    }
}
