use crate::data::scrypto::model::*;
use crate::{
    math::{BnumI256, Decimal},
    *,
};
use sbor::*;

/// Represents the type of a resource.
#[derive(Debug, Clone, Copy, Sbor, Eq, PartialEq)]
pub enum ResourceType {
    /// Represents a fungible resource
    Fungible { divisibility: u8 },

    /// Represents a non-fungible resource
    NonFungible { id_type: NonFungibleIdType },
}

impl ResourceType {
    pub fn divisibility(&self) -> Option<u8> {
        match self {
            ResourceType::Fungible { divisibility } => Some(*divisibility),
            ResourceType::NonFungible { .. } => None,
        }
    }

    pub fn id_type(&self) -> Option<NonFungibleIdType> {
        match self {
            ResourceType::Fungible { .. } => None,
            ResourceType::NonFungible { id_type } => Some(*id_type),
        }
    }

    pub fn is_fungible(&self) -> bool {
        match self {
            ResourceType::Fungible { .. } => true,
            ResourceType::NonFungible { .. } => false,
        }
    }

    pub fn check_amount(&self, amount: Decimal) -> bool {
        !amount.is_negative()
            && amount.0 % BnumI256::from(10i128.pow((18 - self.divisibility().unwrap_or(0)).into()))
                == BnumI256::from(0)
    }
}
