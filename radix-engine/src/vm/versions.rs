use radix_common::ScryptoSbor;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[repr(u8)]
pub enum ScryptoVmVersion {
    V1_0,
    V1_1,
    V1_2,
    V1_3,
}

impl ScryptoVmVersion {
    pub const fn latest() -> ScryptoVmVersion {
        Self::dugong()
    }

    pub const fn babylon_genesis() -> ScryptoVmVersion {
        Self::V1_0
    }

    pub const fn anemone() -> ScryptoVmVersion {
        Self::V1_1
    }

    pub const fn bottlenose() -> ScryptoVmVersion {
        Self::V1_1
    }

    pub const fn cuttlefish() -> ScryptoVmVersion {
        Self::V1_2
    }

    pub const fn dugong() -> ScryptoVmVersion {
        Self::V1_3
    }

    pub const fn crypto_utils_v1() -> ScryptoVmVersion {
        Self::anemone()
    }

    pub const fn crypto_utils_v2() -> ScryptoVmVersion {
        Self::cuttlefish()
    }

    pub const fn wasm_new_features() -> ScryptoVmVersion {
        Self::dugong()
    }
}

impl From<ScryptoVmVersion> for u64 {
    fn from(version: ScryptoVmVersion) -> Self {
        version as u64
    }
}

impl TryFrom<u64> for ScryptoVmVersion {
    type Error = ScryptoVmVersionError;

    fn try_from(version: u64) -> Result<Self, Self::Error> {
        match version {
            0 => Ok(Self::V1_0),
            1 => Ok(Self::V1_1),
            2 => Ok(Self::V1_2),
            3 => Ok(Self::V1_3),
            v => Err(Self::Error::FromIntError(v)),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub enum ScryptoVmVersionError {
    FromIntError(u64),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scrypto_vm_version() {
        let v = ScryptoVmVersion::latest();
        assert_eq!(v, ScryptoVmVersion::V1_3);
        assert_eq!(ScryptoVmVersion::crypto_utils_v1(), ScryptoVmVersion::V1_1);
    }

    #[test]
    fn test_scrypto_vm_version_conversions() {
        let v: u64 = ScryptoVmVersion::V1_1.into();
        assert_eq!(v, 1);

        let v: ScryptoVmVersion = 1u64.try_into().unwrap();
        assert_eq!(v, ScryptoVmVersion::V1_1);

        let e = ScryptoVmVersion::try_from(4u64).unwrap_err();

        assert_eq!(e, ScryptoVmVersionError::FromIntError(4u64));
    }

    #[test]
    fn test_scrypto_vm_version_ordering() {
        assert!(ScryptoVmVersion::crypto_utils_v1() == ScryptoVmVersion::V1_1);
        assert!(ScryptoVmVersion::crypto_utils_v1() > ScryptoVmVersion::V1_0);
        assert!(ScryptoVmVersion::crypto_utils_v1() < ScryptoVmVersion::crypto_utils_v2());

        assert!(ScryptoVmVersion::babylon_genesis() == ScryptoVmVersion::V1_0);
        assert!(ScryptoVmVersion::anemone() > ScryptoVmVersion::babylon_genesis());
        // Anemone == Bottlenose in terms of ScryptoVmVersion
        assert!(ScryptoVmVersion::bottlenose() == ScryptoVmVersion::anemone());
        assert!(ScryptoVmVersion::cuttlefish() > ScryptoVmVersion::bottlenose());
        assert!(ScryptoVmVersion::dugong() > ScryptoVmVersion::cuttlefish());
    }
}
