use scrypto::crypto::*;

pub fn recover(message: &[u8], signature: &SignatureWithPublicKey) -> Option<PublicKey> {
    match signature {
        SignatureWithPublicKey::Ecdsa { signature } => {
            recover_ecdsa(message, signature).map(Into::into)
        }
        SignatureWithPublicKey::Ed25519 { public_key, .. } => Some(public_key.clone().into()),
    }
}

pub fn recover_ecdsa(message: &[u8], signature: &EcdsaSignature) -> Option<EcdsaPublicKey> {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if let Ok(id) = secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()) {
        if let Ok(sig) = secp256k1::ecdsa::RecoverableSignature::from_compact(signature_data, id) {
            let hash = sha256(sha256(message));
            let msg =
                secp256k1::Message::from_slice(&hash.0).expect("Hash is always a valid message");
            if let Ok(pk) = sig.recover(&msg) {
                return Some(EcdsaPublicKey(pk.serialize()));
            }
        }
    }
    None
}

pub fn verify(message: &[u8], public_key: &PublicKey, signature: &Signature) -> bool {
    match (public_key, signature) {
        (PublicKey::Ecdsa(pk), Signature::Ecdsa(sig)) => verify_ecdsa(message, pk, sig),
        (PublicKey::Ed25519(pk), Signature::Ed25519(sig)) => verify_ed25519(message, pk, sig),
        _ => false,
    }
}

pub fn verify_ecdsa(
    message: &[u8],
    public_key: &EcdsaPublicKey,
    signature: &EcdsaSignature,
) -> bool {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()).is_ok() {
        if let Ok(sig) = secp256k1::ecdsa::Signature::from_compact(signature_data) {
            if let Ok(pk) = secp256k1::PublicKey::from_slice(&public_key.0) {
                let hash = sha256(sha256(message));
                let msg = secp256k1::Message::from_slice(&hash.0)
                    .expect("Hash is always a valid message");
                return sig.verify(&msg, &pk).is_ok();
            }
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
