use crate::errors::RuntimeError;
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::resource::AccessRule::{AllowAll, DenyAll};
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::*;

fn build_main_access_rules(
    mut access_rules_map: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
) -> RolesInit {
    let mut main_roles = RolesInit::new();

    // Meta roles
    {
        main_roles.define_immutable_role(
            RESOURCE_PACKAGE_ROLE,
            rule!(require(package_of_direct_caller(RESOURCE_PACKAGE))),
        );
    }

    // Main roles
    {
        // Mint
        {
            let (mint_access_rule, mint_mutability) = access_rules_map
                .remove(&ResourceAction::Mint)
                .unwrap_or((DenyAll, DenyAll));
            let locked = mint_mutability.eq(&DenyAll);
            main_roles.define_role(MINTER_UPDATER_ROLE, mint_mutability, locked);
            main_roles.define_role(MINTER_ROLE, mint_access_rule, locked);
        }

        // Burn
        {
            let (burn_access_rule, burn_mutability) = access_rules_map
                .remove(&ResourceAction::Burn)
                .unwrap_or((DenyAll, DenyAll));
            let locked = burn_mutability.eq(&DenyAll);
            main_roles.define_role(BURNER_UPDATER_ROLE, burn_mutability, locked);
            main_roles.define_role(BURNER_ROLE, burn_access_rule, locked);
        }

        // Non Fungible Update data
        {
            let (update_non_fungible_data_access_rule, update_non_fungible_data_mutability) =
                access_rules_map
                    .remove(&ResourceAction::UpdateNonFungibleData)
                    .unwrap_or((AllowAll, DenyAll));
            let locked = update_non_fungible_data_mutability.eq(&DenyAll);
            main_roles.define_role(
                NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE,
                update_non_fungible_data_mutability,
                locked,
            );
            main_roles.define_role(
                NON_FUNGIBLE_DATA_UPDATER_ROLE,
                update_non_fungible_data_access_rule,
                locked,
            );
        }

        // Withdraw
        {
            let (withdraw_access_rule, withdraw_mutability) = access_rules_map
                .remove(&ResourceAction::Withdraw)
                .unwrap_or((AllowAll, DenyAll));
            let locked = withdraw_mutability.eq(&DenyAll);
            main_roles.define_role(WITHDRAWER_ROLE, withdraw_access_rule, locked);
            main_roles.define_role(WITHDRAWER_UPDATER_ROLE, withdraw_mutability, locked);
        }

        // Recall
        {
            let (recall_access_rule, recall_mutability) = access_rules_map
                .remove(&ResourceAction::Recall)
                .unwrap_or((DenyAll, DenyAll));
            let locked = recall_mutability.eq(&DenyAll);
            main_roles.define_role(RECALLER_ROLE, recall_access_rule, locked);
            main_roles.define_role(RECALLER_UPDATER_ROLE, recall_mutability, locked);
        }

        // Freeze/Unfreeze Role
        {
            let (freeze_access_rule, freeze_mutability) = access_rules_map
                .remove(&ResourceAction::Freeze)
                .unwrap_or((DenyAll, DenyAll));
            let locked = freeze_mutability.eq(&DenyAll);
            main_roles.define_role(FREEZER_ROLE, freeze_access_rule, locked);
            main_roles.define_role(FREEZER_UPDATER_ROLE, freeze_mutability, locked);
        }

        // Deposit
        {
            let (deposit_access_rule, deposit_mutability) = access_rules_map
                .remove(&ResourceAction::Deposit)
                .unwrap_or((AllowAll, DenyAll));
            let locked = deposit_mutability.eq(&DenyAll);
            main_roles.define_role(DEPOSITOR_ROLE, deposit_access_rule, locked);
            main_roles.define_role(DEPOSITOR_UPDATER_ROLE, deposit_mutability, locked);
        }
    }

    main_roles
}

