use radix_engine_interface::crypto::*;

use crate::{
    ecdsa_secp256k1::EcdsaSecp256k1Signature,
    eddsa_ed25519::EddsaEd25519Signature,
    model::{Signature, SignatureWithPublicKey},
};

pub fn recover(message: &[u8], signature: &SignatureWithPublicKey) -> Option<PublicKey> {
    let message_hash = hash(message);

    match signature {
        SignatureWithPublicKey::EcdsaSecp256k1 { signature } => {
            recover_ecdsa_secp256k1(&message_hash, signature).map(Into::into)
        }
        SignatureWithPublicKey::EddsaEd25519 { public_key, .. } => Some(public_key.clone().into()),
    }
}

pub fn recover_ecdsa_secp256k1(
    message_hash: &Hash,
    signature: &EcdsaSecp256k1Signature,
) -> Option<EcdsaSecp256k1PublicKey> {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if let Ok(id) = secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()) {
        if let Ok(sig) = secp256k1::ecdsa::RecoverableSignature::from_compact(signature_data, id) {
            let msg = secp256k1::Message::from_slice(&message_hash.0)
                .expect("Hash is always a valid message");
            if let Ok(pk) = sig.recover(&msg) {
                return Some(EcdsaSecp256k1PublicKey(pk.serialize()));
            }
        }
    }
    None
}

pub fn verify(message: &[u8], public_key: &PublicKey, signature: &Signature) -> bool {
    let message_hash = hash(message);

    match (public_key, signature) {
        (PublicKey::EcdsaSecp256k1(pk), Signature::EcdsaSecp256k1(sig)) => {
            verify_ecdsa_secp256k1(&message_hash, pk, sig)
        }
        (PublicKey::EddsaEd25519(pk), Signature::EddsaEd25519(sig)) => {
            verify_eddsa_ed25519(&message_hash, pk, sig)
        }
        _ => false,
    }
}

pub fn verify_ecdsa_secp256k1(
    message_hash: &Hash,
    public_key: &EcdsaSecp256k1PublicKey,
    signature: &EcdsaSecp256k1Signature,
) -> bool {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()).is_ok() {
        if let Ok(sig) = secp256k1::ecdsa::Signature::from_compact(signature_data) {
            if let Ok(pk) = secp256k1::PublicKey::from_slice(&public_key.0) {
                let msg = secp256k1::Message::from_slice(&message_hash.0)
                    .expect("Hash is always a valid message");
                return sig.verify(&msg, &pk).is_ok();
            }
        }
    }

    false
}

pub fn verify_eddsa_ed25519(
    message_hash: &Hash,
    public_key: &EddsaEd25519PublicKey,
    signature: &EddsaEd25519Signature,
) -> bool {
    if let Ok(sig) = ed25519_dalek::Signature::from_bytes(&signature.0) {
        if let Ok(pk) = ed25519_dalek::PublicKey::from_bytes(&public_key.0) {
            return pk.verify_strict(&message_hash.0, &sig).is_ok();
        }
    }

    false
}
