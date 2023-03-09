use crate::api::types::*;
use crate::blueprints::resource::AccessRulesConfig;
use crate::data::scrypto::model::*;
use radix_engine_common::data::scrypto::ScryptoCustomTypeExtension;
use sbor::rust::prelude::*;
use sbor::{LocalTypeIndex, Schema};
use scrypto_schema::PackageSchema;

pub trait ClientPackageApi<E> {
    fn new_package(
        &mut self,
        code: Vec<u8>,
        schema: PackageSchema,
        access_rules: AccessRulesConfig,
        royalty_config: BTreeMap<String, RoyaltyConfig>,
        metadata: BTreeMap<String, String>,
        event_schema: BTreeMap<String, Vec<(LocalTypeIndex, Schema<ScryptoCustomTypeExtension>)>>,
    ) -> Result<PackageAddress, E>;

    fn call_function(
        &mut self,
        package_address: PackageAddress,
        blueprint_name: &str,
        function_name: &str,
        args: Vec<u8>,
    ) -> Result<Vec<u8>, E>;
}
