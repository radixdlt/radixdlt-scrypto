#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[must_use]
pub struct ManifestBucket(pub u32);

//========
// error
//========

/// Represents an error when parsing ManifestBucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestBucketError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestBucketError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestBucketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestBucket {
    type Error = ParseManifestBucketError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 4 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(u32::from_le_bytes(slice.try_into().unwrap())))
    }
}

impl ManifestBucket {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

manifest_type!(ManifestBucket, ManifestCustomValueKind::Bucket, 4);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_bucket_fail() {
        let bucket = ManifestBucket(0);
        let mut bucket_vec = bucket.to_vec();

        assert!(ManifestBucket::try_from(bucket_vec.as_slice()).is_ok());

        // malform encoded vector
        bucket_vec.push(0);
        let bucket_out = ManifestBucket::try_from(bucket_vec.as_slice());
        assert!(matches!(
            bucket_out,
            Err(ParseManifestBucketError::InvalidLength)
        ));

        println!("Manifest Bucket error: {}", bucket_out.unwrap_err());
    }

    #[test]
    fn manifest_bucket_encode_decode_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        let malformed_value: u8 = 1; // use u8 instead of u32 should inovke an error
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let bucket_output = decoder
            .decode_deeper_body_with_value_kind::<ManifestBucket>(ManifestBucket::value_kind());

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert!(matches!(
            bucket_output,
            Err(DecodeError::BufferUnderflow { .. })
        ));
    }
}
