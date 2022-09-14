use scrypto::crypto::*;

pub fn recover(message: &[u8], signature: &SignatureWithPublicKey) -> Option<PublicKey> {
    match signature {
        SignatureWithPublicKey::EcdsaSecp256k1(sig) => {
            recover_ecdsa_secp256k1(message, sig).map(Into::into)
        }
        SignatureWithPublicKey::EddsaEd25519(pk, _) => Some(pk.clone().into()),
    }
}

pub fn recover_ecdsa_secp256k1(
    message: &[u8],
    signature: &EcdsaSecp256k1Signature,
) -> Option<EcdsaSecp256k1PublicKey> {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if let Ok(id) = secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()) {
        if let Ok(sig) = secp256k1::ecdsa::RecoverableSignature::from_compact(signature_data, id) {
            let hash = sha256(sha256(message));
            let msg =
                secp256k1::Message::from_slice(&hash.0).expect("Hash is always a valid message");
            if let Ok(pk) = sig.recover(&msg) {
                return Some(EcdsaSecp256k1PublicKey(pk.serialize()));
            }
        }
    }
    None
}

pub fn verify(message: &[u8], public_key: &PublicKey, signature: &Signature) -> bool {
    match (public_key, signature) {
        (PublicKey::EcdsaSecp256k1(pk), Signature::EcdsaSecp256k1(sig)) => {
            verify_ecdsa_secp256k1(message, pk, sig)
        }
        (PublicKey::EddsaEd25519(pk), Signature::EddsaEd25519(sig)) => {
            verify_eddsa_ed25519(message, pk, sig)
        }
        _ => false,
    }
}

pub fn verify_ecdsa_secp256k1(
    message: &[u8],
    public_key: &EcdsaSecp256k1PublicKey,
    signature: &EcdsaSecp256k1Signature,
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

pub fn verify_eddsa_ed25519(
    message: &[u8],
    public_key: &EddsaEd25519PublicKey,
    signature: &EddsaEd25519Signature,
) -> bool {
    if let Ok(sig) = ed25519_dalek::Signature::from_bytes(&signature.0) {
        if let Ok(pk) = ed25519_dalek::PublicKey::from_bytes(&public_key.0) {
            return pk.verify_strict(message, &sig).is_ok();
        }
    }

    false
}
