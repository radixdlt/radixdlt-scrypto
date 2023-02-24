use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_MANAGER_PACKAGE;
use radix_engine_interface::data::model::Own;
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode};
use radix_engine_interface::math::Decimal;
use sbor::rust::collections::BTreeMap;
use sbor::rust::fmt::Debug;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;

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
                RESOURCE_MANAGER_BLUEPRINT,
                RESOURCE_MANAGER_CREATE_FUNGIBLE_IDENT,
                scrypto_encode(&ResourceManagerCreateFungibleInput {
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
                RESOURCE_MANAGER_BLUEPRINT,
                RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT,
                scrypto_encode(&ResourceManagerCreateFungibleWithInitialSupplyInput {
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

    pub fn new_non_fungible<Y, E: Debug + ScryptoDecode>(
        id_type: NonFungibleIdType,
        metadata: BTreeMap<String, String>,
        access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientApi<E>,
    {
        let result = api.call_function(
            RESOURCE_MANAGER_PACKAGE,
            RESOURCE_MANAGER_BLUEPRINT,
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_IDENT,
            scrypto_encode(&ResourceManagerCreateNonFungibleInput {
                id_type,
                metadata,
                access_rules,
            })
            .unwrap(),
        )?;
        let resource_address = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<Y, E: Debug + ScryptoDecode>(
        &mut self,
        local_id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let mut entries = BTreeMap::new();
        entries.insert(
            local_id,
            (scrypto_encode(&()).unwrap(), scrypto_encode(&()).unwrap()),
        );

        let rtn = api.call_method(
            RENodeId::Global(self.0.into()),
            RESOURCE_MANAGER_MINT_NON_FUNGIBLE,
            scrypto_encode(&ResourceManagerMintNonFungibleInput { entries }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible_uuid<Y, E: Debug + ScryptoDecode, T: ScryptoEncode>(
        &mut self,
        data: T,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        // TODO: Implement UUID generation in ResourceManager
        let mut entries = Vec::new();
        entries.push((scrypto_encode(&data).unwrap(), scrypto_encode(&()).unwrap()));

        let rtn = api.call_method(
            RENodeId::Global(self.0.into()),
            RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE,
            scrypto_encode(&ResourceManagerMintUuidNonFungibleInput { entries }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Mints non-fungible resources
    pub fn mint_fungible<Y, E: Debug + ScryptoDecode>(
        &mut self,
        amount: Decimal,
        api: &mut Y,
    ) -> Result<Bucket, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Global(self.0.into()),
            RESOURCE_MANAGER_MINT_FUNGIBLE,
            scrypto_encode(&ResourceManagerMintFungibleInput { amount }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn get_non_fungible_mutable_data<Y, E: Debug + ScryptoDecode, T: ScryptoDecode>(
        &self,
        id: NonFungibleLocalId,
        api: &mut Y,
    ) -> Result<T, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Global(self.0.into()),
            RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT,
            scrypto_encode(&ResourceManagerGetNonFungibleInput { id }).unwrap(),
        )?;

        let output: [Vec<u8>; 2] = scrypto_decode(&rtn).unwrap();
        let data = scrypto_decode(&output[0]).unwrap();
        Ok(data)
    }

    pub fn resource_type<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<ResourceType, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Global(self.0.into()),
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
            RENodeId::Global(self.0.into()),
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
            RENodeId::Global(self.0.into()),
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
            RENodeId::Global(self.0.into()),
            RESOURCE_MANAGER_CREATE_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerCreateBucketInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn new_vault<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Own, E>
    where
        Y: ClientApi<E>,
    {
        let rtn = api.call_method(
            RENodeId::Global(self.0.into()),
            RESOURCE_MANAGER_CREATE_VAULT_IDENT,
            scrypto_encode(&ResourceManagerCreateVaultInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
