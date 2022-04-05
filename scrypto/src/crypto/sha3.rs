use ::sha3::{Digest, Sha3_256};

use crate::crypto::*;

/// Computes the SHA3 digest of a message.
pub fn sha3<T: AsRef<[u8]>>(data: T) -> Hash {
    let mut instance = Sha3_256::new();
    instance.update(data);
    let result = instance.finalize();

    Hash(result.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::str::FromStr;

    #[test]
    fn test_sha3() {
        let data = "Hello Radix";
        let hash = sha3(data);
        assert_eq!(
            hash,
            Hash::from_str("b3b4d52dc67eda930a6cb35e8ebd2fb3c414706da8641c1a265f76bb660eb061")
                .unwrap()
        );
    }
}
