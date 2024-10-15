use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use blst::{
    min_pk::{AggregatePublicKey, PublicKey},
    BLST_ERROR,
};

/// Represents a BLS12-381 G1 public key.
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Bls12381G1PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Bls12381G1PublicKey {
    pub const LENGTH: usize = 48;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn to_native_public_key(self) -> Result<PublicKey, ParseBlsPublicKeyError> {
        PublicKey::from_bytes(&self.0).map_err(|err| err.into())
    }

    /// Aggregate multiple public keys into a single one.
    /// This method validates provided input keys if `should_validate` flag is set.
    pub fn aggregate(
        public_keys: &[Self],
        should_validate: bool,
    ) -> Result<Self, ParseBlsPublicKeyError> {
        if public_keys.is_empty() {
            return Err(ParseBlsPublicKeyError::NoPublicKeysGiven);
        }
        let serialized_pks = public_keys
            .into_iter()
            .map(|pk| pk.as_ref())
            .collect::<Vec<_>>();

        let pk = AggregatePublicKey::aggregate_serialized(&serialized_pks, should_validate)?
            .to_public_key();

        Ok(Self(pk.to_bytes()))
    }

    /// Aggregate multiple public keys into a single one.
    /// This method does not validate provided input keys, it is left here
    /// for backward compatibility.
    /// It is recommended to use `aggregate()` method instead.
    pub fn aggregate_anemone(public_keys: &[Self]) -> Result<Self, ParseBlsPublicKeyError> {
        if !public_keys.is_empty() {
            let pk_first = public_keys[0].to_native_public_key()?;

            let mut agg_pk = AggregatePublicKey::from_public_key(&pk_first);

            for pk in public_keys.iter().skip(1) {
                agg_pk.add_public_key(&pk.to_native_public_key()?, true)?;
            }
            Ok(Self(agg_pk.to_public_key().to_bytes()))
        } else {
            Err(ParseBlsPublicKeyError::NoPublicKeysGiven)
        }
    }
}

impl TryFrom<&[u8]> for Bls12381G1PublicKey {
    type Error = ParseBlsPublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Self::LENGTH {
            return Err(ParseBlsPublicKeyError::InvalidLength(slice.len()));
        }

        Ok(Self(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Bls12381G1PublicKey {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Bls12381G1PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//======
// error
//======

impl From<BLST_ERROR> for ParseBlsPublicKeyError {
    fn from(error: BLST_ERROR) -> Self {
        let err_msg = format!("{:?}", error);
        Self::BlsError(err_msg)
    }
}

/// Represents an error when retrieving BLS public key from hex or when aggregating.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseBlsPublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
    NoPublicKeysGiven,
    // Error returned by underlying BLS library
    BlsError(String),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBlsPublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBlsPublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for Bls12381G1PublicKey {
    type Err = ParseBlsPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseBlsPublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Bls12381G1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Bls12381G1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    macro_rules! public_key_validate {
        ($pk: expr) => {
            blst::min_pk::PublicKey::from_bytes(&$pk.0)
                .unwrap()
                .validate()
        };
    }

    #[test]
    fn public_keys_not_in_group() {
        let public_key_valid = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
        let public_key_valid = Bls12381G1PublicKey::from_str(public_key_valid).unwrap();
        let public_key_not_in_group = "8bb1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
        let public_key_not_in_group =
            Bls12381G1PublicKey::from_str(public_key_not_in_group).unwrap();

        assert_eq!(
            public_key_validate!(public_key_not_in_group),
            Err(blst::BLST_ERROR::BLST_POINT_NOT_IN_GROUP)
        );

        let public_keys = vec![public_key_not_in_group, public_key_valid];

        let agg_pk = Bls12381G1PublicKey::aggregate(&public_keys, true);

        assert_eq!(
            agg_pk,
            Err(ParseBlsPublicKeyError::BlsError(
                "BLST_POINT_NOT_IN_GROUP".to_string()
            ))
        );

        let public_keys = vec![public_key_valid, public_key_not_in_group];

        let agg_pk = Bls12381G1PublicKey::aggregate(&public_keys, true);

        assert_eq!(
            agg_pk,
            Err(ParseBlsPublicKeyError::BlsError(
                "BLST_POINT_NOT_IN_GROUP".to_string()
            ))
        );
    }

    #[test]
    fn public_key_is_infinity() {
        let public_key_valid = "93b1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2";
        let public_key_valid = Bls12381G1PublicKey::from_str(public_key_valid).unwrap();
        let public_key_is_infinity =  "c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
        let public_key_is_infinity = Bls12381G1PublicKey::from_str(public_key_is_infinity).unwrap();

        assert_eq!(
            public_key_validate!(public_key_is_infinity),
            Err(blst::BLST_ERROR::BLST_PK_IS_INFINITY)
        );

        let public_keys = vec![public_key_is_infinity, public_key_valid];

        let agg_pk = Bls12381G1PublicKey::aggregate(&public_keys, true);

        assert_eq!(
            agg_pk,
            Err(ParseBlsPublicKeyError::BlsError(
                "BLST_PK_IS_INFINITY".to_string()
            ))
        );

        let public_keys = vec![public_key_is_infinity, public_key_valid];

        let agg_pk = Bls12381G1PublicKey::aggregate(&public_keys, true);

        assert_eq!(
            agg_pk,
            Err(ParseBlsPublicKeyError::BlsError(
                "BLST_PK_IS_INFINITY".to_string()
            ))
        );
    }
}
