use radix_engine_common::data::manifest::model::*;
use radix_engine_common::data::manifest::*;
use radix_engine_common::types::NodeId;
use radix_engine_common::*;

#[derive(ManifestSbor, PartialEq, Eq, Debug)]
struct TestStruct {
    a: ManifestAddress,
    d: ManifestBucket,
    e: ManifestProof,
    f: ManifestExpression,
    g: ManifestBlobRef,
    h: ManifestDecimal,
    i: ManifestPreciseDecimal,
    j: ManifestNonFungibleLocalId,
}

#[test]
fn test_encode_and_decode() {
    let t = TestStruct {
        a: ManifestAddress(NodeId([0u8; 27])),
        d: ManifestBucket(4),
        e: ManifestProof(5),
        f: ManifestExpression::EntireAuthZone,
        g: ManifestBlobRef([6u8; 32]),
        h: ManifestDecimal([7u8; 32]),
        i: ManifestPreciseDecimal([8u8; 64]),
        j: ManifestNonFungibleLocalId::string("abc".to_owned()).unwrap(),
    };

    let bytes = manifest_encode(&t).unwrap();
    assert_eq!(
        bytes,
        vec![
            77, // prefix
            33, // struct
            8,  // field length
            128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, // address
            129, 4, 0, 0, 0, // bucket
            130, 5, 0, 0, 0, // proof
            131, 1, // expression
            132, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, // blob
            133, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
            7, 7, 7, 7, 7, // decimal
            134, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8, 8,
            8, 8, 8, 8, 8, 8, 8, 8, // precise decimal
            135, 0, 3, 97, 98, 99, // non-fungible local id
        ]
    );
    let decoded: TestStruct = manifest_decode(&bytes).unwrap();
    assert_eq!(decoded, t);
}
