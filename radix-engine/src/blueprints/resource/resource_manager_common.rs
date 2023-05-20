use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;
use radix_engine_interface::api::node_modules::metadata::METADATA_SET_IDENT;

fn build_access_rules(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> (
    (AuthorityRules, BTreeMap<MethodKey, Vec<String>>),
    (AuthorityRules, BTreeMap<MethodKey, Vec<String>>),
    (AuthorityRules, BTreeMap<MethodKey, Vec<String>>),
) {
    let (resman_authority_rules, resman_protected_methods) = {
        let (mint_access_rule, mint_mutability) = access_rules_map
            .remove(&Mint)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (burn_access_rule, burn_mutability) = access_rules_map
            .remove(&Burn)
            .unwrap_or((DenyAll, rule!(deny_all)));
        let (update_non_fungible_data_access_rule, update_non_fungible_data_mutability) =
            access_rules_map
                .remove(&UpdateNonFungibleData)
                .unwrap_or((AllowAll, rule!(deny_all)));
        let (update_metadata_access_rule, update_metadata_mutability) = access_rules_map
            .remove(&UpdateMetadata)
            .unwrap_or((DenyAll, rule!(deny_all)));

        let mut resman_authority_rules = AuthorityRules::new();

        {
            resman_authority_rules.define_role(
                UPDATE_METADATA_AUTHORITY,
                update_metadata_access_rule,
                update_metadata_mutability,
            );
        }

        // Mint
        {
            resman_authority_rules.define_role(
                MINT_AUTHORITY,
                mint_access_rule,
                mint_mutability,
            );
        }

        // Burn
        {
            resman_authority_rules.define_role(
                BURN_AUTHORITY,
                burn_access_rule,
                burn_mutability,
            );
        }

        // Non Fungible Update data
        {
            resman_authority_rules.define_role(
                UPDATE_NON_FUNGIBLE_DATA_AUTHORITY,
                update_non_fungible_data_access_rule,
                update_non_fungible_data_mutability,
            );
        }



        let resman_protected_methods = btreemap!(
            MethodKey::metadata(METADATA_SET_IDENT) => vec![UPDATE_METADATA_AUTHORITY.to_string()],
            MethodKey::main(FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT) => vec![MINT_AUTHORITY.to_string()],
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT) => vec![MINT_AUTHORITY.to_string()],
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT) => vec![MINT_AUTHORITY.to_string()],
            MethodKey::main(RESOURCE_MANAGER_BURN_IDENT) => vec![BURN_AUTHORITY.to_string()],
            MethodKey::main(NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT) => vec![UPDATE_NON_FUNGIBLE_DATA_AUTHORITY.to_string()],
        );

        (resman_authority_rules, resman_protected_methods)
    };

    let (vault_authority_rules, vault_protected_methods) = {
        let (deposit_access_rule, deposit_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Deposit)
            .unwrap_or((AllowAll, rule!(deny_all)));
        let (withdraw_access_rule, withdraw_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Withdraw)
            .unwrap_or((AllowAll, rule!(deny_all)));
        let (recall_access_rule, recall_mutability) = access_rules_map
            .remove(&ResourceMethodAuthKey::Recall)
            .unwrap_or((DenyAll, rule!(deny_all)));

        let mut vault_authority_rules = AuthorityRules::new();

        // Withdraw
        {
            vault_authority_rules.define_role(
                WITHDRAW_AUTHORITY,
                withdraw_access_rule,
                withdraw_mutability,
            );
        }

        // Recall
        {
            vault_authority_rules.define_role(
                RECALL_AUTHORITY,
                recall_access_rule,
                recall_mutability,
            );
        }

        // Deposit
        {
            vault_authority_rules.define_role(
                DEPOSIT_AUTHORITY,
                deposit_access_rule,
                deposit_mutability,
            );
        }

        // Internal
        {
            vault_authority_rules.set_fixed_authority_rule(
                "this_package",
                rule!(require(package_of_direct_caller(RESOURCE_PACKAGE))),
            );
        }

        let vault_protected_methods = btreemap!(
            MethodKey::main(VAULT_TAKE_IDENT) => vec![WITHDRAW_AUTHORITY.to_string()],
            MethodKey::main(FUNGIBLE_VAULT_LOCK_FEE_IDENT) => vec![WITHDRAW_AUTHORITY.to_string()],
            MethodKey::main(NON_FUNGIBLE_VAULT_TAKE_NON_FUNGIBLES_IDENT) => vec![WITHDRAW_AUTHORITY.to_string()],
            MethodKey::main(VAULT_RECALL_IDENT) => vec![RECALL_AUTHORITY.to_string()],
            MethodKey::main(NON_FUNGIBLE_VAULT_RECALL_NON_FUNGIBLES_IDENT) => vec![RECALL_AUTHORITY.to_string()],
            MethodKey::main(VAULT_PUT_IDENT) => vec![DEPOSIT_AUTHORITY.to_string()],
            MethodKey::main(FUNGIBLE_VAULT_LOCK_FUNGIBLE_AMOUNT_IDENT) => vec!["this_package".to_string()],
            MethodKey::main(NON_FUNGIBLE_VAULT_LOCK_NON_FUNGIBLES_IDENT) => vec!["this_package".to_string()],
            MethodKey::main(FUNGIBLE_VAULT_UNLOCK_FUNGIBLE_AMOUNT_IDENT) => vec!["this_package".to_string()],
            MethodKey::main(NON_FUNGIBLE_VAULT_UNLOCK_NON_FUNGIBLES_IDENT) => vec!["this_package".to_string()],
        );

        (vault_authority_rules, vault_protected_methods)
    };

    // Note that if a local reference to a bucket is passed to another actor, the recipient will be able
    // to take resource from the bucket. This is not what Scrypto lib supports/encourages, but can be done
    // theoretically.

    let (bucket_authority_rules, bucket_protected_methods) = {
        let mut bucket_authority_rules = AuthorityRules::new();
        bucket_authority_rules.set_fixed_authority_rule(
            "this_package",
            rule!(require(package_of_direct_caller(RESOURCE_PACKAGE))),
        );

        let bucket_protected_methods = btreemap!(
            MethodKey::main(FUNGIBLE_BUCKET_LOCK_AMOUNT_IDENT) => vec!["this_package".to_string()],
            MethodKey::main(FUNGIBLE_BUCKET_UNLOCK_AMOUNT_IDENT) => vec!["this_package".to_string()],
            MethodKey::main(NON_FUNGIBLE_BUCKET_LOCK_NON_FUNGIBLES_IDENT) => vec!["this_package".to_string()],
            MethodKey::main(NON_FUNGIBLE_BUCKET_UNLOCK_NON_FUNGIBLES_IDENT) => vec!["this_package".to_string()],
        );

        (bucket_authority_rules, bucket_protected_methods)
    };

    (
        (resman_authority_rules, resman_protected_methods),
        (vault_authority_rules, vault_protected_methods),
        (bucket_authority_rules, bucket_protected_methods),
    )
}

