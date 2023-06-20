use radix_engine::types::*;
use scrypto_unit::*;
use utils::ContextualDisplay;

#[test]
fn scan_native_blueprint_schemas_and_highlight_unsafe_types() {
    let test_runner = TestRunner::builder().build();
    let bech32 = Bech32Encoder::for_simulator();

    let package_addresses = test_runner.find_all_packages();
    for package_address in package_addresses {
        let schemas_by_hash = test_runner.get_package_schema(&package_address);
        println!(
            "Found {} schemas for {}",
            schemas_by_hash.len(),
            package_address.to_string(&bech32)
        );

        for schema in schemas_by_hash.values() {
            for (i, type_kind) in schema.type_kinds.iter().enumerate() {
                match type_kind {
                    ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Own) => {
                        match &schema.type_validations[i] {
                            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(x)) => {
                                match x {
                                    OwnValidation::IsTypedObject(_, _) => {}
                                    OwnValidation::IsKeyValueStore => {
                                        println!("Warning: non typed KeyValueStore is used");
                                    }
                                    OwnValidation::IsGlobalAddressReservation => {
                                        println!(
                                            "Warning: non typed GlobalAddressReservation is used"
                                        );
                                    }
                                    _ => {
                                        println!("Warning: non typed validation {:?} used", x);
                                    }
                                }
                            }
                            _ => panic!("Wrong type validation attached to `Own` type kind"),
                        }
                    }
                    ScryptoTypeKind::Custom(ScryptoCustomTypeKind::Reference) => {
                        match &schema.type_validations[i] {
                            TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(x)) => {
                                match x {
                                    ReferenceValidation::IsGlobalTyped(_, _)
                                    | ReferenceValidation::IsInternalTyped(_, _) => {}
                                    _ => {
                                        println!("Warning: non typed validation {:?} used", x);
                                    }
                                }
                            }
                            _ => panic!("Wrong type validation attached to `Reference` type kind"),
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
