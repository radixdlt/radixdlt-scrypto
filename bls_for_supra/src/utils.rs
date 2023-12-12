use transaction::prelude::*;

pub fn create_notarized_transaction(
    network_definition: &NetworkDefinition,
    epoch: u64,
    private_key: &Secp256k1PrivateKey,
    manifest: TransactionManifestV1,
) -> (NotarizedTransactionV1, IntentHash) {
    let transaction = TransactionBuilder::new()
        .header(TransactionHeaderV1 {
            // Below params are just for the test.
            // They shall be adjusted with care and awareness.
            // Eg. in production nonce mustn't be hardcoded.
            network_id: network_definition.id,
            start_epoch_inclusive: Epoch::of(epoch),
            end_epoch_exclusive: Epoch::of(epoch + 10),
            nonce: 5,
            notary_public_key: private_key.public_key().into(),
            notary_is_signatory: false,
            tip_percentage: 0,
        })
        .manifest(manifest)
        .notarize(private_key)
        .build();

    let intent_hash = transaction.prepare().unwrap().intent_hash();

    (transaction, intent_hash)
}
