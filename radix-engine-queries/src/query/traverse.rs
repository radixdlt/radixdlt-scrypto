use radix_engine::system::node_modules::type_info::TypeInfoSubstate;
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_engine_interface::api::node_modules::royalty::ComponentRoyaltySubstate;
use radix_engine_interface::api::{ModuleId, ObjectModuleId};
use radix_engine_interface::blueprints::resource::{
    LiquidNonFungibleVault, FUNGIBLE_VAULT_BLUEPRINT, NON_FUNGIBLE_VAULT_BLUEPRINT,
};
use radix_engine_interface::constants::RESOURCE_PACKAGE;
use radix_engine_interface::data::scrypto::model::NonFungibleLocalId;
use radix_engine_interface::prelude::scrypto_decode;
use radix_engine_interface::types::{
    BlueprintId, FungibleVaultField, IndexedScryptoValue, NonFungibleVaultField, ObjectType,
    ResourceAddress,
};
use radix_engine_interface::{blueprints::resource::LiquidFungibleResource, types::NodeId};
use radix_engine_store_interface::interface::SubstateDatabase;
use sbor::rust::prelude::*;

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
}

impl<'s, 'v, S: SubstateDatabase, V: StateTreeVisitor + 'v> StateTreeTraverser<'s, 'v, S, V> {
    pub fn new(substate_db: &'s S, visitor: &'v mut V, max_depth: u32) -> Self {
        StateTreeTraverser {
            system_db_reader: SystemDatabaseReader::new(substate_db),
            visitor,
            max_depth,
        }
    }

    pub fn traverse_all_descendents(&mut self, node_id: NodeId) {
        Self::traverse_recursive(
            &self.system_db_reader,
            &mut self.visitor,
            node_id,
            0,
            self.max_depth,
        )
    }

    fn traverse_recursive(
        system_db_reader: &SystemDatabaseReader<'s, S>,
        visitor: &mut V,
        node_id: NodeId,
        depth: u32,
        max_depth: u32,
    ) {
        if depth > max_depth {
            return;
        }

        // Load type info
        let type_info = system_db_reader
            .get_type_info(&node_id)
            .expect("Missing TypeInfo substate");

        match type_info {
            TypeInfoSubstate::KeyValueStore(_) => {
                for (_key, value) in system_db_reader.key_value_store_iter(&node_id).unwrap() {
                    let (_, owned_nodes, _) =
                        IndexedScryptoValue::from_slice(&value).unwrap().unpack();
                    for child_node_id in owned_nodes {
                        Self::traverse_recursive(
                            system_db_reader,
                            visitor,
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
                    let liquid: LiquidFungibleResource = system_db_reader
                        .read_typed_object_field(
                            &node_id,
                            ObjectModuleId::Main,
                            FungibleVaultField::LiquidFungible.into(),
                        )
                        .expect("Broken database");

                    visitor.visit_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                        &liquid,
                    );
                } else if info.blueprint_info.blueprint_id.eq(&BlueprintId::new(
                    &RESOURCE_PACKAGE,
                    NON_FUNGIBLE_VAULT_BLUEPRINT,
                )) {
                    let liquid: LiquidNonFungibleVault = system_db_reader
                        .read_typed_object_field(
                            &node_id,
                            ObjectModuleId::Main,
                            NonFungibleVaultField::LiquidNonFungible.into(),
                        )
                        .expect("Broken database");

                    visitor.visit_non_fungible_vault(
                        node_id,
                        &ResourceAddress::new_or_panic(info.get_outer_object().into()),
                        &liquid,
                    );

                    for (key, _value) in system_db_reader
                        .collection_iter(&node_id, ObjectModuleId::Main, 0u8)
                        .unwrap()
                    {
                        let non_fungible_local_id: NonFungibleLocalId =
                            scrypto_decode(&key).unwrap();
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
                                    ModuleId::Royalty => {
                                        let royalty: ComponentRoyaltySubstate = system_db_reader
                                            .read_typed_object_field(
                                                &node_id,
                                                module_id.into(),
                                                0u8,
                                            )
                                            .expect("Broken database");
                                        Self::traverse_recursive(
                                            system_db_reader,
                                            visitor,
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

                    if let Some((_, fields)) = blueprint_def.interface.state.fields {
                        for (index, _field) in fields.iter().enumerate() {
                            let field_value = system_db_reader
                                .read_object_field(&node_id, ObjectModuleId::Main, index as u8)
                                .expect("Broken database");
                            let (_, owned_nodes, _) = field_value.unpack();
                            for child_node_id in owned_nodes {
                                Self::traverse_recursive(
                                    system_db_reader,
                                    visitor,
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
                        for (_key, value) in system_db_reader
                            .collection_iter(&node_id, ObjectModuleId::Main, index as u8)
                            .unwrap()
                        {
                            let (_, owned_nodes, _) =
                                IndexedScryptoValue::from_slice(&value).unwrap().unpack();
                            for child_node_id in owned_nodes {
                                Self::traverse_recursive(
                                    system_db_reader,
                                    visitor,
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
