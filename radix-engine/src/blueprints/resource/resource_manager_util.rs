use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRulesObject;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::node_modules::metadata::{METADATA_GET_IDENT, METADATA_SET_IDENT};
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_access_rules(
    mut access_rules_map: BTreeMap<ResourceMethodAuthKey, (AccessRule, AccessRule)>,
) -> (AccessRulesConfig, AccessRulesConfig) {
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

    let mut resman_access_rules = AccessRulesConfig::new();
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::Metadata, METADATA_SET_IDENT.to_string()),
        update_metadata_access_rule,
        update_metadata_mutability,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::Metadata, METADATA_GET_IDENT.to_string()),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_group_access_rule_and_mutability(
        "mint".to_string(),
        mint_access_rule,
        mint_mutability,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
        ),
        "mint".to_string(),
        DenyAll,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_UUID_IDENT.to_string(),
        ),
        "mint".to_string(),
        DenyAll,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_MINT_SINGLE_UUID_IDENT.to_string(),
        ),
        "mint".to_string(),
        DenyAll,
    );
    resman_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT.to_string(),
        ),
        "mint".to_string(),
        DenyAll,
    );

    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_BURN_IDENT.to_string(),
        ),
        burn_access_rule,
        burn_mutability,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_UPDATE_DATA_IDENT.to_string(),
        ),
        update_non_fungible_data_access_rule,
        update_non_fungible_data_mutability,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_CREATE_BUCKET_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            RESOURCE_MANAGER_CREATE_VAULT_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_EXISTS_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    resman_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            NON_FUNGIBLE_RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );

    let (deposit_access_rule, deposit_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Deposit)
        .unwrap_or((AllowAll, rule!(deny_all)));
    let (withdraw_access_rule, withdraw_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Withdraw)
        .unwrap_or((AllowAll, rule!(deny_all)));
    let (recall_access_rule, recall_mutability) = access_rules_map
        .remove(&ResourceMethodAuthKey::Recall)
        .unwrap_or((DenyAll, rule!(deny_all)));

    let mut vault_access_rules = AccessRulesConfig::new();
    vault_access_rules.set_group_access_rule_and_mutability(
        "withdraw".to_string(),
        withdraw_access_rule,
        withdraw_mutability,
    );
    vault_access_rules.set_group_access_rule_and_mutability(
        "recall".to_string(),
        recall_access_rule,
        recall_mutability,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, VAULT_TAKE_IDENT.to_string()),
        "withdraw".to_string(),
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_TAKE_NON_FUNGIBLES_IDENT.to_string(),
        ),
        "withdraw".to_string(),
        DenyAll,
    );
    vault_access_rules.set_group_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, VAULT_LOCK_FEE_IDENT.to_string()),
        "withdraw".to_string(),
        DenyAll,
    );

    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(TypedModuleId::ObjectState, VAULT_PUT_IDENT.to_string()),
        deposit_access_rule,
        deposit_mutability,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_GET_AMOUNT_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_GET_RESOURCE_ADDRESS_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_GET_NON_FUNGIBLE_LOCAL_IDS_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_CREATE_PROOF_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_CREATE_PROOF_BY_AMOUNT_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_CREATE_PROOF_BY_IDS_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_LOCK_AMOUNT_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_LOCK_NON_FUNGIBLES_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_UNLOCK_AMOUNT_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );
    vault_access_rules.set_access_rule_and_mutability(
        MethodKey::new(
            TypedModuleId::ObjectState,
            VAULT_UNLOCK_NON_FUNGIBLES_IDENT.to_string(),
        ),
        AllowAll,
        DenyAll,
    );

    (resman_access_rules, vault_access_rules)
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
    let (resman_access_rules, vault_access_rules) = build_access_rules(access_rules);
    let resman_access_rules = AccessRulesObject::sys_new(resman_access_rules, api)?;
    let vault_access_rules = AccessRulesObject::sys_new(vault_access_rules, api)?;
    let metadata = Metadata::sys_create_with_data(metadata, api)?;
    let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

    api.globalize_with_address(
        NodeId::Object(object_id),
        btreemap!(
            TypedModuleId::AccessRules => resman_access_rules.id(),
            TypedModuleId::AccessRules1 => vault_access_rules.id(),
            TypedModuleId::Metadata => metadata.0,
            TypedModuleId::Royalty => royalty.0,
        ),
        resource_address.into(),
    )?;

    Ok(())
}
