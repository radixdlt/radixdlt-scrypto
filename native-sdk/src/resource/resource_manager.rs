use radix_engine_interface::api::{ClientApi, ClientObjectApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_MANAGER_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode, ScryptoValue,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::NonFungibleData;
use radix_engine_interface::types::*;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;

/// Represents a resource manager.
#[derive(Debug)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn new_fungible<Y, E: Debug + ScryptoDecode>(
        divisibility: u8,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let result = api
            .call_function(
                RESOURCE_MANAGER_PACKAGE,
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
                scrypto_encode(&FungibleResourceManagerCreateInput {
                    metadata,
                    access_rules,
                    divisibility,
                })
                .unwrap(),
            )
            .unwrap();
        let resource_address = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    pub fn new_fungible_with_initial_supply<Y, E: Debug + ScryptoDecode>(
        divisibility: u8,
        amount: Decimal,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<(Self, Bucket), E>
    where
        Y: ClientApi<E>,
    {
        let result = api
            .call_function(
                RESOURCE_MANAGER_PACKAGE,
                FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
                FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&FungibleResourceManagerCreateWithInitialSupplyInput {
                    metadata,
                    access_rules,
                    divisibility,
                    initial_supply: amount,
                })
                .unwrap(),
            )
            .unwrap();
        let (resource_address, bucket): (ResourceAddress, Bucket) =
            scrypto_decode(result.as_slice()).unwrap();
        Ok((ResourceManager(resource_address), bucket))
    }

    pub fn new_non_fungible<N: NonFungibleData, Y, E: Debug + ScryptoDecode>(
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let non_fungible_schema = NonFungibleDataSchema::new_schema::<N>();
        let result = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            scrypto_encode(&NonFungibleResourceManagerCreateInput {
                id_type,
                non_fungible_schema,
                metadata,
                access_rules,
            })
            .unwrap(),
        )?;
        let resource_address = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    pub fn new_non_fungible_with_address<N: NonFungibleData, Y, E: Debug + ScryptoDecode>(
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        address: [u8; 27], // TODO: Clean this up
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let result = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_ADDRESS_IDENT,
            scrypto_encode(&NonFungibleResourceManagerCreateWithAddressInput {
                id_type,
                non_fungible_schema: NonFungibleDataSchema::new_schema::<N>(),
                metadata,
                access_rules,
                resource_address: address,
            })
            .unwrap(),
        )?;
        let resource_address: ResourceAddress = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible_single_uuid<Y, E: Debug + ScryptoDecode, T: ScryptoEncode>(
        &self,
        data: T,
        api: &mut Y,
    ) -> Result<(Bucket, NonFungibleLocalId), E>
    where
        Y: ClientApi<E>,
    {
        let value: ScryptoValue = scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();

        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT,
            scrypto_encode(&NonFungibleResourceManagerMintSingleUuidInput { entry: value })
                .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Mints fungible resources
    pub fn mint_fungible<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            scrypto_encode(&FungibleResourceManagerMintInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn get_non_fungible_data<Y, E: Debug + ScryptoDecode, T: ScryptoDecode>(
        &self,
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<T, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
            scrypto_encode(&NonFungibleResourceManagerGetNonFungibleInput { id }).unwrap(),
        )?;

        let data = scrypto_decode(&rtn).unwrap();
        Ok(data)
    }

    pub fn resource_type<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<ResourceType, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT,
            scrypto_encode(&ResourceManagerGetResourceTypeInput {}).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn burn<Y, E: Debug + ScryptoDecode>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_BURN_IDENT,
            scrypto_encode(&ResourceManagerBurnInput { bucket }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn total_supply<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Decimal, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT,
            scrypto_encode(&ResourceManagerGetTotalSupplyInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn new_empty_bucket<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_CREATE_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerCreateBucketInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn new_vault<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Own, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_CREATE_VAULT_IDENT,
            scrypto_encode(&ResourceManagerCreateVaultInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
