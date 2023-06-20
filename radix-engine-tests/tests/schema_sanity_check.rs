use radix_engine::types::*;
use radix_engine_common::prelude::well_known_scrypto_custom_types::*;
use radix_engine_interface::schema::BlueprintCollectionSchema;
use radix_engine_queries::typed_substate_layout::TypePointer;
use sbor::basic_well_known_types::*;
use scrypto_unit::*;
use utils::ContextualDisplay;

#[test]
fn scan_native_blueprint_schemas_and_highlight_unsafe_types() {
    let test_runner = TestRunner::builder().build();
    let bech32 = Bech32Encoder::for_simulator();

    let package_addresses = test_runner.find_all_packages();
    for package_address in package_addresses {
        println!("\nChecking {}", package_address.to_string(&bech32));

        let schemas_by_hash = test_runner.get_package_scrypto_schemas(&package_address);
        println!("Found {} schemas", schemas_by_hash.len());

        let blueprint_definitions = test_runner.get_package_blueprint_definitions(&package_address);
        for (key, definition) in blueprint_definitions {
            println!("Checking blueprint {:?}", key.blueprint);
            if let Some(fields) = definition.interface.state.fields {
                for (i, f) in fields.1.iter().enumerate() {
                    if !is_safe_type_pointer(&schemas_by_hash, &f.field) {
                        println!("Unsafe: field #{}", i);
                    }
                }
            }
            let collections = definition.interface.state.collections;
            for (partition, collection_schema) in collections {
                match collection_schema {
                    BlueprintCollectionSchema::KeyValueStore(kv) => {
                        if !is_safe_type_pointer(&schemas_by_hash, &kv.key) {
                            println!("Unsafe: key of partition #{:?}", partition.0);
                        }
                        if !is_safe_type_pointer(&schemas_by_hash, &kv.value) {
                            println!("Unsafe: value of partition #{:?}", partition.0);
                        }
                    }
                    BlueprintCollectionSchema::Index(_) => {
                        // TODO: add check when schema is added
                    }
                    BlueprintCollectionSchema::SortedIndex(_) => {
                        // TODO: add check when schema is added
                    }
                }
            }
            let functions = definition.interface.functions;
            for (name, func) in functions {
                if !is_safe_type_pointer(&schemas_by_hash, &func.input) {
                    println!("Unsafe: function input of {:?}", name);
                }
                if !is_safe_type_pointer(&schemas_by_hash, &func.output) {
                    println!("Unsafe: function output of {:?}", name);
                }
            }
            let events = definition.interface.events;
            for (name, ty) in events {
                if !is_safe_type_pointer(&schemas_by_hash, &ty) {
                    println!("Unsafe: event {:?}", name);
                }
            }
        }
    }
}

fn is_safe_type_pointer(
    schemas_by_hash: &IndexMap<Hash, ScryptoSchema>,
    type_pointer: &TypePointer,
) -> bool {
    match type_pointer {
        TypePointer::Package(hash, index) => {
            is_safe_type(schemas_by_hash.get(hash).unwrap(), *index)
        }
        TypePointer::Instance(_) => true,
    }
}

fn is_safe_type(schema: &ScryptoSchema, index: LocalTypeIndex) -> bool {
    let mut visited_indices = index_set_new();
    is_safe_type_internal(schema, index, &mut visited_indices)
}

