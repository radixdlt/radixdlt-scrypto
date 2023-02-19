use crate::abi::BlueprintAbi;
use crate::api::types::*;
use crate::blueprints::resource::AccessRules;
use sbor::rust::collections::BTreeMap;
use sbor::rust::vec::Vec;

pub trait ClientPackageApi<E> {
    fn new_package(
        &mut self,
        code: Vec<u8>,
        abi: Vec<u8>,
        access_rules: AccessRules,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
    ) -> Result<PackageAddress, E>;

    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;

    fn get_code(&mut self, package_address: PackageAddress) -> Result<PackageCode, E>;

    fn get_abi(
        &mut self,
        package_address: PackageAddress,
    ) -> Result<BTreeMap<String, BlueprintAbi>, E>;
}
