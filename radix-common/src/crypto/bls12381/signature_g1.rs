use crate::internal_prelude::*;

/// BLS12-381 "minimal-signature-size" (min-sig) ciphersuite v1.
///
/// It has the following parameters:
///  - hash-to-curve: BLS12381G1_XMD:SHA-256_SSWU_RO
///    - pairing-friendly elliptic curve: BLS12-381
///    - hash function: SHA-256
///    - signature variant: G1 minimal signature size (48-byte signature, 96-byte public key)
///  - scheme:
///    - basic (NUL) — no proof-of-possession
///
/// This is the variant and domain separation tag used by unchained drand networks
/// (e.g. quicknet). It differs from [`crate::crypto::BLS12381_CIPHERSITE_V1`]
/// in both the signature group (G1 vs G2) and the scheme suffix (`NUL` vs `POP`).
///
/// More details: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-05
pub const BLS12381_MIN_SIG_CIPHERSITE_V1: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

/// Represents a BLS12-381 G1 signature (variant with 48-byte signature and 96-byte public key).
///
/// This is the signature of the BLS12-381 "minimal-signature-size" (min-sig) variant,
/// paired with [`Bls12381G2PublicKey`]. It is the variant used by unchained drand networks.
///
/// Note: this is distinct from [`Bls12381G2Signature`], which is the signature of the
/// "minimal-pubkey-size" (min-pk) variant used by [`verify_bls12381_v1`].
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Bls12381G1Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Bls12381G1Signature {
    pub const LENGTH: usize = 48;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for Bls12381G1Signature {
    type Error = ParseBlsSignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Self::LENGTH {
            return Err(ParseBlsSignatureError::InvalidLength(slice.len()));
        }

        Ok(Self(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Bls12381G1Signature {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Bls12381G1Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//======
// text
//======

impl FromStr for Bls12381G1Signature {
    type Err = ParseBlsSignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseBlsSignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Bls12381G1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Bls12381G1Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn g1_signature_text_round_trip() {
        // drand quicknet round 21180130 signature (G1, 48 bytes)
        let sig_hex = "96c36d9223c01c8c539c09573afdcad8ce626e0147b245bf9ad489fb01a49403b1588c8803972754b9d8ca8d6ac14319";
        let sig = Bls12381G1Signature::from_str(sig_hex).unwrap();
        assert_eq!(sig.to_string(), sig_hex);
        assert_eq!(sig.0.len(), Bls12381G1Signature::LENGTH);
    }

    #[test]
    fn g1_signature_invalid_length() {
        assert_eq!(
            Bls12381G1Signature::try_from([0u8; 96].as_slice()),
            Err(ParseBlsSignatureError::InvalidLength(96))
        );
    }
}
