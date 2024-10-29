#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_rust::copy_u8_array;
use sbor::rust::convert::TryFrom;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ManifestBlobRef(pub [u8; 32]);

//========
// error
//========

/// Represents an error when parsing ManifestBlobRef.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestBlobRefError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestBlobRefError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestBlobRefError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestBlobRef {
    type Error = ParseManifestBlobRefError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 32 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(copy_u8_array(slice)))
    }
}

impl ManifestBlobRef {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

manifest_type!(ManifestBlobRef, ManifestCustomValueKind::Blob, 32);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal_prelude::*;

    #[test]
    fn manifest_blob_parse_fail() {
        let buf = Vec::from_iter(0..32u8);

        let blob = ManifestBlobRef(buf.as_slice().try_into().unwrap());
        let mut blob_vec = blob.to_vec();

        assert!(ManifestBlobRef::try_from(blob_vec.as_slice()).is_ok());

        // malform encoded vector
        blob_vec.push(0);
        let blob_out = ManifestBlobRef::try_from(blob_vec.as_slice());
        assert_matches!(blob_out, Err(ParseManifestBlobRefError::InvalidLength));

        #[cfg(not(feature = "alloc"))]
        println!("Manifest Blob error: {}", blob_out.unwrap_err());
    }

    #[test]
    fn manifest_blob_encode_decode_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        let malformed_value: u8 = 0;
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let blob_output = decoder
            .decode_deeper_body_with_value_kind::<ManifestBlobRef>(ManifestBlobRef::value_kind());

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert_matches!(blob_output, Err(DecodeError::BufferUnderflow { .. }));
    }
}
