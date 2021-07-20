use sha2::{Digest, Sha256};

use crate::types::*;

/// Computes the SHA-256 digest of a message.
pub fn sha256<T: AsRef<[u8]>>(data: T) -> Hash {
    let mut instance = Sha256::new();
    instance.update(data);
    let result = instance.finalize();

    Hash::from_slice(&result).unwrap()
}

/// Computes the double SHA-256 digest of a message.
pub fn sha256_twice<T: AsRef<[u8]>>(data: T) -> Hash {
    sha256(sha256(data).as_slice())
}

#[cfg(test)]
mod tests {
    use crate::types::Hash;
    use crate::utils::*;

    #[test]
    fn test_sha256_twice() {
        let data = "Hello Radix";
        let hash = sha256_twice(data);
        assert_eq!(
            hash,
            Hash::from_hex("fd6be8b4b12276857ac1b63594bf38c01327bd6e8ae0eb4b0c6e253563cc8cc7")
                .unwrap()
        );
    }
}
