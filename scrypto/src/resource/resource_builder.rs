use sbor::*;

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::BTreeMap;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::types::*;

/// Utility for creating a resource
pub struct ResourceBuilder {
    metadata: HashMap<String, String>,
}

impl ResourceBuilder {
    /// Starts a new builder.
    pub fn new() -> Self {
        Self {
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata attribute.
    pub fn metadata<K: AsRef<str>, V: AsRef<str>>(&mut self, name: K, value: V) -> &mut Self {
        self.metadata
            .insert(name.as_ref().to_owned(), value.as_ref().to_owned());
        self
    }

    /// Creates a token resource with mutable supply.
    pub fn new_token_mutable<A: Into<ResourceDef>>(&self, minter: A) -> ResourceDef {
        ResourceDef::new_mutable(
            ResourceType::Fungible {
                granularity: 1.into(),
            },
            self.metadata.clone(),
            minter.into().address(),
        )
    }

    /// Creates a token resource with fixed supply.
    pub fn new_token_fixed<T: Into<Decimal>>(&self, supply: T) -> Bucket {
        ResourceDef::new_fixed(
            ResourceType::Fungible {
                granularity: 1.into(),
            },
            self.metadata.clone(),
            ResourceSupply::Fungible {
                amount: supply.into(),
            },
        )
        .1
    }

    /// Creates a badge resource with mutable supply.
    pub fn new_badge_mutable<A: Into<ResourceDef>>(&self, minter: A) -> ResourceDef {
        ResourceDef::new_mutable(
            ResourceType::Fungible {
                granularity: 19.into(),
            },
            self.metadata.clone(),
            minter.into().address(),
        )
    }

    /// Creates a badge resource with fixed supply.
    pub fn new_badge_fixed<T: Into<Decimal>>(&self, supply: T) -> Bucket {
        ResourceDef::new_fixed(
            ResourceType::Fungible {
                granularity: 19.into(),
            },
            self.metadata.clone(),
            ResourceSupply::Fungible {
                amount: supply.into(),
            },
        )
        .1
    }

    /// Creates a NFT resource with mutable supply.
    pub fn new_nft_mutable<A: Into<ResourceDef>>(&self, minter: A) -> ResourceDef {
        ResourceDef::new_mutable(
            ResourceType::NonFungible,
            self.metadata.clone(),
            minter.into().address(),
        )
    }

    /// Creates a NFT resource with fixed supply.
    pub fn new_nft_fixed<V: Encode>(&self, supply: BTreeMap<u64, V>) -> Bucket {
        let mut encoded = BTreeMap::new();
        for (k, v) in supply {
            encoded.insert(k, scrypto_encode(&v));
        }

        ResourceDef::new_fixed(
            ResourceType::NonFungible,
            self.metadata.clone(),
            ResourceSupply::NonFungible { entries: encoded },
        )
        .1
    }
}

impl Default for ResourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
