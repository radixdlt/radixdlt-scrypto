use scrypto::crypto::*;

pub fn verify_ecdsa(
    message: &[u8],
    public_key: &EcdsaPublicKey,
    signature: &EcdsaSignature,
) -> bool {
    if let Ok(sig) = secp256k1::ecdsa::Signature::from_compact(&signature.0) {
        if let Ok(pk) = secp256k1::PublicKey::from_slice(&public_key.0) {
            let hash = sha256(sha256(message));
            let msg =
                secp256k1::Message::from_slice(&hash.0).expect("Hash is always a valid message");
            return sig.verify(&msg, &pk).is_ok();
        }
    }

    false
}

pub fn verify_ed25519(
    message: &[u8],
    public_key: &Ed25519PublicKey,
    signature: &Ed25519Signature,
) -> bool {
    if let Ok(sig) = ed25519_dalek::Signature::from_bytes(&signature.0) {
        if let Ok(pk) = ed25519_dalek::PublicKey::from_bytes(&public_key.0) {
            return pk.verify_strict(message, &sig).is_ok();
        }
    }

    false
}
