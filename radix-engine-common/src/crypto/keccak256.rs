use crate::crypto::*;
use sha3::{Digest, Keccak256};

pub fn keccak256_hash<T: AsRef<[u8]>>(data: T) -> Hash {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    Hash(hash.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sbor::rust::str::FromStr;

    #[test]
    fn test_keccak256_hash() {
        let data = "Hello Radix";
        let hash = keccak256_hash(data);
        assert_eq!(
            hash,
            Hash::from_str("415942230ddb029416a4612818536de230d827cbac9646a0b26d9855a4c45587")
                .unwrap()
        );
    }
}
