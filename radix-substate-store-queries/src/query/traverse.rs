use radix_common::constants::RESOURCE_PACKAGE;
use radix_common::data::scrypto::model::NonFungibleLocalId;
use radix_common::prelude::*;
use radix_engine::blueprints::resource::*;
use radix_engine::object_modules::royalty::{
    ComponentRoyaltyAccumulatorFieldPayload, ComponentRoyaltyField,
};
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_engine::system::type_info::TypeInfoSubstate;
use radix_engine_interface::api::{AttachedModuleId, ModuleId};
use radix_engine_interface::blueprints::resource::{
    LiquidNonFungibleVault, FUNGIBLE_VAULT_BLUEPRINT, NON_FUNGIBLE_VAULT_BLUEPRINT,
};
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::{
    BlueprintId, IndexedScryptoValue, ObjectType, ResourceAddress,
};
use radix_engine_interface::{blueprints::resource::LiquidFungibleResource, types::NodeId};
use radix_substate_store_interface::interface::SubstateDatabase;

pub struct StateTreeTraverser<'s, 'v, S: SubstateDatabase, V: StateTreeVisitor + 'v> {
    system_db_reader: SystemDatabaseReader<'s, S>,
    visitor: &'v mut V,
    max_depth: u32,
}

pub trait StateTreeVisitor {
    fn visit_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        _address: &ResourceAddress,
        _resource: &LiquidFungibleResource,
    ) {
    }

    fn visit_non_fungible_vault(
        &mut self,
        _vault_id: NodeId,
        _address: &ResourceAddress,
        _resource: &LiquidNonFungibleVault,
    ) {
    }

    fn visit_non_fungible(
        &mut self,
        _vault_id: NodeId,
        _address: &ResourceAddress,
        _id: &NonFungibleLocalId,
    ) {
    }

    fn visit_node_id(
        &mut self,
        _parent_id: Option<&(NodeId, PartitionNumber, SubstateKey)>,
        _node_id: &NodeId,
        _depth: u32,
    ) {
    }
}

