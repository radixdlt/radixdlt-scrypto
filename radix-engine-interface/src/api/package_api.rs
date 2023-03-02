use crate::api::types::*;
use crate::blueprints::resource::AccessRules;
use crate::data::scrypto::model::*;
use sbor::rust::prelude::*;
use scrypto_schema::PackageSchema;

pub trait ClientPackageApi<E> {
    fn new_package(
        &mut self,
        code: Vec<u8>,
        schema: PackageSchema,
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
}
