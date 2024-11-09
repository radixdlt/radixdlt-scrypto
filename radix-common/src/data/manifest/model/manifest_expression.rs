use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ManifestExpression {
    /// Can be encoded into [`BucketBatch`]
    EntireWorktop,
    /// Can be encoded into [`ProofBatch`]
    EntireAuthZone,
}

//========
// Alternative Representations
//========

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestBucketBatch {
    ManifestBuckets(Vec<ManifestBucket>),
    EntireWorktop,
}

labelled_resolvable_with_identity_impl!(ManifestBucketBatch, resolver_output: ManifestBucket);

impl<T: LabelledResolve<Vec<ManifestBucket>>> LabelledResolveFrom<T> for ManifestBucketBatch {
    fn labelled_resolve_from(
        value: T,
        resolver: &impl LabelResolver<ManifestBucket>,
    ) -> ManifestBucketBatch {
        ManifestBucketBatch::ManifestBuckets(value.labelled_resolve(resolver))
    }
}

impl LabelledResolveFrom<ManifestExpression> for ManifestBucketBatch {
    fn labelled_resolve_from(
        value: ManifestExpression,
        _: &impl LabelResolver<ManifestBucket>,
    ) -> ManifestBucketBatch {
        match value {
            ManifestExpression::EntireWorktop => {
                // No named buckets are consumed - instead EntireWorktop refers only to the
                // unnamed buckets on the worktop part of the transaction processor
                ManifestBucketBatch::EntireWorktop
            }
            ManifestExpression::EntireAuthZone => {
                panic!("Not an allowed expression for a batch of buckets")
            }
        }
    }
}

impl ManifestBucketBatch {
    pub fn from_buckets(buckets: impl IntoIterator<Item = ManifestBucket>) -> Self {
        Self::ManifestBuckets(buckets.into_iter().collect())
    }
}

impl<E: sbor::Encoder<ManifestCustomValueKind>> sbor::Encode<ManifestCustomValueKind, E>
    for ManifestBucketBatch
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
        match self {
            ManifestBucketBatch::ManifestBuckets(buckets) => buckets.encode_value_kind(encoder),
            ManifestBucketBatch::EntireWorktop => {
                ManifestExpression::EntireWorktop.encode_value_kind(encoder)
            }
        }
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
        match self {
            ManifestBucketBatch::ManifestBuckets(buckets) => buckets.encode_body(encoder),
            ManifestBucketBatch::EntireWorktop => {
                ManifestExpression::EntireWorktop.encode_body(encoder)
            }
        }
    }
}

impl<D: sbor::Decoder<ManifestCustomValueKind>> sbor::Decode<ManifestCustomValueKind, D>
    for ManifestBucketBatch
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: sbor::ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, sbor::DecodeError> {
        Ok(match value_kind {
            ValueKind::Array => Self::ManifestBuckets(
                Vec::<ManifestBucket>::decode_body_with_value_kind(decoder, value_kind)?,
            ),
            ValueKind::Custom(_) => {
                let expression =
                    ManifestExpression::decode_body_with_value_kind(decoder, value_kind)?;
                if !matches!(expression, ManifestExpression::EntireWorktop) {
                    return Err(sbor::DecodeError::InvalidCustomValue);
                }
                Self::EntireWorktop
            }
            _ => {
                return Err(sbor::DecodeError::UnexpectedValueKind {
                    expected: ManifestValueKind::Array.as_u8(),
                    actual: value_kind.as_u8(),
                });
            }
        })
    }
}

