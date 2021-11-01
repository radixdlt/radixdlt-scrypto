use crate::resource::*;
use crate::rust::borrow::ToOwned;
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

    /// Creates a resource with mutable supply.
    pub fn create_mutable<A: Into<ResourceDef>>(&self, mint_burn_auth: A) -> ResourceDef {
        ResourceDef::new_mutable(self.metadata.clone(), mint_burn_auth)
    }

    /// Creates a resource with fixed supply.
    pub fn create_fixed<T: Into<Amount>>(&self, supply: T) -> Bucket {
        ResourceDef::new_fixed(self.metadata.clone(), supply.into()).1
    }
}

impl Default for ResourceBuilder {
    fn default() -> Self {
        Self::new()
    }
}
