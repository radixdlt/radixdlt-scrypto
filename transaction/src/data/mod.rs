mod custom_value;
mod custom_value_kind;
pub mod model;

pub use custom_value::*;
pub use custom_value_kind::*;

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
