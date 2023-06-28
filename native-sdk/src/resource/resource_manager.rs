use radix_engine_derive::ScryptoSbor;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::{ClientBlueprintApi, ClientObjectApi};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::constants::RESOURCE_PACKAGE;
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::data::scrypto::{
    scrypto_decode, scrypto_encode, ScryptoDecode, ScryptoEncode, ScryptoValue,
};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::types::NonFungibleData;
use radix_engine_interface::types::*;
use sbor::rust::prelude::*;

/// Represents a resource manager.
#[derive(Debug, PartialEq, Eq, ScryptoSbor, Clone)]
pub struct ResourceManager(pub ResourceAddress);

impl ResourceManager {
    pub fn new_fungible<Y, E: Debug + ScryptoDecode, M: Into<MetadataInit>>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
        metadata: M,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientBlueprintApi<E>,
    {
        let metadata = ModuleConfig {
            init: metadata.into(),
            roles: RolesInit::default(),
        };

        let result = api.call_function(
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            scrypto_encode(&FungibleResourceManagerCreateInput {
                owner_role,
                track_total_supply,
                metadata,
                access_rules,
                divisibility,
                address_reservation,
            })
            .unwrap(),
        )?;

        let resource_address = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    pub fn new_fungible_with_initial_supply<Y, E: Debug + ScryptoDecode, M: Into<MetadataInit>>(
        owner_role: OwnerRole,
        track_total_supply: bool,
        divisibility: u8,
        initial_supply: Decimal,
        access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
        metadata: M,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<(Self, Bucket), E>
    where
        Y: ClientBlueprintApi<E>,
    {
        let metadata = ModuleConfig {
            init: metadata.into(),
            roles: RolesInit::default(),
        };

        let result = api.call_function(
            RESOURCE_PACKAGE,
            FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT,
            scrypto_encode(&FungibleResourceManagerCreateWithInitialSupplyInput {
                owner_role,
                track_total_supply,
                metadata,
                access_rules,
                divisibility,
                initial_supply,
                address_reservation,
            })
            .unwrap(),
        )?;
        let (resource_address, bucket): (ResourceAddress, Bucket) =
            scrypto_decode(result.as_slice()).unwrap();
        Ok((ResourceManager(resource_address), bucket))
    }

    pub fn new_non_fungible<
        N: NonFungibleData,
        Y,
        E: Debug + ScryptoDecode,
        M: Into<MetadataInit>,
    >(
        owner_role: OwnerRole,
        id_type: NonFungibleIdType,
        track_total_supply: bool,
        access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
        metadata: M,
        address_reservation: Option<GlobalAddressReservation>,
        api: &mut Y,
    ) -> Result<Self, E>
    where
        Y: ClientBlueprintApi<E>,
    {
        let metadata = ModuleConfig {
            init: metadata.into(),
            roles: RolesInit::default(),
        };

        let non_fungible_schema = NonFungibleDataSchema::new_schema::<N>();
        let result = api.call_function(
            RESOURCE_PACKAGE,
            NON_FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT,
            NON_FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT,
            scrypto_encode(&NonFungibleResourceManagerCreateInput {
                owner_role,
                id_type,
                track_total_supply,
                non_fungible_schema,
                metadata,
                access_rules,
                address_reservation,
            })
            .unwrap(),
        )?;
        let resource_address = scrypto_decode(result.as_slice()).unwrap();
        Ok(ResourceManager(resource_address))
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible_single_ruid<Y, E: Debug + ScryptoDecode, T: ScryptoEncode>(
        &self,
        data: T,
        api: &mut Y,
    ) -> Result<(Bucket, NonFungibleLocalId), E>
    where
        Y: ClientObjectApi<E>,
    {
        let value: ScryptoValue = scrypto_decode(&scrypto_encode(&data).unwrap()).unwrap();

        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_RUID_IDENT,
            scrypto_encode(&NonFungibleResourceManagerMintSingleRuidInput { entry: value })
                .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Mints non-fungible resources
    pub fn mint_non_fungible<Y, E: Debug + ScryptoDecode, T: ScryptoEncode>(
        &self,
        data: BTreeMap<NonFungibleLocalId, T>,
        api: &mut Y,
    ) -> Result<NonFungibleResourceManagerMintOutput, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
            scrypto_encode(&NonFungibleResourceManagerMintInput {
                entries: data
                    .into_iter()
                    .map(|(key, value)| {
                        (
                            key,
                            (scrypto_decode(&scrypto_encode(&value).unwrap()).unwrap(),),
                        )
                    })
                    .collect(),
            })
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
        Y: ClientObjectApi<E>,
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
        Y: ClientObjectApi<E>,
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
        Y: ClientObjectApi<E>,
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
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_BURN_IDENT,
            scrypto_encode(&ResourceManagerBurnInput { bucket }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn package_burn<Y, E: Debug + ScryptoDecode>(
        &mut self,
        bucket: Bucket,
        api: &mut Y,
    ) -> Result<(), E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_PACKAGE_BURN_IDENT,
            scrypto_encode(&ResourceManagerPackageBurnInput { bucket }).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn total_supply<Y, E: Debug + ScryptoDecode>(
        &self,
        api: &mut Y,
    ) -> Result<Option<Decimal>, E>
    where
        Y: ClientObjectApi<E>,
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
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyBucketInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn new_empty_vault<Y, E: Debug + ScryptoDecode>(&self, api: &mut Y) -> Result<Own, E>
    where
        Y: ClientObjectApi<E>,
    {
        let rtn = api.call_method(
            self.0.as_node_id(),
            RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT,
            scrypto_encode(&ResourceManagerCreateEmptyVaultInput {}).unwrap(),
        )?;
        Ok(scrypto_decode(&rtn).unwrap())
    }
}
