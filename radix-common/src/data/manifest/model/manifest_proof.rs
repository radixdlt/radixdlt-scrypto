use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

use crate::data::manifest::*;
use crate::*;

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[must_use]
pub struct ManifestProof(pub u32);

labelled_resolvable_with_identity_impl!(ManifestProof, resolver_output: Self);

//========
// error
//========

/// Represents an error when parsing ManifestProof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseManifestProofError {
    InvalidLength,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseManifestProofError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseManifestProofError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl TryFrom<&[u8]> for ManifestProof {
    type Error = ParseManifestProofError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != 4 {
            return Err(Self::Error::InvalidLength);
        }
        Ok(Self(u32::from_le_bytes(slice.try_into().unwrap())))
    }
}

impl ManifestProof {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

manifest_type!(ManifestProof, ManifestCustomValueKind::Proof, 4);
scrypto_describe_for_manifest_type!(ManifestProof, OWN_PROOF_TYPE, own_proof_type_data);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_proof_fail() {
        let proof = ManifestProof(37);
        let mut proof_vec = proof.to_vec();

        assert!(ManifestProof::try_from(proof_vec.as_slice()).is_ok());

        // malform encoded vector
        proof_vec.push(0);
        let proof_out = ManifestProof::try_from(proof_vec.as_slice());
        assert_matches!(proof_out, Err(ParseManifestProofError::InvalidLength));

        #[cfg(not(feature = "alloc"))]
        println!("Manifest Proof error: {}", proof_out.unwrap_err());
    }

    #[test]
    fn manifest_proof_encode_decode_fail() {
        let mut buf = Vec::new();
        let mut encoder = VecEncoder::<ManifestCustomValueKind>::new(&mut buf, 1);
        let malformed_value: u8 = 1; // use u8 instead of u32 should inovke an error
        encoder.write_slice(&malformed_value.to_le_bytes()).unwrap();

        let mut decoder = VecDecoder::<ManifestCustomValueKind>::new(&buf, 1);
        let proof_output = decoder
            .decode_deeper_body_with_value_kind::<ManifestProof>(ManifestProof::value_kind());

        // expecting 4 bytes, found only 1, so Buffer Underflow error should occur
        assert_matches!(proof_output, Err(DecodeError::BufferUnderflow { .. }));
    }
}
