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

/// Performs BLS12-381 G1 signature verification (minimal-signature-size "min-sig" variant).
///
/// In this variant signatures live in G1 (48 bytes) and public keys in G2 (96 bytes) — the
/// mirror of [`verify_bls12381_v1`] (min-pk, with 96-byte G2 signatures and 48-byte G1 keys).
///
/// Domain specifier tag: BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_
///
/// This is the variant used by unchained drand networks (e.g. quicknet). The public
/// key is validated (in-group / not the identity) during verification, since in the on-chain
/// host-function context it is supplied as untrusted input.
pub fn verify_bls12381_v1_min_sig(
    message: &[u8],
    public_key: &Bls12381G2PublicKey,
    signature: &Bls12381G1Signature,
) -> bool {
    if let Ok(sig) = blst::min_sig::Signature::from_bytes(&signature.0) {
        if let Ok(pk) = blst::min_sig::PublicKey::from_bytes(&public_key.0) {
            let result = sig.verify(true, message, BLS12381_MIN_SIG_CIPHERSITE_V1, &[], &pk, true);

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

#[cfg(test)]
mod min_sig_tests {
    use super::*;
    use sbor::rust::str::FromStr;

    // Real drand quicknet (unchained, min-sig) group public key — G2, 96 bytes.
    const QUICKNET_PK: &str = "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a";

    // For unchained drand the signed message is SHA-256(round_number_as_big_endian_u64).
    // These digests are precomputed so the test needs no SHA-256 dependency in radix-common.

    fn pk() -> Bls12381G2PublicKey {
        Bls12381G2PublicKey::from_str(QUICKNET_PK).unwrap()
    }

    #[test]
    fn verify_min_sig_real_quicknet_round_21180130() {
        // round 21180130
        let message =
            hex::decode("7d2207c2d03c3c7561ea7fd7cc80d6f99434428528c266a74ca42766bed7d191")
                .unwrap();
        let sig = Bls12381G1Signature::from_str(
            "96c36d9223c01c8c539c09573afdcad8ce626e0147b245bf9ad489fb01a49403b1588c8803972754b9d8ca8d6ac14319",
        )
        .unwrap();

        assert!(verify_bls12381_v1_min_sig(&message, &pk(), &sig));
    }

    #[test]
    fn verify_min_sig_real_quicknet_round_21180131() {
        // round 21180131
        let message =
            hex::decode("a7ad19ac9c5255d6233274657a64696767c9c8e3d6391f38a790613201cd0b6f")
                .unwrap();
        let sig = Bls12381G1Signature::from_str(
            "966e214c7e632f006ade570591eabcdc9f709cb16b992d7d1658ad0e9b784ad5f54a9e3dc92f1118233aeb79c5a7d6b2",
        )
        .unwrap();

        assert!(verify_bls12381_v1_min_sig(&message, &pk(), &sig));
    }

    #[test]
    fn verify_min_sig_rejects_wrong_message() {
        let wrong_message =
            hex::decode("a7ad19ac9c5255d6233274657a64696767c9c8e3d6391f38a790613201cd0b6f")
                .unwrap(); // round 21180131's message
        let sig_for_130 = Bls12381G1Signature::from_str(
            "96c36d9223c01c8c539c09573afdcad8ce626e0147b245bf9ad489fb01a49403b1588c8803972754b9d8ca8d6ac14319",
        )
        .unwrap(); // round 21180130's signature

        assert!(!verify_bls12381_v1_min_sig(&wrong_message, &pk(), &sig_for_130));
    }

    #[test]
    fn verify_min_sig_rejects_zero_inputs() {
        let message =
            hex::decode("7d2207c2d03c3c7561ea7fd7cc80d6f99434428528c266a74ca42766bed7d191")
                .unwrap();
        let valid_sig = Bls12381G1Signature::from_str(
            "96c36d9223c01c8c539c09573afdcad8ce626e0147b245bf9ad489fb01a49403b1588c8803972754b9d8ca8d6ac14319",
        )
        .unwrap();

        // All-zero public key / signature are not valid encoded points and must be rejected.
        assert!(!verify_bls12381_v1_min_sig(
            &message,
            &Bls12381G2PublicKey([0u8; Bls12381G2PublicKey::LENGTH]),
            &valid_sig
        ));
        assert!(!verify_bls12381_v1_min_sig(
            &message,
            &pk(),
            &Bls12381G1Signature([0u8; Bls12381G1Signature::LENGTH])
        ));
    }

    #[test]
    fn verify_min_sig_rejects_points_not_in_group() {
        // Parity with the min-pk sibling tests (`signature_not_in_group`,
        // `public_keys_not_in_group`): a 96-byte G2 point outside the prime-order subgroup is
        // an invalid min-sig public key, and a 48-byte G1 point outside the subgroup is an
        // invalid min-sig signature. The subgroup checks (sig group-check + pk validation) must
        // reject both, returning `false` without panicking. (Note: a not-in-group point also
        // fails the pairing regardless of the validation flags, so this is a robustness /
        // panic-safety check, not a guarantee that the flags themselves are enabled.)
        let message =
            hex::decode("7d2207c2d03c3c7561ea7fd7cc80d6f99434428528c266a74ca42766bed7d191")
                .unwrap();

        // 96-byte G2 point not in group (reused as a public key).
        let pk_not_in_group = Bls12381G2PublicKey::from_str(
            "8b84ff5a1d4f8095ab8a80518ac99230ed24a7d1ec90c4105f9c719aa7137ed5d7ce1454d4a953f5f55f3959ab416f3014f4cd2c361e4d32c6b4704a70b0e2e652a908f501acb54ec4e79540be010e3fdc1fbf8e7af61625705e185a71c884f0",
        )
        .unwrap();
        let valid_sig = Bls12381G1Signature::from_str(
            "96c36d9223c01c8c539c09573afdcad8ce626e0147b245bf9ad489fb01a49403b1588c8803972754b9d8ca8d6ac14319",
        )
        .unwrap();
        assert!(!verify_bls12381_v1_min_sig(
            &message,
            &pk_not_in_group,
            &valid_sig
        ));

        // 48-byte G1 point not in group (reused as a signature).
        let sig_not_in_group = Bls12381G1Signature::from_str(
            "8bb1aa7542a5423e21d8e84b4472c31664412cc604a666e9fdf03baf3c758e728c7a11576ebb01110ac39a0df95636e2",
        )
        .unwrap();
        assert!(!verify_bls12381_v1_min_sig(&message, &pk(), &sig_not_in_group));
    }
}
