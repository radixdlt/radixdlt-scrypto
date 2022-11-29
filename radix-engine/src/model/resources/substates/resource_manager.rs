use crate::model::{
    convert, InvokeError, MethodAuthorization, NonFungible, Resource, ResourceManagerError,
};
use crate::types::*;
use radix_engine_interface::api::types::NonFungibleStoreId;
use radix_engine_interface::data::IndexedScryptoValue;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerSubstate {
    pub resource_type: ResourceType,
    pub metadata: HashMap<String, String>,
    pub total_supply: Decimal,
    pub nf_store_id: Option<NonFungibleStoreId>,
    pub resource_address: ResourceAddress, // always set after instantiation
}

impl ResourceManagerSubstate {
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        nf_store_id: Option<NonFungibleStoreId>,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManagerSubstate, InvokeError<ResourceManagerError>> {
        let resource_manager = ResourceManagerSubstate {
            resource_type,
            metadata,
            total_supply: 0.into(),
            nf_store_id,
            resource_address,
        };

        Ok(resource_manager)
    }

    pub fn check_amount(&self, amount: Decimal) -> Result<(), InvokeError<ResourceManagerError>> {
        let divisibility = self.resource_type.divisibility();

        if amount.is_negative()
            || amount.0 % I256::from(10i128.pow((18 - divisibility).into())) != I256::from(0)
        {
            Err(InvokeError::Error(ResourceManagerError::InvalidAmount(
                amount,
                divisibility,
            )))
        } else {
            Ok(())
        }
    }

    pub fn burn(&mut self, amount: Decimal) {
        self.total_supply -= amount;
    }

    pub fn update_metadata(
        &mut self,
        new_metadata: HashMap<String, String>,
    ) -> Result<(), InvokeError<ResourceManagerError>> {
        self.metadata = new_metadata;

        Ok(())
    }

    pub fn mint(
        &mut self,
        mint_params: MintParams,
        self_address: ResourceAddress,
    ) -> Result<(Resource, HashMap<NonFungibleId, NonFungible>), InvokeError<ResourceManagerError>>
    {
        match mint_params {
            MintParams::Fungible { amount } => self.mint_fungible(amount, self_address),
            MintParams::NonFungible { entries } => self.mint_non_fungibles(entries, self_address),
        }
    }

    pub fn mint_fungible(
        &mut self,
        amount: Decimal,
        self_address: ResourceAddress,
    ) -> Result<(Resource, HashMap<NonFungibleId, NonFungible>), InvokeError<ResourceManagerError>>
    {
        if let ResourceType::Fungible { divisibility } = self.resource_type {
            // check amount
            self.check_amount(amount)?;

            // Practically impossible to overflow the Decimal type with this limit in place.
            if amount > dec!("1000000000000000000") {
                return Err(InvokeError::Error(
                    ResourceManagerError::MaxMintAmountExceeded,
                ));
            }

            self.total_supply += amount;

            Ok((
                Resource::new_fungible(self_address, divisibility, amount),
                HashMap::new(),
            ))
        } else {
            Err(InvokeError::Error(
                ResourceManagerError::ResourceTypeDoesNotMatch,
            ))
        }
    }

    pub fn mint_non_fungibles(
        &mut self,
        entries: HashMap<NonFungibleId, (Vec<u8>, Vec<u8>)>,
        self_address: ResourceAddress,
    ) -> Result<(Resource, HashMap<NonFungibleId, NonFungible>), InvokeError<ResourceManagerError>>
    {
        // check resource type
        if !matches!(self.resource_type, ResourceType::NonFungible) {
            return Err(InvokeError::Error(
                ResourceManagerError::ResourceTypeDoesNotMatch,
            ));
        }

        // check amount
        let amount: Decimal = entries.len().into();
        self.check_amount(amount)?;

        self.total_supply += amount;

        // Allocate non-fungibles
        let mut ids = BTreeSet::new();
        let mut non_fungibles = HashMap::new();
        for (id, data) in entries {
            let non_fungible = NonFungible::new(data.0, data.1);
            ids.insert(id.clone());
            non_fungibles.insert(id, non_fungible);
        }

        Ok((Resource::new_non_fungible(self_address, ids), non_fungibles))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
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
        convert(&Type::Unit, &IndexedScryptoValue::unit(), &$auth)
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
    ) -> Result<IndexedScryptoValue, InvokeError<ResourceManagerError>> {
        match method {
            MethodAccessRuleMethod::Lock() => self.lock(),
            MethodAccessRuleMethod::Update(method_auth) => {
                self.update(method_auth);
            }
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn update(&mut self, method_auth: AccessRule) {
        self.auth = convert_auth!(method_auth)
    }

    fn lock(&mut self) {
        self.update_auth = MethodAuthorization::DenyAll;
    }
}
