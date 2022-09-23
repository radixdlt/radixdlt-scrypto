use crate::model::MethodAuthorization;
use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ResourceManagerSubstate {
    resource_type: ResourceType,
    metadata: HashMap<String, String>,
    method_table: HashMap<ResourceManagerFnIdentifier, ResourceMethodRule>,
    vault_method_table: HashMap<VaultFnIdentifier, ResourceMethodRule>,
    bucket_method_table: HashMap<BucketFnIdentifier, ResourceMethodRule>,
    authorization: HashMap<ResourceMethodAuthKey, MethodAccessRule>,
    total_supply: Decimal,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum ResourceMethodRule {
    Public,
    Protected(ResourceMethodAuthKey),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct MethodAccessRule {
    auth: MethodAuthorization,
    update_auth: MethodAuthorization,
}
