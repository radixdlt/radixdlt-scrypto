use crate::internal_prelude::*;

pub fn recover_secp256k1(
    signed_hash: &Hash,
    signature: &Secp256k1Signature,
) -> Option<Secp256k1PublicKey> {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if let Ok(id) = ::secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()) {
        if let Ok(sig) = ::secp256k1::ecdsa::RecoverableSignature::from_compact(signature_data, id)
        {
            let msg = ::secp256k1::Message::from_slice(&signed_hash.0)
                .expect("Hash is always a valid message");

            if let Ok(pk) = SECP256K1_CTX.recover_ecdsa(&msg, &sig) {
                return Some(Secp256k1PublicKey(pk.serialize()));
            }
        }
    }
    None
}

pub fn verify_secp256k1(
    signed_hash: &Hash,
    public_key: &Secp256k1PublicKey,
    signature: &Secp256k1Signature,
) -> bool {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if ::secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()).is_ok() {
        if let Ok(sig) = ::secp256k1::ecdsa::Signature::from_compact(signature_data) {
            if let Ok(pk) = ::secp256k1::PublicKey::from_slice(&public_key.0) {
                let msg = ::secp256k1::Message::from_slice(&signed_hash.0)
                    .expect("Hash is always a valid message");
                return SECP256K1_CTX.verify_ecdsa(&msg, &sig, &pk).is_ok();
            }
        }
    }

    false
}

pub fn verify_ed25519(
    signed_hash: &Hash,
    public_key: &Ed25519PublicKey,
    signature: &Ed25519Signature,
) -> bool {
    if let Ok(sig) = ed25519_dalek::Signature::from_bytes(&signature.0) {
        if let Ok(pk) = ed25519_dalek::PublicKey::from_bytes(&public_key.0) {
            return pk.verify_strict(&signed_hash.0, &sig).is_ok();
        }
    }

    false
}

/// Performs BLS12-381 G2 signature verification.
/// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
pub fn verify_bls12381_v1(
    message: &[u8],
    public_key: &Bls12381G1PublicKey,
    signature: &Bls12381G2Signature,
) -> bool {
    if let Ok(sig) = blst::min_pk::Signature::from_bytes(&signature.0) {
        if let Ok(pk) = blst::min_pk::PublicKey::from_bytes(&public_key.0) {
            let result = sig.verify(true, message, BLS12381_CIPHERSITE_V1, &[], &pk, true);

            match result {
                blst::BLST_ERROR::BLST_SUCCESS => return true,
                _ => return false,
            }
        }
    }

    false
}

/// Performs BLS12-381 G2 aggregated signature verification of
/// multiple messages each signed with different key.
/// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
pub fn aggregate_verify_bls12381_v1(
    pub_keys_and_msgs: &[(Bls12381G1PublicKey, Vec<u8>)],
    signature: &Bls12381G2Signature,
) -> bool {
    if let Ok(sig) = blst::min_pk::Signature::from_bytes(&signature.0) {
        let mut pks = vec![];
        let mut msg_refs = vec![];
        for (pk, msg) in pub_keys_and_msgs.iter() {
            if let Ok(pk) = blst::min_pk::PublicKey::from_bytes(&pk.0) {
                pks.push(pk);
            } else {
                return false;
            }
            msg_refs.push(msg.as_slice());
        }
        let pks_refs: Vec<&blst::min_pk::PublicKey> = pks.iter().collect();

        let result = sig.aggregate_verify(true, &msg_refs, BLS12381_CIPHERSITE_V1, &pks_refs, true);

        match result {
            blst::BLST_ERROR::BLST_SUCCESS => return true,
            _ => return false,
        }
    }

    false
}

/// Performs BLS12-381 G2 aggregated signature verification
/// one message signed with multiple keys.
/// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
pub fn fast_aggregate_verify_bls12381_v1(
    message: &[u8],
    public_keys: &[Bls12381G1PublicKey],
    signature: &Bls12381G2Signature,
) -> bool {
    if let Ok(agg_pk) = Bls12381G1PublicKey::aggregate(public_keys) {
        return verify_bls12381_v1(message, &agg_pk, signature);
    }

    false
}