impl<'s, 'v, S: SubstateDatabase, V: StateTreeVisitor + 'v> StateTreeTraverser<'s, 'v, S, V> {
    pub fn new(substate_db: &'s S, visitor: &'v mut V, max_depth: u32) -> Self {
        StateTreeTraverser {
            system_db_reader: SystemDatabaseReader::new(substate_db),
            visitor,
            max_depth,
        }
    }

    pub fn traverse_subtree(
        &mut self,
        parent_node_id: Option<&(NodeId, PartitionNumber, SubstateKey)>,
        node_id: NodeId,
    ) {
        Self::traverse_recursive(
            &self.system_db_reader,
            self.visitor,
            parent_node_id,
            node_id,
            0,
            self.max_depth,
        )
    }

    fn traverse_recursive(
        system_db_reader: &SystemDatabaseReader<'s, S>,
        visitor: &mut V,
        parent: Option<&(NodeId, PartitionNumber, SubstateKey)>,
        node_id: NodeId,
        depth: u32,
        max_depth: u32,
    ) {
        if depth > max_depth {
            return;
        }

        // Notify visitor
        visitor.visit_node_id(parent, &node_id, depth);

        // Load type info
        let type_info = system_db_reader
            .get_type_info(&node_id)
            .expect("Missing TypeInfo substate");

        match type_info {
            TypeInfoSubstate::KeyValueStore(_) => {
                for (key, value) in system_db_reader
                    .key_value_store_iter(&node_id, None)
                    .unwrap()
                {
                    let (_, owned_nodes, _) =
                        IndexedScryptoValue::from_slice(&value).unwrap().unpack();
                    for child_node_id in owned_nodes {
                        Self::traverse_recursive(
                            system_db_reader,
                            visitor,
                            Some(&(node_id, MAIN_BASE_PARTITION, SubstateKey::Map(key.clone()))),
                            child_node_id,
                            depth + 1,
                            max_depth,
                        );
                    }
                }
            }
            TypeInfoSubstate::Object(info) => {
                if info.blueprint_info.blueprint_id.eq(&BlueprintId::new(
                    &RESOURCE_PACKAGE,
                    FUNGIBLE_VAULT_BLUEPRINT,
                )) {
                    let liquid: VersionedFungibleVaultBalance = system_db_reader
                        .read_typed_object_field(
                            &node_id,
                            ModuleId::Main,
                            FungibleVaultField::Balance.into(),
                        )
                        .expect("Broken database");

                    let liquid = liquid.fully_update_and_into_latest_version();

                    visitor.visit_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                        &liquid,
                    );
                } else if info.blueprint_info.blueprint_id.eq(&BlueprintId::new(
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_VAULT_BLUEPRINT,
                )) {
                    let liquid: VersionedNonFungibleVaultBalance = system_db_reader
                        .read_typed_object_field(
                            &node_id,
                            ModuleId::Main,
                            NonFungibleVaultField::Balance.into(),
                        )
                        .expect("Broken database");

                    let liquid = liquid.fully_update_and_into_latest_version();

                    visitor.visit_non_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                        &liquid,
                    );

                    for (key, _value) in system_db_reader
                        .collection_iter(
                            &node_id,
                            ModuleId::Main,
                            NonFungibleVaultCollection::NonFungibleIndex.collection_index(),
                        )
                        .unwrap()
                    {
                        let map_key = key.into_map();
                        let non_fungible_local_id: NonFungibleLocalId =
                            scrypto_decode(&map_key).unwrap();
                        visitor.visit_non_fungible(
                            node_id,
                            &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                            &non_fungible_local_id,
                        );
                    }
                } else {
                    match info.object_type {
                        ObjectType::Global { modules } => {
                            for (module_id, _) in modules {
                                match &module_id {
                                    AttachedModuleId::Royalty => {
                                        let royalty = system_db_reader
                                            .read_typed_object_field::<ComponentRoyaltyAccumulatorFieldPayload>(
                                                &node_id,
                                                module_id.into(),
                                                0u8,
                                            )
                                            .expect("Broken database")
                                            .fully_update_and_into_latest_version();
                                        Self::traverse_recursive(
                                            system_db_reader,
                                            visitor,
                                            Some(&(
                                                node_id,
                                                ROYALTY_BASE_PARTITION,
                                                SubstateKey::Field(
                                                    ComponentRoyaltyField::Accumulator
                                                        .field_index(),
                                                ),
                                            )),
                                            royalty.royalty_vault.0 .0,
                                            depth + 1,
                                            max_depth,
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                        ObjectType::Owned => {}
                    }

                    let blueprint_def = system_db_reader
                        .get_blueprint_definition(&info.blueprint_info.blueprint_id)
                        .expect("Broken database");

                    if let Some((_, fields)) = &blueprint_def.interface.state.fields {
                        for (index, _field) in fields.iter().enumerate() {
                            // TODO: what if the field is conditional?
                            let (field_value, partition_number) = system_db_reader
                                .read_object_field_advanced(&node_id, ModuleId::Main, index as u8)
                                .expect("Broken database");
                            let (_, owned_nodes, _) = field_value.unpack();
                            for child_node_id in owned_nodes {
                                Self::traverse_recursive(
                                    system_db_reader,
                                    visitor,
                                    Some(&(
                                        node_id,
                                        partition_number,
                                        SubstateKey::Field(index as u8),
                                    )),
                                    child_node_id,
                                    depth + 1,
                                    max_depth,
                                );
                            }
                        }
                    }

                    for (index, _collection) in
                        blueprint_def.interface.state.collections.iter().enumerate()
                    {
                        let (iter, partition_number) = system_db_reader
                            .collection_iter_advanced(&node_id, ModuleId::Main, index as u8, None)
                            .unwrap();

                        for (substate_key, value) in iter {
                            let (_, owned_nodes, _) =
                                IndexedScryptoValue::from_slice(&value).unwrap().unpack();
                            for child_node_id in owned_nodes {
                                Self::traverse_recursive(
                                    system_db_reader,
                                    visitor,
                                    Some(&(node_id, partition_number, substate_key.clone())),
                                    child_node_id,
                                    depth + 1,
                                    max_depth,
                                );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