impl sbor::Describe<ScryptoCustomTypeKind> for ManifestBucketBatch {
    const TYPE_ID: sbor::RustTypeId = Vec::<ManifestBucket>::TYPE_ID;

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, sbor::RustTypeId> {
        Vec::<ManifestBucket>::type_data()
    }
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ManifestProofBatch {
    ManifestProofs(Vec<ManifestProof>),
    EntireAuthZone,
}

labelled_resolvable_with_identity_impl!(ManifestProofBatch, resolver_output: ManifestProof);

impl<T: LabelledResolve<Vec<ManifestProof>>> LabelledResolveFrom<T> for ManifestProofBatch {
    fn labelled_resolve_from(
        value: T,
        resolver: &impl LabelResolver<ManifestProof>,
    ) -> ManifestProofBatch {
        ManifestProofBatch::ManifestProofs(value.labelled_resolve(resolver))
    }
}

impl LabelledResolveFrom<ManifestExpression> for ManifestProofBatch {
    fn labelled_resolve_from(
        value: ManifestExpression,
        _: &impl LabelResolver<ManifestProof>,
    ) -> ManifestProofBatch {
        match value {
            ManifestExpression::EntireWorktop => {
                panic!("Not an allowed expression for a batch of proofs");
            }
            ManifestExpression::EntireAuthZone => ManifestProofBatch::EntireAuthZone,
        }
    }
}

impl<E: sbor::Encoder<ManifestCustomValueKind>> sbor::Encode<ManifestCustomValueKind, E>
    for ManifestProofBatch
{
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
        match self {
            ManifestProofBatch::ManifestProofs(proofs) => proofs.encode_value_kind(encoder),
            ManifestProofBatch::EntireAuthZone => {
                ManifestExpression::EntireAuthZone.encode_value_kind(encoder)
            }
        }
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), sbor::EncodeError> {
        match self {
            ManifestProofBatch::ManifestProofs(proofs) => proofs.encode_body(encoder),
            ManifestProofBatch::EntireAuthZone => {
                ManifestExpression::EntireAuthZone.encode_body(encoder)
            }
        }
    }
}

impl<D: sbor::Decoder<ManifestCustomValueKind>> sbor::Decode<ManifestCustomValueKind, D>
    for ManifestProofBatch
{
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: sbor::ValueKind<ManifestCustomValueKind>,
    ) -> Result<Self, sbor::DecodeError> {
        Ok(match value_kind {
            ValueKind::Array => Self::ManifestProofs(
                Vec::<ManifestProof>::decode_body_with_value_kind(decoder, value_kind)?,
            ),
            ValueKind::Custom(_) => {
                let expression =
                    ManifestExpression::decode_body_with_value_kind(decoder, value_kind)?;
                if !matches!(expression, ManifestExpression::EntireAuthZone) {
                    return Err(sbor::DecodeError::InvalidCustomValue);
                }
                Self::EntireAuthZone
            }
            _ => {
                return Err(sbor::DecodeError::UnexpectedValueKind {
                    expected: ManifestValueKind::Array.as_u8(),
                    actual: value_kind.as_u8(),
                });
            }
        })
    }
}

impl sbor::Describe<ScryptoCustomTypeKind> for ManifestProofBatch {
    const TYPE_ID: sbor::RustTypeId = Vec::<ManifestProof>::TYPE_ID;

    fn type_data() -> sbor::TypeData<ScryptoCustomTypeKind, sbor::RustTypeId> {
        Vec::<ManifestProof>::type_data()
    }
}

//========
// error
//========

/// Represents an error when parsing ManifestExpression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestExpressionError {
    InvalidLength,
    UnknownExpression,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestExpressionError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestExpression {
    type Error = ParseManifestExpressionError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 1 {
            return Err(Self::Error::InvalidLength);
        }
        match slice[0] {
            0 => Ok(Self::EntireWorktop),
            1 => Ok(Self::EntireAuthZone),
            _ => Err(Self::Error::UnknownExpression),
        }
    }
}

impl ManifestExpression {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            ManifestExpression::EntireWorktop => {
                bytes.push(0);
            }
            ManifestExpression::EntireAuthZone => {
                bytes.push(1);
            }
        };
        bytes
    }
}

manifest_type!(ManifestExpression, ManifestCustomValueKind::Expression, 1);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_expression_parse_fail() {
        // wrong length
        let vec_err_1 = vec![1u8, 2];
        // wrong variant id
        let vec_err_2 = vec![10u8];

        let err1 = ManifestExpression::try_from(vec_err_1.as_slice());
        assert_matches!(err1, Err(ParseManifestExpressionError::InvalidLength));
        #[cfg(not(feature = "alloc"))]
        println!("Decoding manifest expression error: {}", err1.unwrap_err());

        let err2 = ManifestExpression::try_from(vec_err_2.as_slice());
        assert_matches!(err2, Err(ParseManifestExpressionError::UnknownExpression));
        #[cfg(not(feature = "alloc"))]
        println!("Decoding manifest expression error: {}", err2.unwrap_err());
    }

    #[test]
    fn manifest_expression_discriminator_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        // use invalid discriminator value
        encoder.write_discriminator(0xff).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let addr_output = decoder.decode_deeper_body_with_value_kind::<ManifestExpression>(
            ManifestExpression::value_kind(),
        );

        assert_matches!(addr_output, Err(DecodeError::InvalidCustomValue));
    }
}
