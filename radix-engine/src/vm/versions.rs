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
    /// The latest Scrypto VM version known to this build.
    ///
    /// This is the dev-time validator ceiling — the newest version a package may be compiled
    /// and validated against (see [`crate::vm::wasm::ScryptoV1WasmValidator`]). It is decoupled
    /// from the VM version *enacted* at runtime, which is governed by the `VmBoot` substate set
    /// by protocol updates. V1_3 (Crypto Utils v3) is introduced by the Dugong protocol update
    /// but is not enacted by default (the runtime VmBoot remains at cuttlefish / V1_2 unless the
    /// corresponding Dugong VM-boot flash is enabled).
    pub const fn latest() -> ScryptoVmVersion {
        Self::V1_3
    }

    pub const fn babylon_genesis() -> ScryptoVmVersion {
        Self::V1_0
    }

    pub const fn anemone() -> ScryptoVmVersion {
        Self::V1_1
    }

    pub const fn cuttlefish() -> ScryptoVmVersion {
        Self::V1_2
    }

    pub const fn crypto_utils_v1() -> ScryptoVmVersion {
        Self::V1_1
    }

    pub const fn crypto_utils_v2() -> ScryptoVmVersion {
        Self::V1_2
    }

    /// Introduces Crypto Utils v3: BLS12-381 G1 (min-sig) signature verification
    /// ([`crypto_utils_bls12381_v1_verify_min_sig`]), used by unchained drand networks.
    ///
    /// Note: this VM version is introduced by the Dugong protocol update but is not enacted
    /// by default — the runtime `VmBoot` remains at cuttlefish / V1_2 unless the corresponding
    /// Dugong VM-boot flash is enabled. It is, however, the dev-time validator ceiling
    /// ([`Self::latest`]).
    pub const fn crypto_utils_v3() -> ScryptoVmVersion {
        Self::V1_3
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

        let v: ScryptoVmVersion = 3u64.try_into().unwrap();
        assert_eq!(v, ScryptoVmVersion::V1_3);

        let e = ScryptoVmVersion::try_from(4u64).unwrap_err();

        assert_eq!(e, ScryptoVmVersionError::FromIntError(4u64));
    }

    #[test]
    fn test_scrypto_vm_version_ordering() {
        assert!(ScryptoVmVersion::crypto_utils_v1() == ScryptoVmVersion::V1_1);
        assert!(ScryptoVmVersion::crypto_utils_v1() > ScryptoVmVersion::V1_0);
        assert!(ScryptoVmVersion::crypto_utils_v1() < ScryptoVmVersion::crypto_utils_v2());
        assert!(ScryptoVmVersion::crypto_utils_v2() < ScryptoVmVersion::crypto_utils_v3());
        assert!(ScryptoVmVersion::crypto_utils_v3() == ScryptoVmVersion::V1_3);
    }
}
