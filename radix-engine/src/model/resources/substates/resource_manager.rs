use crate::model::{InvokeError, NonFungible, Resource, ResourceManagerError};
use crate::types::*;
use radix_engine_interface::api::types::NonFungibleStoreId;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct ResourceManagerSubstate {
    pub resource_type: ResourceType,
    pub resource_address: ResourceAddress, // TODO: Figure out a way to remove?
    pub total_supply: Decimal,
    pub nf_store_id: Option<NonFungibleStoreId>,
}

impl ResourceManagerSubstate {
    pub fn new(
        resource_type: ResourceType,
        nf_store_id: Option<NonFungibleStoreId>,
        resource_address: ResourceAddress,
    ) -> Result<ResourceManagerSubstate, InvokeError<ResourceManagerError>> {
        let resource_manager = ResourceManagerSubstate {
            resource_type,
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
        let this_non_fungible_id_type = match self.resource_type {
            ResourceType::NonFungible { id_type } => id_type,
            _ => {
                return Err(InvokeError::Error(
                    ResourceManagerError::ResourceTypeDoesNotMatch,
                ))
            }
        };

        // check amount
        let amount: Decimal = entries.len().into();
        self.check_amount(amount)?;

        self.total_supply += amount;

        // Allocate non-fungibles
        let mut ids = BTreeSet::new();
        let mut non_fungibles = HashMap::new();
        for (id, data) in entries {
            if id.id_type() != this_non_fungible_id_type {
                return Err(InvokeError::Error(
                    ResourceManagerError::NonFungibleIdTypeDoesNotMatch(
                        id.id_type(),
                        this_non_fungible_id_type,
                    ),
                ));
            }

            let non_fungible = NonFungible::new(data.0, data.1);
            ids.insert(id.clone());
            non_fungibles.insert(id, non_fungible);
        }

        Ok((
            Resource::new_non_fungible(self_address, ids, this_non_fungible_id_type),
            non_fungibles,
        ))
    }
}
