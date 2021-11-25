use sbor::{describe::Type, *};

use crate::buffer::*;
use crate::kernel::*;
use crate::resource::*;
use crate::rust::borrow::ToOwned;
use crate::rust::collections::HashMap;
use crate::rust::string::String;
use crate::rust::vec;
use crate::types::*;
use crate::utils::*;

/// Represents the definition of a resource.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDef {
    address: Address,
}

impl From<Address> for ResourceDef {
    fn from(address: Address) -> Self {
        if !address.is_resource_def() {
            panic!("{} is not a resource definition address", address);
        }

        Self { address }
    }
}

impl From<ResourceDef> for Address {
    fn from(a: ResourceDef) -> Address {
        a.address
    }
}

impl ResourceDef {
    /// Creates a resource with fixed supply. The created resource is immediately returned.
    pub fn new(
        resource_type: ResourceType,
        metadata: HashMap<String, String>,
        initial_supply: ResourceSupply,
        configs: ResourceConfigs,
    ) -> Bucket {
        let input = CreateResourceInput {
            resource_type,
            metadata,
            initial_supply,
            configs,
        };
        let output: CreateResourceOutput = call_kernel(CREATE_RESOURCE, input);

        output.bucket.into()
    }

    /// Mints fungible resources
    pub fn mint<T: Into<Decimal>>(&self, amount: T, auth: BucketRef) -> Bucket {
        let input = MintResourceInput {
            resource_def: self.address,
            new_supply: ResourceSupply::Fungible {
                amount: amount.into(),
            },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    /// Mints non-fungible resources
    pub fn mint_nft<T: Encode>(&self, id: u128, data: T, auth: BucketRef) -> Bucket {
        let mut entries = HashMap::new();
        entries.insert(id, scrypto_encode(&data));

        let input = MintResourceInput {
            resource_def: self.address,
            new_supply: ResourceSupply::NonFungible { entries },
            auth: auth.into(),
        };
        let output: MintResourceOutput = call_kernel(MINT_RESOURCE, input);

        output.bucket.into()
    }

    /// Burns a bucket of resources.
    pub fn burn(&self, bucket: Bucket, auth: BucketRef) {
        let input = BurnResourceInput {
            bucket: bucket.into(),
            auth: auth.into(),
        };
        let _output: BurnResourceOutput = call_kernel(BURN_RESOURCE, input);
    }

    /// Returns the metadata associated with this resource.
    pub fn metadata(&self) -> HashMap<String, String> {
        let input = GetResourceMetadataInput {
            resource_def: self.address,
        };
        let output: GetResourceMetadataOutput = call_kernel(GET_RESOURCE_METADATA, input);

        output.metadata
    }

    /// Returns the authorization configurations.
    pub fn auth_configs(&self) -> Option<ResourceConfigs> {
        let input = GetResourceConfigsInput {
            resource_def: self.address,
        };
        let output: GetResourceConfigsOutput = call_kernel(GET_RESOURCE_AUTH_CONFIGS, input);

        output.auth_configs
    }

    /// Returns the resource type.
    pub fn resource_type(&self) -> ResourceType {
        let input = GetResourceTypeInput {
            resource_def: self.address,
        };
        let output: GetResourceTypeOutput = call_kernel(GET_RESOURCE_AUTH_CONFIGS, input);

        output.resource_type
    }

    /// Returns the current supply of this resource.
    #[deprecated(note = "Please use `total_supply()` instead")]
    pub fn supply(&self) -> Decimal {
        self.total_supply()
    }

    /// Returns the current supply of this resource.
    pub fn total_supply(&self) -> Decimal {
        let input = GetResourceTotalSupplyInput {
            resource_def: self.address,
        };
        let output: GetResourceTotalSupplyOutput = call_kernel(GET_RESOURCE_TOTAL_SUPPLY, input);

        output.supply
    }

    /// Returns the address of this resource.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Gets the data of an NFT.
    ///
    /// # Panics
    /// Panics if this is not an NFT resource or the specified NFT is not found.
    pub fn get_nft_data<T: Decode>(&self, id: u128) -> T {
        let input = GetNftDataInput {
            resource_def: self.address,
            id,
        };
        let output: GetNftDataOutput = call_kernel(GET_NFT_DATA, input);

        scrypto_unwrap(scrypto_decode(&output.data))
    }

    /// Updates the data of an NFT.
    ///
    /// # Panics
    /// Panics if this is not an NFT resource or the specified NFT is not found.
    pub fn update_nft_data<T: Encode>(&self, id: u128, data: T, auth: BucketRef) {
        let input = UpdateNftDataInput {
            resource_def: self.address,
            id,
            data: scrypto_encode(&data),
            auth: auth.into(),
        };
        let _: UpdateNftDataOutput = call_kernel(UPDATE_NFT_DATA, input);
    }

    /// Changes this resource to immutable.
    ///
    /// #Panics
    /// Panics if this resource is not mutable
    pub fn change_to_immutable(&self, auth: BucketRef) {
        let input = ChangeToImmutableInput {
            resource_def: self.address,
            auth: auth.into(),
        };
        let _: ChangeToImmutableOutput = call_kernel(CHANGE_TO_IMMUTABLE, input);
    }
}

//========
// SBOR
//========

impl TypeId for ResourceDef {
    fn type_id() -> u8 {
        Address::type_id()
    }
}

impl Encode for ResourceDef {
    fn encode_value(&self, encoder: &mut Encoder) {
        self.address.encode_value(encoder);
    }
}

impl Decode for ResourceDef {
    fn decode_value(decoder: &mut Decoder) -> Result<Self, DecodeError> {
        Address::decode_value(decoder).map(Into::into)
    }
}

impl Describe for ResourceDef {
    fn describe() -> Type {
        Type::Custom {
            name: SCRYPTO_NAME_RESOURCE_DEF.to_owned(),
            generics: vec![],
        }
    }
}
