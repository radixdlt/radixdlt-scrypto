use scrypto::crypto::*;
use secp256k1::ecdsa::Signature;
use secp256k1::Message;
use secp256k1::PublicKey;

pub fn verify_ecdsa(
    message: &[u8],
    public_key: &EcdsaPublicKey,
    signature: &EcdsaSignature,
) -> bool {
    if let Ok(sig) = Signature::from_compact(&signature.0) {
        if let Ok(pk) = PublicKey::from_slice(&public_key.0) {
            let hash = hash(message);
            let msg = Message::from_slice(&hash.0).expect("Hash is always a valid message");
            return sig.verify(&msg, &pk).is_ok();
        }
    }

    false
}
