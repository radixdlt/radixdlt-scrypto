use crate::model::{InvokeError, Resource, ResourceManagerError};
use crate::types::*;
use radix_engine_interface::api::types::NonFungibleStoreId;
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct ResourceManagerSubstate {
    pub resource_address: ResourceAddress, // TODO: Figure out a way to remove?
    pub resource_type: ResourceType,
    pub total_supply: Decimal,
    pub nf_store_id: Option<NonFungibleStoreId>,
}

impl ResourceManagerSubstate {
    pub fn new(
        resource_type: ResourceType,
        nf_store_id: Option<NonFungibleStoreId>,
        resource_address: ResourceAddress,
    ) -> ResourceManagerSubstate {
        Self {
            resource_type,
            total_supply: 0.into(),
            nf_store_id,
            resource_address,
        }
    }

    pub fn check_fungible_amount(
        &self,
        amount: Decimal,
    ) -> Result<(), InvokeError<ResourceManagerError>> {
        let divisibility = self.resource_type.divisibility();

        if amount.is_negative()
            || amount.0 % BnumI256::from(10i128.pow((18 - divisibility).into()))
                != BnumI256::from(0)
        {
            Err(InvokeError::SelfError(ResourceManagerError::InvalidAmount(
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

    pub fn mint_fungible(
        &mut self,
        amount: Decimal,
        self_address: ResourceAddress,
    ) -> Result<Resource, InvokeError<ResourceManagerError>> {
        if let ResourceType::Fungible { divisibility } = self.resource_type {
            // check amount
            self.check_fungible_amount(amount)?;

            // Practically impossible to overflow the Decimal type with this limit in place.
            if amount > dec!("1000000000000000000") {
                return Err(InvokeError::SelfError(
                    ResourceManagerError::MaxMintAmountExceeded,
                ));
            }

            self.total_supply += amount;

            Ok(Resource::new_fungible(self_address, divisibility, amount))
        } else {
            Err(InvokeError::SelfError(
                ResourceManagerError::ResourceTypeDoesNotMatch,
            ))
        }
    }
}