pub fn globalize_resource_manager<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, String>,
    api: &mut Y,
) -> Result<(), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (
        (resman_roles, protected_resman_methods),
        (vault_authorities, protected_vault_methods),
        (bucket_authorities, protected_bucket_methods)
    ) =
        build_access_rules(access_rules);

    let (vault_blueprint_name, bucket_blueprint_name, proof_blueprint_name) =
        if resource_address.as_node_id().is_global_fungible_resource() {
            (
                FUNGIBLE_VAULT_BLUEPRINT,
                FUNGIBLE_BUCKET_BLUEPRINT,
                FUNGIBLE_PROOF_BLUEPRINT,
            )
        } else {
            (
                NON_FUNGIBLE_VAULT_BLUEPRINT,
                NON_FUNGIBLE_BUCKET_BLUEPRINT,
                NON_FUNGIBLE_PROOF_BLUEPRINT,
            )
        };

    let resman_access_rules = AccessRules::create(
        protected_resman_methods,
        resman_roles,
        btreemap!(
            vault_blueprint_name.to_string() => (vault_authorities, protected_vault_methods),
            bucket_blueprint_name.to_string() => (bucket_authorities, protected_bucket_methods),
            proof_blueprint_name.to_string() => (AuthorityRules::new(), btreemap!()),
        ),
        api,
    )?
    .0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    api.globalize_with_address(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
    )?;

    Ok(())
}

pub fn globalize_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, String>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (
        (resman_roles, protected_resman_methods),
        (vault_authorities, protected_vault_methods),
        (bucket_authorities, protected_bucket_methods)
    ) =
        build_access_rules(access_rules);

    let resman_access_rules = AccessRules::create(
        protected_resman_methods,
        resman_roles,
        btreemap!(
            FUNGIBLE_VAULT_BLUEPRINT.to_string() => (vault_authorities, protected_vault_methods),
            FUNGIBLE_BUCKET_BLUEPRINT.to_string() => (bucket_authorities, protected_bucket_methods),
            FUNGIBLE_PROOF_BLUEPRINT.to_string() => (AuthorityRules::new(), btreemap!()),
        ),
        api,
    )?
    .0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    let bucket_id = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
        FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidFungibleResource::new(initial_supply)).unwrap(),
            scrypto_encode(&LockedFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok(Bucket(Own(bucket_id)))
}

pub fn globalize_non_fungible_with_initial_supply<Y>(
    object_id: NodeId,
    resource_address: ResourceAddress,
    access_rules: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
    metadata: BTreeMap<String, String>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<Bucket, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let (
        (resman_roles, protected_resman_methods),
        (vault_authorities, protected_vault_methods),
        (bucket_authorities, protected_bucket_methods)
    ) =
        build_access_rules(access_rules);

    let resman_access_rules = AccessRules::create(
        protected_resman_methods,
        resman_roles,
        btreemap!(
            NON_FUNGIBLE_VAULT_BLUEPRINT.to_string() => (vault_authorities, protected_vault_methods),
            NON_FUNGIBLE_BUCKET_BLUEPRINT.to_string()=> (bucket_authorities, protected_bucket_methods),
            NON_FUNGIBLE_PROOF_BLUEPRINT.to_string() => (AuthorityRules::new(), btreemap!()),
        ),
        api,
    )?
    .0;

    let metadata = Metadata::create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::create(RoyaltyConfig::default(), api)?;

    let bucket_id = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
            ObjectModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
        NON_FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
            scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok(Bucket(Own(bucket_id)))
}
