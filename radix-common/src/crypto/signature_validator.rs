use crate::internal_prelude::*;

#[cfg(feature = "secp256k1_sign_and_validate")]
pub fn verify_and_recover_secp256k1(
    signed_hash: &Hash,
    signature: &Secp256k1Signature,
) -> Option<Secp256k1PublicKey> {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if let Ok(id) = ::secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()) {
        if let Ok(sig) = ::secp256k1::ecdsa::RecoverableSignature::from_compact(signature_data, id)
        {
            let msg = ::secp256k1::Message::from_digest_slice(&signed_hash.0)
                .expect("Hash is always a valid message");

            // The recover method also verifies the signature as part of the recovery process
            if let Ok(pk) = SECP256K1_CTX.recover_ecdsa(&msg, &sig) {
                return Some(Secp256k1PublicKey(pk.serialize()));
            }
        }
    }
    None
}

#[cfg(feature = "secp256k1_sign_and_validate")]
pub fn verify_and_recover_secp256k1_uncompressed(
    signed_hash: &Hash,
    signature: &Secp256k1Signature,
) -> Option<Secp256k1UncompressedPublicKey> {
    let recovery_id = signature.0[0];
    let signature_data = &signature.0[1..];
    if let Ok(id) = ::secp256k1::ecdsa::RecoveryId::from_i32(recovery_id.into()) {
        if let Ok(sig) = ::secp256k1::ecdsa::RecoverableSignature::from_compact(signature_data, id)
        {
            let msg = ::secp256k1::Message::from_digest_slice(&signed_hash.0)
                .expect("Hash is always a valid message");

            // The recover method also verifies the signature as part of the recovery process
            if let Ok(pk) = SECP256K1_CTX.recover_ecdsa(&msg, &sig) {
                return Some(Secp256k1UncompressedPublicKey(pk.serialize_uncompressed()));
            }
        }
    }
    None
}

#[cfg(feature = "secp256k1_sign_and_validate")]
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
                let msg = ::secp256k1::Message::from_digest_slice(&signed_hash.0)
                    .expect("Hash is always a valid message");
                return SECP256K1_CTX.verify_ecdsa(&msg, &sig, &pk).is_ok();
            }
        }
    }

    false
}

pub fn verify_ed25519(
    message: impl AsRef<[u8]>,
    public_key: &Ed25519PublicKey,
    signature: &Ed25519Signature,
) -> bool {
    let sig = ed25519_dalek::Signature::from_bytes(&signature.0);
    if let Ok(pk) = ed25519_dalek::VerifyingKey::from_bytes(&public_key.0) {
        return pk.verify_strict(message.as_ref(), &sig).is_ok();
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

#[cfg(any(target_arch = "wasm32", feature = "alloc"))]
/// Local implementation of aggregated verify for no_std and WASM32 variants (no threads)
/// see: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-05#name-coreaggregateverify
/// Inspired with blst::min_pk::Signature::aggregate_verify
fn aggregate_verify_bls12381_v1_no_threads(
    pub_keys_and_msgs: &[(Bls12381G1PublicKey, Vec<u8>)],
    signature: blst::min_pk::Signature,
) -> bool {
    // Below structs are copies of PublicKey and Signature
    // Redefining them to be able to access point field, which is private for PublicKey and Signature
    struct LocalPublicKey {
        point: blst::blst_p1_affine,
    }
    struct LocalSignature {
        point: blst::blst_p2_affine,
    }
    let mut pairing = blst::Pairing::new(true, BLS12381_CIPHERSITE_V1);

    // Aggregate
    for (pk, msg) in pub_keys_and_msgs.iter() {
        if let Ok(pk) = blst::min_pk::PublicKey::from_bytes(&pk.0) {
            // transmute to LocalPublicKey to access point field
            let local_pk: LocalPublicKey = unsafe { core::mem::transmute(pk) };

            if pairing.aggregate(
                &local_pk.point,
                true,
                &unsafe { core::ptr::null::<blst::blst_p2_affine>().as_ref() },
                false,
                msg,
                &[],
            ) != blst::BLST_ERROR::BLST_SUCCESS
            {
                return false;
            }
        } else {
            return false;
        }
    }
    pairing.commit();

    if let Err(_err) = signature.validate(false) {
        return false;
    }

    // transmute to LocalSignature to access point field
    let local_sig: LocalSignature = unsafe { core::mem::transmute(signature) };
    let mut gtsig = blst::blst_fp12::default();
    blst::Pairing::aggregated(&mut gtsig, &local_sig.point);

    pairing.finalverify(Some(&gtsig))
}

/// Performs BLS12-381 G2 aggregated signature verification of
/// multiple messages each signed with different key.
/// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
pub fn aggregate_verify_bls12381_v1(
    pub_keys_and_msgs: &[(Bls12381G1PublicKey, Vec<u8>)],
    signature: &Bls12381G2Signature,
) -> bool {
    if let Ok(sig) = blst::min_pk::Signature::from_bytes(&signature.0) {
        #[cfg(not(any(target_arch = "wasm32", feature = "alloc")))]
        {
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

            let result =
                sig.aggregate_verify(true, &msg_refs, BLS12381_CIPHERSITE_V1, &pks_refs, true);

            matches!(result, blst::BLST_ERROR::BLST_SUCCESS)
        }

        #[cfg(any(target_arch = "wasm32", feature = "alloc"))]
        aggregate_verify_bls12381_v1_no_threads(pub_keys_and_msgs, sig)
    } else {
        false
    }
}

/// Performs BLS12-381 G2 aggregated signature verification
/// one message signed with multiple keys.
/// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
/// This method validates provided input keys when aggregating.
pub fn fast_aggregate_verify_bls12381_v1(
    message: &[u8],
    public_keys: &[Bls12381G1PublicKey],
    signature: &Bls12381G2Signature,
) -> bool {
    if let Ok(agg_pk) = Bls12381G1PublicKey::aggregate(public_keys, true) {
        return verify_bls12381_v1(message, &agg_pk, signature);
    }

    false
}

/// Performs BLS12-381 G2 aggregated signature verification
/// one message signed with multiple keys.
/// Domain specifier tag: BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_
/// This method does not validate provided input keys when aggregating,
/// it is left here for backward compatibility.
/// It is recommended to use [`fast_aggregate_verify_bls12381_v1()`] method instead.
pub fn fast_aggregate_verify_bls12381_v1_anemone(
    message: &[u8],
    public_keys: &[Bls12381G1PublicKey],
    signature: &Bls12381G2Signature,
) -> bool {
    if let Ok(agg_pk) = Bls12381G1PublicKey::aggregate_anemone(public_keys) {
        return verify_bls12381_v1(message, &agg_pk, signature);
    }

    false
}