fn is_safe_type_internal(
    schema: &ScryptoSchema,
    index: LocalTypeIndex,
    visited_indices: &mut IndexSet<LocalTypeIndex>,
) -> bool {
    if visited_indices.contains(&index) {
        return true;
    }
    visited_indices.insert(index);
    match index {
        LocalTypeIndex::WellKnown(x) => return is_safe_well_known_type(x),
        LocalTypeIndex::SchemaLocalIndex(i) => match &schema.type_kinds[i] {
            ScryptoTypeKind::Array { element_type } => {
                return is_safe_type_internal(schema, *element_type, visited_indices);
            }
            ScryptoTypeKind::Tuple { field_types } => {
                for ty in field_types {
                    if !is_safe_type_internal(schema, *ty, visited_indices) {
                        return false;
                    }
                }
                return true;
            }
            ScryptoTypeKind::Enum { variants } => {
                for v in variants {
                    for ty in v.1 {
                        if !is_safe_type_internal(schema, *ty, visited_indices) {
                            return false;
                        }
                    }
                }
                return true;
            }
            ScryptoTypeKind::Map {
                key_type,
                value_type,
            } => {
                return is_safe_type_internal(schema, *key_type, visited_indices)
                    && is_safe_type_internal(schema, *value_type, visited_indices);
            }
            ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own) => {
                match &schema.type_validations[i] {
                    TypeValidation::Custom(ScryptoCustomTypeValidation::Own(x)) => match x {
                        OwnValidation::IsTypedObject(_, _) => {
                            return true;
                        }
                        OwnValidation::IsKeyValueStore => {
                            // TODO: consider this as unsafe in native blueprints?
                            println!("Warning: KeyValueStore is used");
                            return true;
                        }
                        OwnValidation::IsGlobalAddressReservation => {
                            // TODO: consider this as unsafe in native blueprints?
                            println!("Warning: GlobalAddressReservation is used");
                            return true;
                        }
                        x => {
                            println!("Debug: unsafe own validation {:?}", x);
                            return false;
                        }
                    },
                    _ => panic!("Wrong type validation attached to `Own` type kind"),
                }
            }
            ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference) => {
                match &schema.type_validations[i] {
                    TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(x)) => match x {
                        ReferenceValidation::IsGlobalTyped(_, _)
                        | ReferenceValidation::IsInternalTyped(_, _)
                        | ReferenceValidation::IsGlobalPackage
                        | ReferenceValidation::IsGlobalResourceManager
                        | ReferenceValidation::IsGlobalComponent => {
                            return true;
                        }
                        x => {
                            println!("Debug: unsafe reference validation {:?}", x);
                            return false;
                        }
                    },
                    _ => panic!("Wrong type validation attached to `Reference` type kind"),
                }
            }
            _ => {
                return true;
            }
        },
    };
}

fn is_safe_well_known_type(type_id: u8) -> bool {
    let is_safe = match type_id {
        // Basic SBOR
        BOOL_ID => true,
        I8_ID => true,
        I16_ID => true,
        I32_ID => true,
        I64_ID => true,
        I128_ID => true,
        U8_ID => true,
        U16_ID => true,
        U32_ID => true,
        U64_ID => true,
        U128_ID => true,
        STRING_ID => true,
        ANY_ID => false,
        BYTES_ID => true,
        UNIT_ID => true,

        // Scrypto SBOR
        REFERENCE_ID => false,
        GLOBAL_ADDRESS_ID => false,
        INTERNAL_ADDRESS_ID => false,
        PACKAGE_ADDRESS_ID => true,
        COMPONENT_ADDRESS_ID => true,
        RESOURCE_ADDRESS_ID => true,
        OWN_ID => false,
        OWN_BUCKET_ID => false,
        OWN_FUNGIBLE_BUCKET_ID => true,
        OWN_NON_FUNGIBLE_BUCKET_ID => true,
        OWN_PROOF_ID => false,
        OWN_FUNGIBLE_PROOF_ID => true,
        OWN_NON_FUNGIBLE_PROOF_ID => true,
        OWN_VAULT_ID => false,
        OWN_FUNGIBLE_VAULT_ID => true,
        OWN_NON_FUNGIBLE_VAULT_ID => true,
        OWN_KEY_VALUE_STORE_ID => false,
        OWN_GLOBAL_ADDRESS_RESERVATION_ID => true,
        DECIMAL_ID => true,
        PRECISE_DECIMAL_ID => true,
        NON_FUNGIBLE_LOCAL_ID_ID => true,
        t => panic!("Unexpected well-known type id: {}", t),
    };

    if !is_safe {
        println!("Debug: unsafe well-known type {:?}", type_id);
    }

    return is_safe;
}
