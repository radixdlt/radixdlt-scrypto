use crate::model::ResourceManagerError;
use crate::model::{convert, InvokeError, MethodAuthorization};
use crate::types::*;

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct ResourceManagerSubstate {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub method_table: HashMap<ResourceManagerFnIdentifier, ResourceMethodRule>,
    pub vault_method_table: HashMap<VaultFnIdentifier, ResourceMethodRule>,
    pub bucket_method_table: HashMap<BucketFnIdentifier, ResourceMethodRule>,
    pub authorization: HashMap<ResourceMethodAuthKey, MethodAccessRule>,
    pub total_supply: Decimal,
    pub non_fungible_store_id: Option<NonFungibleStoreId>,
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub enum ResourceMethodRule {
    Public,
    Protected(ResourceMethodAuthKey),
}

#[derive(Debug, Clone, TypeId, Encode, Decode, PartialEq, Eq)]
pub struct MethodAccessRule {
    pub auth: MethodAuthorization,
    pub update_auth: MethodAuthorization,
}

pub enum MethodAccessRuleMethod {
    Lock(),
    Update(AccessRule),
}

/// Converts soft authorization rule to a hard authorization rule.
/// Currently required as all auth is defined by soft authorization rules.
macro_rules! convert_auth {
    ($auth:expr) => {
        convert(&Type::Unit, &ScryptoValue::unit(), &$auth)
    };
}

impl MethodAccessRule {
    // TODO: turn this into a proper node, i.e. id generation and invocation support

    pub fn new(entry: (AccessRule, Mutability)) -> Self {
        MethodAccessRule {
            auth: convert_auth!(entry.0),
            update_auth: match entry.1 {
                Mutability::LOCKED => MethodAuthorization::DenyAll,
                Mutability::MUTABLE(method_auth) => convert_auth!(method_auth),
            },
        }
    }

    pub fn get_method_auth(&self) -> &MethodAuthorization {
        &self.auth
    }

    pub fn get_update_auth(&self, method: MethodAccessRuleMethod) -> &MethodAuthorization {
        match method {
            MethodAccessRuleMethod::Lock() | MethodAccessRuleMethod::Update(_) => &self.update_auth,
        }
    }

    pub fn main(
        &mut self,
        method: MethodAccessRuleMethod,
    ) -> Result<ScryptoValue, InvokeError<ResourceManagerError>> {
        match method {
            MethodAccessRuleMethod::Lock() => self.lock(),
            MethodAccessRuleMethod::Update(method_auth) => {
                self.update(method_auth);
            }
        }

        Ok(ScryptoValue::from_typed(&()))
    }

    fn update(&mut self, method_auth: AccessRule) {
        self.auth = convert_auth!(method_auth)
    }

    fn lock(&mut self) {
        self.update_auth = MethodAuthorization::DenyAll;
    }
}
