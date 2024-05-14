use radix_common::crypto::*;
use radix_common::types::*;

pub fn signature<T: HasPublicKeyHash>(public_key: T) -> NonFungibleGlobalId {
    NonFungibleGlobalId::from_public_key_hash(public_key.get_hash())
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use radix_common::crypto::*;

    #[test]
    fn can_define_a_rule_from_a_public_key_using_signature_function() {
        let key = [
            2, 89, 118, 196, 222, 207, 118, 167, 224, 167, 222, 238, 242, 218, 37, 31, 173, 46,
            217, 185, 176, 182, 124, 0, 115, 241, 243, 228, 46, 49, 221, 47, 113,
        ];
        let secp256k1_public_key = Secp256k1PublicKey(key);
        let ed25519_public_key = Secp256k1PublicKey(key);
        let public_key = PublicKey::Secp256k1(secp256k1_public_key);

        let _ = rule!(require(signature(secp256k1_public_key)));
        let _ = rule!(require(signature(ed25519_public_key)));
        let _ = rule!(require(signature(public_key)));
    }
}
