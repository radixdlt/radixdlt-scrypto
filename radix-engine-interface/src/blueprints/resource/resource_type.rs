use crate::internal_prelude::*;
use radix_common::data::scrypto::model::*;
use radix_common::math::Decimal;
use sbor::*;

use super::{check_fungible_amount, check_non_fungible_amount};

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
        match self {
            ResourceType::Fungible { divisibility } => {
                check_fungible_amount(&amount, *divisibility)
            }
            ResourceType::NonFungible { .. } => check_non_fungible_amount(&amount).is_ok(),
        }
    }
}
