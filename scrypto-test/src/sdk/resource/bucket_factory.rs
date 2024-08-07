use crate::prelude::*;

/// A factory for buckets that can create them (for testing) through multiple creation strategies
pub struct BucketFactory;

impl BucketFactory {
    pub fn create_fungible_bucket<S>(
        resource_address: ResourceAddress,
        amount: Decimal,
        creation_strategy: CreationStrategy,
        env: &mut TestEnvironment<S>,
    ) -> Result<FungibleBucket, RuntimeError>
    where
        S: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        let bucket = Self::create_bucket(
            FactoryResourceSpecifier::Amount(resource_address, amount),
            creation_strategy,
            env,
        )?;
        Ok(FungibleBucket(bucket))
    }

    pub fn create_non_fungible_bucket<I, D, S>(
        resource_address: ResourceAddress,
        non_fungibles: I,
        creation_strategy: CreationStrategy,
        env: &mut TestEnvironment<S>,
    ) -> Result<NonFungibleBucket, RuntimeError>
    where
        I: IntoIterator<Item = (NonFungibleLocalId, D)>,
        D: ScryptoEncode,
        S: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        let bucket = Self::create_bucket(
            FactoryResourceSpecifier::Ids(
                resource_address,
                non_fungibles
                    .into_iter()
                    .map(|(id, data)| {
                        (
                            id,
                            scrypto_decode::<ScryptoValue>(&scrypto_encode(&data).unwrap())
                                .unwrap(),
                        )
                    })
                    .collect(),
            ),
            creation_strategy,
            env,
        )?;
        Ok(NonFungibleBucket(bucket))
    }

    pub fn create_bucket<S>(
        resource_specifier: FactoryResourceSpecifier,
        creation_strategy: CreationStrategy,
        env: &mut TestEnvironment<S>,
    ) -> Result<Bucket, RuntimeError>
    where
        S: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        match (&resource_specifier, creation_strategy) {
            (
                FactoryResourceSpecifier::Amount(resource_address, amount),
                CreationStrategy::DisableAuthAndMint,
            ) => env.with_auth_module_disabled(|env| {
                let bucket = ResourceManager(*resource_address).mint_fungible(*amount, env)?;
                Ok(bucket.into())
            }),
            (
                FactoryResourceSpecifier::Ids(resource_address, ids),
                CreationStrategy::DisableAuthAndMint,
            ) => env.with_auth_module_disabled(|env| {
                let bucket = ResourceManager(*resource_address).mint_non_fungible(ids.clone(), env)?;
                Ok(bucket.into())
            }),
            (
                FactoryResourceSpecifier::Amount(resource_address, amount),
                CreationStrategy::Mock,
            ) => env.with_auth_module_disabled(|env| {
                assert!(Self::validate_resource_specifier(&resource_specifier, env)?);

                env.as_method_actor(
                    resource_address.into_node_id(),
                    ModuleId::Main,
                    FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                    |env| {
                        env.new_simple_object(
                            FUNGIBLE_BUCKET_BLUEPRINT,
                            indexmap!(
                                FungibleBucketField::Liquid.into() => FieldValue::new(LiquidFungibleResource::new(*amount)),
                                FungibleBucketField::Locked.into() => FieldValue::new(LockedFungibleResource::default()),
                            )
                        ).map(|node_id| Bucket(Own(node_id)))
                    },
                )?
            }),
            (
                FactoryResourceSpecifier::Ids(resource_address, non_fungibles),
                CreationStrategy::Mock,
            ) => env.with_auth_module_disabled(|env| {
                assert!(Self::validate_resource_specifier(&resource_specifier, env)?);

                env.as_method_actor(
                    resource_address.into_node_id(),
                    ModuleId::Main,
                    NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT,
                    |env| {
                        for (local_id, data) in non_fungibles.iter() {
                            let non_fungible_handle = env.actor_open_key_value_entry(
                                ACTOR_STATE_SELF,
                                NonFungibleResourceManagerCollection::DataKeyValue.collection_index(),
                                &local_id.to_key(),
                                LockFlags::MUTABLE,
                            )?;

                            let cur_non_fungible = env
                                .key_value_entry_get_typed::<NonFungibleResourceManagerDataEntryPayload>(
                                    non_fungible_handle,
                                )?;

                            if cur_non_fungible.is_some() {
                                return Err(RuntimeError::ApplicationError(
                                    ApplicationError::NonFungibleResourceManagerError(
                                        NonFungibleResourceManagerError::NonFungibleAlreadyExists(Box::new(
                                            NonFungibleGlobalId::new(*resource_address, local_id.clone()),
                                        )),
                                    ),
                                ));
                            }

                            env.key_value_entry_set_typed(
                                non_fungible_handle,
                                NonFungibleResourceManagerDataEntryPayload::from_content_source(data.clone()),
                            )?;
                            env.key_value_entry_close(non_fungible_handle)?;
                        }

                        env.new_simple_object(
                            NON_FUNGIBLE_BUCKET_BLUEPRINT,
                            indexmap!(
                                NonFungibleBucketField::Liquid.into() => FieldValue::new(LiquidNonFungibleResource::new(non_fungibles.keys().cloned().collect())),
                                NonFungibleBucketField::Locked.into() => FieldValue::new(LockedNonFungibleResource::default()),
                            )
                        ).map(|node_id| Bucket(Own(node_id)))
                    },
                )?
            }),
        }
    }

    fn validate_resource_specifier<S>(
        resource_specifier: &FactoryResourceSpecifier,
        env: &mut TestEnvironment<S>,
    ) -> Result<bool, RuntimeError>
    where
        S: SubstateDatabase + CommittableSubstateDatabase + 'static,
    {
        // Validating the resource is correct - can't mint IDs of a fungible resource and can't mint
        // an amount of a non-fungible resource.
        match resource_specifier {
            FactoryResourceSpecifier::Amount(resource_address, ..)
                if resource_address.is_fungible() =>
            {
                // No additional validations are needed for fungible resources
            }
            FactoryResourceSpecifier::Ids(resource_address, non_fungibles)
                if !resource_address.is_fungible() =>
            {
                // Some more validations are needed for non-fungibles.

                // Validate that the ids provided are:
                // 1. All of one type.
                // 2. This one type is the type of the non-fungible local ids.
                let id_type = {
                    let mut iter = non_fungibles
                        .keys()
                        .map(|id| id.id_type())
                        .collect::<IndexSet<NonFungibleIdType>>()
                        .into_iter();
                    let Some(id_type) = iter.next() else {
                        return Ok(true);
                    };
                    if iter.next().is_some() {
                        return Ok(false);
                    }
                    id_type
                };

                let ResourceType::NonFungible {
                    id_type: expected_id_type,
                } = ResourceManager(*resource_address).resource_type(env)?
                else {
                    return Ok(false);
                };

                if id_type != expected_id_type {
                    return Ok(false);
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_validate_resource_specifier_method() {
        // Arrange
        let mut env = TestEnvironment::new();
        let bucket = ResourceBuilder::new_integer_non_fungible::<()>(OwnerRole::None)
            .mint_initial_supply([], &mut env)
            .unwrap();

        let items = indexmap!(
            NonFungibleLocalId::integer(1u64) => SborValue::U8 { value: 1 },
            NonFungibleLocalId::integer(2u64) => SborValue::U8 { value: 2 },
        );

        let resource =
            FactoryResourceSpecifier::Ids(bucket.resource_address(&mut env).unwrap(), items);

        // Act
        let result = BucketFactory::validate_resource_specifier(&resource, &mut env).unwrap();

        // Assert
        assert!(result);
    }

    #[test]
    fn verify_validate_resource_specifier_method_should_fail() {
        // Arrange
        let mut env = TestEnvironment::new();
        let bucket = ResourceBuilder::new_integer_non_fungible::<()>(OwnerRole::None)
            .mint_initial_supply([], &mut env)
            .unwrap();

        let items = indexmap!(
            NonFungibleLocalId::integer(1u64) => SborValue::U8 { value: 1 },
            NonFungibleLocalId::string("value").unwrap() => SborValue::U8 { value: 2 },
        );

        let resource =
            FactoryResourceSpecifier::Ids(bucket.resource_address(&mut env).unwrap(), items);

        // Act
        let result = BucketFactory::validate_resource_specifier(&resource, &mut env).unwrap();

        // Assert
        assert!(!result);
    }
}
