use crate::internal_prelude::*;

/// Represents a BLS12-381 G2 public key (96 bytes).
///
/// This is the public key of the BLS12-381 "minimal-signature-size" (min-sig) variant,
/// where signatures live in G1 (48 bytes) and public keys in G2 (96 bytes).
/// It is the variant used by unchained drand networks (e.g. quicknet) and is
/// paired with [`Bls12381G1Signature`] and the
/// [`crate::crypto::BLS12381_MIN_SIG_CIPHERSITE_V1`] domain separation tag.
///
/// Note: this is distinct from [`Bls12381G1PublicKey`], which is the public key of the
/// "minimal-pubkey-size" (min-pk) variant used by [`verify_bls12381_v1`].
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Bls12381G2PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Bls12381G2PublicKey {
    pub const LENGTH: usize = 96;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for Bls12381G2PublicKey {
    type Error = ParseBlsPublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Self::LENGTH {
            return Err(ParseBlsPublicKeyError::InvalidLength(slice.len()));
        }

        Ok(Self(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Bls12381G2PublicKey {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Bls12381G2PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//======
// text
//======

impl FromStr for Bls12381G2PublicKey {
    type Err = ParseBlsPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseBlsPublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Bls12381G2PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Bls12381G2PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn g2_public_key_text_round_trip() {
        // drand quicknet group public key (G2, 96 bytes)
        let pk_hex = "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a";
        let pk = Bls12381G2PublicKey::from_str(pk_hex).unwrap();
        assert_eq!(pk.to_string(), pk_hex);
        assert_eq!(pk.0.len(), Bls12381G2PublicKey::LENGTH);
    }

    #[test]
    fn g2_public_key_invalid_length() {
        assert_eq!(
            Bls12381G2PublicKey::try_from([0u8; 48].as_slice()),
            Err(ParseBlsPublicKeyError::InvalidLength(48))
        );
    }
}
