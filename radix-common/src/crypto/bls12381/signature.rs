use crate::internal_prelude::*;
use blst::{
    min_pk::{AggregateSignature, Signature},
    BLST_ERROR,
};
use radix_rust::copy_u8_array;
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

/// BLS12-381 ciphersuite v1
/// It has following parameters
///  - hash-to-curve: BLS12381G2_XMD:SHA-256_SSWU_RO
///    - pairing-friendly elliptic curve: BLS12-381
///    - hash function: SHA-256
///    - signature variant: G2 minimal pubkey size
///  - scheme:
///    - proof-of-possession
/// More details: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04
pub const BLS12381_CIPHERSITE_V1: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

/// Represents a BLS12-381 G2 signature (variant with 96-byte signature and 48-byte public key)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Bls12381G2Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Bls12381G2Signature {
    pub const LENGTH: usize = 96;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn to_native_signature(self) -> Result<Signature, ParseBlsSignatureError> {
        Signature::from_bytes(&self.0).map_err(|err| err.into())
    }

    /// Aggregate multiple signatures into a single one.
    /// This method validates provided input signatures if `should_validate` flag is set.
    pub fn aggregate(
        signatures: &[Self],
        should_validate: bool,
    ) -> Result<Self, ParseBlsSignatureError> {
        if signatures.is_empty() {
            return Err(ParseBlsSignatureError::NoSignatureGiven);
        }
        let serialized_sigs = signatures
            .into_iter()
            .map(|sig| sig.as_ref())
            .collect::<Vec<_>>();

        let sig = AggregateSignature::aggregate_serialized(&serialized_sigs, should_validate)?
            .to_signature();

        Ok(Self(sig.to_bytes()))
    }

    /// Aggregate multiple signatures into a single one.
    /// This method does not validate provided input signatures, it is left
    /// here for backward compatibility.
    /// It is recommended to use `aggregate()` method instead.
    pub fn aggregate_anemone(signatures: &[Self]) -> Result<Self, ParseBlsSignatureError> {
        if !signatures.is_empty() {
            let sig_first = signatures[0].to_native_signature()?;

            let mut agg_sig = AggregateSignature::from_signature(&sig_first);

            for sig in signatures.iter().skip(1) {
                agg_sig.add_signature(&sig.to_native_signature()?, true)?;
            }
            Ok(Self(agg_sig.to_signature().to_bytes()))
        } else {
            Err(ParseBlsSignatureError::NoSignatureGiven)
        }
    }
}

impl TryFrom<&[u8]> for Bls12381G2Signature {
    type Error = ParseBlsSignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Self::LENGTH {
            return Err(ParseBlsSignatureError::InvalidLength(slice.len()));
        }

        Ok(Self(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Bls12381G2Signature {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Bls12381G2Signature {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//======
// error
//======

impl From<BLST_ERROR> for ParseBlsSignatureError {
    fn from(error: BLST_ERROR) -> Self {
        let err_msg = format!("{:?}", error);
        Self::BlsError(err_msg)
    }
}

/// Represents an error when retrieving BLS signature from hex or when aggregating.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseBlsSignatureError {
    InvalidHex(String),
    InvalidLength(usize),
    NoSignatureGiven,
    // Error returned by underlying BLS library
    BlsError(String),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBlsSignatureError {}

impl fmt::Display for ParseBlsSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for Bls12381G2Signature {
    type Err = ParseBlsSignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseBlsSignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Bls12381G2Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Bls12381G2Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    macro_rules! signature_validate {
        ($sig: expr) => {
            blst::min_pk::Signature::from_bytes(&$sig.0)
                .unwrap()
                .validate(true)
        };
    }

    #[test]
    fn signature_not_in_group() {
        let signature_not_in_group = "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f0";
        let signature_not_in_group = Bls12381G2Signature::from_str(signature_not_in_group).unwrap();
        let signature_valid = "82131f69b6699755f830e29d6ed41cbf759591a2ab598aa4e9686113341118d1db900d190436048601791121b5757c341045d4d0c94a95ec31a9ba6205f9b7504de85dadff52874375c58eec6cec28397279de87d5595101e398d31646d345bb";
        let signature_valid = Bls12381G2Signature::from_str(signature_valid).unwrap();

        assert_eq!(
            signature_validate!(signature_not_in_group),
            Err(blst::BLST_ERROR::BLST_POINT_NOT_IN_GROUP)
        );

        let sigs = vec![signature_not_in_group, signature_valid];

        let agg_sig = Bls12381G2Signature::aggregate(&sigs, true);

        assert_eq!(
            agg_sig,
            Err(ParseBlsSignatureError::BlsError(
                "BLST_POINT_NOT_IN_GROUP".to_string()
            ))
        );

        let sigs = vec![signature_valid, signature_not_in_group];

        let agg_sig = Bls12381G2Signature::aggregate(&sigs, true);

        assert_eq!(
            agg_sig,
            Err(ParseBlsSignatureError::BlsError(
                "BLST_POINT_NOT_IN_GROUP".to_string()
            ))
        );
    }
}