pub fn features(
    track_total_supply: bool,
    access_rules: &BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
) -> Vec<&str> {
    let mut features = Vec::new();

    if track_total_supply {
        features.push(TRACK_TOTAL_SUPPLY_FEATURE);
    }

    if let Some((rule, updater)) = access_rules.get(&ResourceAction::Freeze) {
        if rule.ne(&AccessRule::DenyAll) || updater.ne(&AccessRule::DenyAll) {
            features.push(VAULT_FREEZE_FEATURE);
        }
    }

    if let Some((rule, updater)) = access_rules.get(&ResourceAction::Recall) {
        if rule.ne(&AccessRule::DenyAll) || updater.ne(&AccessRule::DenyAll) {
            features.push(VAULT_RECALL_FEATURE);
        }
    }

    if let Some((rule, updater)) = access_rules.get(&ResourceAction::Mint) {
        if rule.ne(&AccessRule::DenyAll) || updater.ne(&AccessRule::DenyAll) {
            features.push(MINT_FEATURE);
        }
    }

    if let Some((rule, updater)) = access_rules.get(&ResourceAction::Burn) {
        if rule.ne(&AccessRule::DenyAll) || updater.ne(&AccessRule::DenyAll) {
            features.push(BURN_FEATURE);
        }
    }

    features
}

pub fn globalize_resource_manager<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    metadata: ModuleConfig<MetadataInit>,
    api: &mut Y,
) -> Result<ResourceAddress, RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let main_roles = build_main_access_rules(access_rules);

    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );

    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata.init, api)?;

    let address = api.globalize(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
        ),
        Some(resource_address_reservation),
    )?;

    Ok(ResourceAddress::new_or_panic(address.into()))
}

pub fn globalize_fungible_with_initial_supply<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    metadata: ModuleConfig<MetadataInit>,
    initial_supply: Decimal,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let main_roles = build_main_access_rules(access_rules);
    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );
    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;
    let metadata = Metadata::create_with_data(metadata.init, api)?;

    let modules = btreemap!(
        ObjectModuleId::Main => object_id,
        ObjectModuleId::AccessRules => resman_access_rules.0,
        ObjectModuleId::Metadata => metadata.0,
    );

    let (address, bucket_id) = api.globalize_with_address_and_create_inner_object(
        modules,
        resource_address_reservation,
        FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidFungibleResource::new(initial_supply)).unwrap(),
            scrypto_encode(&LockedFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok((
        ResourceAddress::new_or_panic(address.into()),
        Bucket(Own(bucket_id)),
    ))
}

pub fn globalize_non_fungible_with_initial_supply<Y>(
    owner_role: OwnerRole,
    object_id: NodeId,
    resource_address_reservation: GlobalAddressReservation,
    access_rules: BTreeMap<ResourceAction, (AccessRule, AccessRule)>,
    metadata: ModuleConfig<MetadataInit>,
    ids: BTreeSet<NonFungibleLocalId>,
    api: &mut Y,
) -> Result<(ResourceAddress, Bucket), RuntimeError>
where
    Y: ClientApi<RuntimeError>,
{
    let main_roles = build_main_access_rules(access_rules);
    let roles = btreemap!(
        ObjectModuleId::Main => main_roles,
        ObjectModuleId::Metadata => metadata.roles,
    );
    let resman_access_rules = AccessRules::create(owner_role, roles, api)?.0;

    let metadata = Metadata::create_with_data(metadata.init, api)?;

    let (address, bucket_id) = api.globalize_with_address_and_create_inner_object(
        btreemap!(
            ObjectModuleId::Main => object_id,
            ObjectModuleId::AccessRules => resman_access_rules.0,
            ObjectModuleId::Metadata => metadata.0,
        ),
        resource_address_reservation,
        NON_FUNGIBLE_BUCKET_BLUEPRINT,
        vec![
            scrypto_encode(&LiquidNonFungibleResource::new(ids)).unwrap(),
            scrypto_encode(&LockedNonFungibleResource::default()).unwrap(),
        ],
    )?;

    Ok((
        ResourceAddress::new_or_panic(address.into()),
        Bucket(Own(bucket_id)),
    ))
}
