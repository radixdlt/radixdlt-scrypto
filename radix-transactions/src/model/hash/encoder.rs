use bech32::{ToBase32, Variant};

use crate::internal_prelude::*;

pub struct TransactionHashBech32Encoder {
    pub hrp_set: HrpSet,
}

impl TransactionHashBech32Encoder {
    pub fn for_simulator() -> Self {
        Self::new(&NetworkDefinition::simulator())
    }

    pub fn new(network: &NetworkDefinition) -> Self {
        Self {
            hrp_set: network.into(),
        }
    }

    pub fn encode<T>(&self, hash: &T) -> Result<String, TransactionHashBech32EncodeError>
    where
        T: IsTransactionHash,
    {
        let mut buf = String::new();
        self.encode_to_fmt(&mut buf, hash)?;
        Ok(buf)
    }

    pub fn encode_to_fmt<T, F>(
        &self,
        fmt: &mut F,
        hash: &T,
    ) -> Result<(), TransactionHashBech32EncodeError>
    where
        T: IsTransactionHash,
        F: fmt::Write,
    {
        let hrp = hash.hrp(&self.hrp_set);
        let data = hash.as_inner_hash().as_slice();
        Self::encode_to_fmt_raw(fmt, hrp, data)
    }

    fn encode_to_fmt_raw<F: fmt::Write>(
        fmt: &mut F,
        hrp: &str,
        data: &[u8],
    ) -> Result<(), TransactionHashBech32EncodeError> {
        match bech32_encode_to_fmt(fmt, hrp, data.to_base32(), Variant::Bech32m) {
            Ok(Ok(())) => Ok(()),
            Ok(Err(format_error)) => {
                Err(TransactionHashBech32EncodeError::FormatError(format_error))
            }
            Err(encoding_error) => Err(TransactionHashBech32EncodeError::Bech32mEncodingError(
                encoding_error,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::internal_prelude::*;

    #[test]
    fn intent_hash_is_bech32_encoded_as_expected() {
        // Arrange
        let encoder = TransactionHashBech32Encoder::for_simulator();
        let transaction = transaction();
        let hash = transaction
            .prepare(PreparationSettings::latest_ref())
            .unwrap()
            .transaction_intent_hash();

        // Act
        let encoded = encoder.encode(&hash).unwrap();

        // Assert
        assert_eq!(
            encoded,
            "txid_sim1vrjkzlt8pekg5s46tum5na8lzpulvc3p72p92nkdm2dd8p0vkx2svr7ejr"
        )
    }

    #[test]
    fn signed_intent_hash_is_bech32_encoded_as_expected() {
        // Arrange
        let encoder = TransactionHashBech32Encoder::for_simulator();
        let transaction = transaction();
        let hash = transaction
            .prepare(PreparationSettings::latest_ref())
            .unwrap()
            .signed_transaction_intent_hash();

        // Act
        let encoded = encoder.encode(&hash).unwrap();

        // Assert
        assert_eq!(
            encoded,
            "signedintent_sim1c3f6q287pvw2pfs2extnh4yfmtc6ephgga7shf23nck85467026qrzn64x"
        )
    }

    #[test]
    fn notarized_transaction_hash_is_bech32_encoded_as_expected() {
        // Arrange
        let encoder = TransactionHashBech32Encoder::for_simulator();
        let transaction = transaction();
        let hash = transaction
            .prepare(PreparationSettings::latest_ref())
            .unwrap()
            .notarized_transaction_hash();

        // Act
        let encoded = encoder.encode(&hash).unwrap();

        // Assert
        assert_eq!(
            encoded,
            "notarizedtransaction_sim16aya9aqejr35u23g4gklcs3mya5nllxyy4y2y4yw9lur3wq6cdfsgpgkww"
        )
    }

    fn transaction() -> NotarizedTransactionV1 {
        let pk = Secp256k1PrivateKey::from_u64(1).unwrap();
        let manifest = ManifestBuilder::new().build();
        let header = TransactionHeaderV1 {
            network_id: 0xf2,
            start_epoch_inclusive: Epoch::of(0),
            end_epoch_exclusive: Epoch::of(10),
            nonce: 10,
            notary_is_signatory: true,
            notary_public_key: pk.public_key().into(),
            tip_percentage: 0,
        };
        TransactionBuilder::new()
            .manifest(manifest)
            .header(header)
            .notarize(&pk)
            .build()
    }
}
