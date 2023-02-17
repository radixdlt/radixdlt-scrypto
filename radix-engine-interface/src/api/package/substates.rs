use crate::abi::*;
use crate::api::types::*;
use crate::data::types::Own;
use radix_engine_derive::*;
use sbor::rust::collections::*;
use sbor::rust::fmt::{Debug, Formatter};

pub const RESOURCE_MANAGER_PACKAGE_CODE_ID: u8 = 0u8;
pub const IDENTITY_PACKAGE_CODE_ID: u8 = 1u8;
pub const EPOCH_MANAGER_PACKAGE_CODE_ID: u8 = 2u8;
pub const CLOCK_PACKAGE_CODE_ID: u8 = 3u8;
pub const ACCOUNT_PACKAGE_CODE_ID: u8 = 4u8;
pub const ACCESS_CONTROLLER_PACKAGE_CODE_ID: u8 = 5u8;
pub const LOGGER_CODE_ID: u8 = 6u8;
pub const TRANSACTION_RUNTIME_CODE_ID: u8 = 7u8;
pub const AUTH_ZONE_CODE_ID: u8 = 8u8;
pub const METADATA_CODE_ID: u8 = 9u8;
pub const ROYALTY_CODE_ID: u8 = 10u8;

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct NativeCodeSubstate {
    pub native_package_code_id: u8,
}

/// A collection of blueprints, compiled and published as a single unit.
#[derive(Clone, Categorize, Encode, Decode, PartialEq, Eq)]
pub struct WasmCodeSubstate {
    pub code: Vec<u8>,
}

impl WasmCodeSubstate {
    pub fn code(&self) -> &[u8] {
        &self.code
    }
}

impl Debug for WasmCodeSubstate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("WasmCodeSubstate").finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackageInfoSubstate {
    pub blueprint_abis: BTreeMap<String, BlueprintAbi>,
    pub dependent_resources: BTreeSet<ResourceAddress>,
    pub dependent_components: BTreeSet<ComponentAddress>,
}

impl PackageInfoSubstate {
    pub fn blueprint_abi(&self, blueprint_name: &str) -> Option<&BlueprintAbi> {
        self.blueprint_abis.get(blueprint_name)
    }

    // TODO: Reorganize structure
    pub fn fn_abi(&self, export_name: &str) -> Option<&Fn> {
        for (_, abi) in &self.blueprint_abis {
            for function in &abi.fns {
                if export_name.eq(&function.export_name) {
                    return Some(function);
                }
            }
        }

        return None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackageRoyaltyConfigSubstate {
    pub royalty_config: BTreeMap<String, RoyaltyConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct PackageRoyaltyAccumulatorSubstate {
    pub royalty: Own,
}
