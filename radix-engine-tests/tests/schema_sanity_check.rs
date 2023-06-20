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
                    let result = check_type_pointer(&schemas_by_hash, &f.field);
                    if result.is_not_safe() {
                        println!("Field {:?} is {:?}", i, result);
                    }
                }
            }
            let collections = definition.interface.state.collections;
            for (partition, collection_schema) in collections {
                match collection_schema {
                    BlueprintCollectionSchema::KeyValueStore(kv) => {
                        let result = check_type_pointers(&schemas_by_hash, &[kv.key, kv.value]);
                        if result.is_not_safe() {
                            println!("Partition {:?} is {:?}", partition.0, result);
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
                let result = check_type_pointers(&schemas_by_hash, &[func.input, func.output]);
                if result.is_not_safe() {
                    println!("Function {:?} is {:?}", name, result);
                }
            }
            let events = definition.interface.events;
            for (name, ty) in events {
                let result = check_type_pointer(&schemas_by_hash, &ty);
                if result.is_not_safe() {
                    println!("Event {:?} is {:?}", name, result);
                }
            }
        }
    }
}

fn check_type_pointers(
    schemas_by_hash: &IndexMap<Hash, ScryptoSchema>,
    type_pointers: &[TypePointer],
) -> CheckResult {
    for ty in type_pointers {
        let result = check_type_pointer(schemas_by_hash, ty);
        if result.is_not_safe() {
            return result;
        }
    }
    return CheckResult::Safe;
}

fn check_type_pointer(
    schemas_by_hash: &IndexMap<Hash, ScryptoSchema>,
    type_pointer: &TypePointer,
) -> CheckResult {
    match type_pointer {
        TypePointer::Package(hash, index) => check_type(schemas_by_hash.get(hash).unwrap(), *index),
        TypePointer::Instance(_) => CheckResult::Safe,
    }
}

fn check_type(schema: &ScryptoSchema, index: LocalTypeIndex) -> CheckResult {
    let mut visited_indices = index_set_new();
    check_type_internal(schema, index, &mut visited_indices)
}

fn check_types_internal(
    schema: &ScryptoSchema,
    indices: &[LocalTypeIndex],
    visited_indices: &mut IndexSet<LocalTypeIndex>,
) -> CheckResult {
    for index in indices {
        let result = check_type_internal(schema, *index, visited_indices);
        if result.is_not_safe() {
            return result;
        }
    }
    CheckResult::Safe
}

fn check_type_internal(
    schema: &ScryptoSchema,
    index: LocalTypeIndex,
    visited_indices: &mut IndexSet<LocalTypeIndex>,
) -> CheckResult {
    if visited_indices.contains(&index) {
        return CheckResult::Safe;
    }
    visited_indices.insert(index);
    match index {
        LocalTypeIndex::WellKnown(x) => return is_safe_well_known_type(schema, x),
        LocalTypeIndex::SchemaLocalIndex(i) => {
            let type_kind = &schema.type_kinds[i];
            match type_kind {
                ScryptoTypeKind::Array { element_type } => {
                    return check_type_internal(schema, *element_type, visited_indices);
                }
                ScryptoTypeKind::Tuple { field_types } => {
                    return check_types_internal(schema, field_types, visited_indices);
                }
                ScryptoTypeKind::Enum { variants } => {
                    let mut indices = Vec::<LocalTypeIndex>::new();
                    for v in variants {
                        for ty in v.1 {
                            indices.push(*ty);
                        }
                    }
                    return check_types_internal(schema, &indices, visited_indices);
                }
                ScryptoTypeKind::Map {
                    key_type,
                    value_type,
                } => {
                    return check_types_internal(
                        schema,
                        &[*key_type, *value_type],
                        visited_indices,
                    );
                }
                ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own) => {
                    match &schema.type_validations[i] {
                        TypeValidation::Custom(ScryptoCustomTypeValidation::Own(x)) => match x {
                            OwnValidation::IsTypedObject(_, _) => {
                                return CheckResult::Safe;
                            }
                            OwnValidation::IsKeyValueStore => {
                                // TODO: consider this as unsafe in native blueprints?
                                return CheckResult::Safe;
                            }
                            OwnValidation::IsGlobalAddressReservation => {
                                // TODO: consider this as unsafe in native blueprints?
                                return CheckResult::Safe;
                            }
                            _ => {
                                return CheckResult::PossiblyUnsafe {
                                    type_kind: type_kind.clone(),
                                    type_validation: schema.type_validations[i].clone(),
                                };
                            }
                        },
                        _ => panic!("Wrong type validation attached to `Own` type kind"),
                    }
                }
                ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference) => {
                    match &schema.type_validations[i] {
                        TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(x)) => {
                            match x {
                                ReferenceValidation::IsGlobalTyped(_, _)
                                | ReferenceValidation::IsInternalTyped(_, _)
                                | ReferenceValidation::IsGlobalPackage
                                | ReferenceValidation::IsGlobalResourceManager
                                | ReferenceValidation::IsGlobalComponent => {
                                    return CheckResult::Safe;
                                }
                                _ => {
                                    return CheckResult::PossiblyUnsafe {
                                        type_kind: type_kind.clone(),
                                        type_validation: schema.type_validations[i].clone(),
                                    };
                                }
                            }
                        }
                        _ => panic!("Wrong type validation attached to `Reference` type kind"),
                    }
                }
                _ => {
                    return CheckResult::Safe;
                }
            }
        }
    };
}

fn is_safe_well_known_type(schema: &ScryptoSchema, type_id: u8) -> CheckResult {
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
        OWN_BUCKET_ID => true, // TODO: maybe unsafe?
        OWN_FUNGIBLE_BUCKET_ID => true,
        OWN_NON_FUNGIBLE_BUCKET_ID => true,
        OWN_PROOF_ID => true, // TODO: maybe unsafe?
        OWN_FUNGIBLE_PROOF_ID => true,
        OWN_NON_FUNGIBLE_PROOF_ID => true,
        OWN_VAULT_ID => false,
        OWN_FUNGIBLE_VAULT_ID => true,
        OWN_NON_FUNGIBLE_VAULT_ID => true,
        OWN_KEY_VALUE_STORE_ID => true, // TODO: maybe unsafe?
        OWN_GLOBAL_ADDRESS_RESERVATION_ID => true,
        DECIMAL_ID => true,
        PRECISE_DECIMAL_ID => true,
        NON_FUNGIBLE_LOCAL_ID_ID => true,
        t => panic!("Unexpected well-known type id: {}", t),
    };

    if is_safe {
        CheckResult::Safe
    } else {
        CheckResult::PossiblyUnsafe {
            type_kind: schema
                .resolve_type_kind(LocalTypeIndex::WellKnown(type_id))
                .unwrap()
                .clone(),
            type_validation: schema
                .resolve_type_validation(LocalTypeIndex::WellKnown(type_id))
                .unwrap()
                .clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CheckResult {
    Safe,
    PossiblyUnsafe {
        type_kind: ScryptoTypeKind<LocalTypeIndex>,
        type_validation: TypeValidation<ScryptoCustomTypeValidation>,
    },
}

impl CheckResult {
    fn is_safe(&self) -> bool {
        matches!(self, CheckResult::Safe)
    }
    fn is_not_safe(&self) -> bool {
        !self.is_safe()
    }
}
