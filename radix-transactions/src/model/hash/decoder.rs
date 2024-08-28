use bech32::{FromBase32, Variant};

use crate::internal_prelude::*;

pub struct TransactionHashBech32Decoder {
    pub hrp_set: HrpSet,
}

impl TransactionHashBech32Decoder {
    pub fn for_simulator() -> Self {
        Self::new(&NetworkDefinition::simulator())
    }

    /// Instantiates a new TransactionHashBech32Decoder with the HRP corresponding to the passed network.
    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            hrp_set: network.into(),
        }
    }

    pub fn validate_and_decode<T>(&self, hash: &str) -> Result<T, TransactionHashBech32DecodeError>
    where
        T: IsTransactionHash,
    {
        // Decode the hash string
        let (hrp, data, variant) = bech32::decode(hash)
            .map_err(|err| TransactionHashBech32DecodeError::Bech32mDecodingError(err))?;

        // Validate the Bech32 variant to ensure that is is Bech32m
        match variant {
            Variant::Bech32m => {}
            _ => return Err(TransactionHashBech32DecodeError::InvalidVariant(variant)),
        };

        // Convert the data to u8 from u5.
        let data = Vec::<u8>::from_base32(&data)
            .map_err(|err| TransactionHashBech32DecodeError::Bech32mDecodingError(err))?;

        // Validate the length
        let hash = data
            .try_into()
            .map(Hash)
            .map_err(|_| TransactionHashBech32DecodeError::InvalidLength)?;

        // Validation complete, return data bytes
        T::create_from_hrp_and_hash(&hrp, hash, &self.hrp_set).map_err(|err| match err {
            HashCreationError::InvalidHrp => TransactionHashBech32DecodeError::InvalidHrp,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::internal_prelude::*;

    #[test]
    fn intent_hash_is_decoded_as_expected() {
        // Arrange
        let decoder = TransactionHashBech32Decoder::for_simulator();
        let encoded_hash = "txid_sim1vrjkzlt8pekg5s46tum5na8lzpulvc3p72p92nkdm2dd8p0vkx2svr7ejr";
        let expected_hash =
            Hash::from_str("60e5617d670e6c8a42ba5f3749f4ff1079f66221f282554ecdda9ad385ecb195")
                .unwrap();

        // Act
        let decoded = decoder
            .validate_and_decode::<TransactionIntentHash>(encoded_hash)
            .unwrap();

        // Assert
        assert_eq!(decoded.0, expected_hash)
    }

    #[test]
    fn signed_intent_hash_is_decoded_as_expected() {
        // Arrange
        let decoder = TransactionHashBech32Decoder::for_simulator();
        let encoded_hash =
            "signedintent_sim1c3f6q287pvw2pfs2extnh4yfmtc6ephgga7shf23nck85467026qrzn64x";
        let expected_hash =
            Hash::from_str("c453a028fe0b1ca0a60ac9973bd489daf1ac86e8477d0ba5519e2c7a575e7ab4")
                .unwrap();

        // Act
        let decoded = decoder
            .validate_and_decode::<SignedTransactionIntentHash>(encoded_hash)
            .unwrap();

        // Assert
        assert_eq!(decoded.0, expected_hash)
    }

    #[test]
    fn notarized_transaction_hash_is_decoded_as_expected() {
        // Arrange
        let decoder = TransactionHashBech32Decoder::for_simulator();
        let encoded_hash =
            "notarizedtransaction_sim16aya9aqejr35u23g4gklcs3mya5nllxyy4y2y4yw9lur3wq6cdfsgpgkww";
        let expected_hash =
            Hash::from_str("d749d2f41990e34e2a28aa2dfc423b27693ffcc42548a2548e2ff838b81ac353")
                .unwrap();

        // Act
        let decoded = decoder
            .validate_and_decode::<NotarizedTransactionHash>(encoded_hash)
            .unwrap();

        // Assert
        assert_eq!(decoded.0, expected_hash)
    }
}
