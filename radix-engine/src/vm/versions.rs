#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum ScryptoVmVersion {
    V1_0,
    V1_1,
}

impl ScryptoVmVersion {
    pub fn latest() -> ScryptoVmVersion {
        ScryptoVmVersion::V1_1
    }

    pub fn crypto_utils_added() -> ScryptoVmVersion {
        ScryptoVmVersion::V1_1
    }
}

impl From<ScryptoVmVersion> for u64 {
    fn from(version: ScryptoVmVersion) -> Self {
        version as u64
    }
}

impl From<u64> for ScryptoVmVersion {
    fn from(version: u64) -> Self {
        match version {
            0 => Self::V1_0,
            1 => Self::V1_1,
            v => panic!("ScryptoVmVersion {:?} not supported", v),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scrypto_vm_version() {
        let v = ScryptoVmVersion::latest();
        assert_eq!(v, ScryptoVmVersion::V1_1);

        let v: u64 = v.into();
        assert_eq!(v, 1);

        assert_eq!(
            ScryptoVmVersion::crypto_utils_added(),
            ScryptoVmVersion::V1_1
        );

        assert!(ScryptoVmVersion::crypto_utils_added() > ScryptoVmVersion::V1_0);
        assert!(ScryptoVmVersion::crypto_utils_added() == ScryptoVmVersion::V1_1);

        let v: ScryptoVmVersion = 1u64.into();
        assert_eq!(v, ScryptoVmVersion::V1_1)
    }
}
