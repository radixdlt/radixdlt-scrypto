use radix_blueprint_schema_init::*;
use radix_common::constants::MAX_NUMBER_OF_BLUEPRINT_FIELDS;
use radix_common::prelude::*;
use radix_engine::blueprints::package::*;
use radix_engine::errors::*;
use radix_engine::system::system_modules::auth::*;
use radix_engine::vm::wasm::PrepareError;
use radix_engine::vm::wasm::*;
use radix_engine_interface::*;
use radix_engine_tests::common::*;
use sbor::basic_well_known_types::*;
use scrypto_test::prelude::*;

#[test]
fn missing_memory_should_cause_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let code = wat2wasm(
        r#"
            (module
                (func (export "test") (result i32)
                    i32.const 1337
                )
            )
            "#,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            PackageDefinition::default(),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            &RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(PrepareError::InvalidMemory(
                    InvalidMemory::MissingMemorySection
                ))
            ))
        )
    });
}

#[test]
fn large_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("package"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "LargeReturnSize", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::VmError(VmError::Wasm(b)) = e {
            matches!(*b, WasmRuntimeError::MemoryAccessError)
        } else {
            false
        }
    });
}

#[test]
fn overflow_return_len_should_cause_memory_access_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("package"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "MaxReturnSize", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        if let RuntimeError::VmError(VmError::Wasm(b)) = e {
            matches!(*b, WasmRuntimeError::MemoryAccessError)
        } else {
            false
        }
    });
}

#[test]
fn zero_return_len_should_cause_data_validation_error() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("package"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package, "ZeroReturnSize", "f", manifest_args!())
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| matches!(e, RuntimeError::SystemUpstreamError(_)));
}

#[test]
fn test_basic_package() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let code = wat2wasm(include_local_wasm_str!("basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            single_function_package_definition("Test", "f"),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_basic_package_missing_export() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let mut blueprints = index_map_new();
    blueprints.insert(
        "Test".to_string(),
        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set: indexset!(),
            dependencies: indexset!(),

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema: VersionedScryptoSchema::from_latest_version(SchemaV1 {
                    type_kinds: vec![],
                    type_metadata: vec![],
                    type_validations: vec![],
                }),
                state: BlueprintStateSchemaInit {
                    fields: vec![FieldSchema::static_field(LocalTypeId::WellKnown(UNIT_TYPE))],
                    collections: vec![],
                },
                events: BlueprintEventSchemaInit::default(),
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit {
                    functions: indexmap!(
                        "f".to_string() => FunctionSchemaInit {
                            receiver: Option::None,
                            input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                            output: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                            export: "not_exist".to_string(),
                        }
                    ),
                },
                hooks: BlueprintHooksInit::default(),
            },

            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig::default(),
        },
    );
    // Act
    let code = wat2wasm(include_local_wasm_str!("basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            PackageDefinition { blueprints },
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(PrepareError::MissingExport { .. })
            ))
        )
    });
}

#[test]
fn bad_radix_blueprint_schema_init_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let (code, mut definition) = PackageLoader::get("package");
    let blueprint_schema = &mut definition
        .blueprints
        .iter_mut()
        .next()
        .unwrap()
        .1
        .schema
        .schema;
    blueprint_schema
        .v1_mut()
        .type_metadata
        .push(TypeMetadata::unnamed());

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidBlueprintSchema(..)
            ))
        )
    });
}

#[test]
fn bad_function_schema_should_fail() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let (code, definition) = PackageLoader::get("package_invalid");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidLocalTypeId(_)
            ))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_wasm_package_outside_of_transaction_processor() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("publish_package"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "PublishPackage",
            "publish_package",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_advanced_wasm_package_outside_of_transaction_processor() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("publish_package"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "PublishPackage",
            "publish_package_advanced",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_native_packages() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            PACKAGE_PACKAGE,
            PACKAGE_BLUEPRINT,
            "publish_native",
            PackagePublishNativeManifestInput {
                package_address: None,
                native_package_code_id: 0u64,
                definition: PackageDefinition::default(),
                metadata: metadata_init!(),
            },
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn should_not_be_able_to_publish_native_packages_in_scrypto() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package = ledger.publish_package_simple(PackageLoader::get("publish_package"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package,
            "PublishPackage",
            "publish_native",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::SystemModuleError(SystemModuleError::AuthError(AuthError::Unauthorized(
                ..
            )))
        )
    });
}

#[test]
fn name_validation_blueprint() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("publish_package");

    definition.blueprints = indexmap![
       String::from("wrong_bluepint_name_*") =>
            definition
                .blueprints
                .values_mut()
                .next()
                .unwrap()
                .to_owned(),
    ];

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidName { .. }
            ))
        )
    });
}

#[test]
fn name_validation_feature_set() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("publish_package");

    definition
        .blueprints
        .values_mut()
        .next()
        .unwrap()
        .feature_set
        .insert(String::from("wrong-feature"));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidName { .. }
            ))
        )
    });
}

#[test]
fn well_known_types_in_schema_are_validated() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    let (code, mut definition) = PackageLoader::get("publish_package");

    let method_definition = definition
        .blueprints
        .values_mut()
        .next()
        .unwrap()
        .schema
        .functions
        .functions
        .get_mut(&String::from("some_method"))
        .unwrap();

    // Invalid well known type
    method_definition.input = TypeRef::Static(LocalTypeId::WellKnown(WellKnownTypeId::of(0)));

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(None, code, definition, BTreeMap::new(), OwnerRole::None)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidLocalTypeId(..)
            ))
        )
    });
}

#[test]
fn publishing_of_package_with_blueprint_name_exceeding_length_limit_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    let (_, value) = definition.blueprints.pop().unwrap();
    definition
        .blueprints
        .insert(name(MAX_BLUEPRINT_NAME_LEN + 1, 'A'), value);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::ExceededMaxBlueprintNameLen {
                    limit: 100,
                    actual: 101
                }
            ))
        )
    })
}

#[test]
fn publishing_of_package_where_outer_blueprint_is_inner_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    let (bp_name1, mut bp_definition1) = definition.blueprints.pop().unwrap();
    let (bp_name2, mut bp_definition2) = definition.blueprints.pop().unwrap();

    bp_definition1.blueprint_type = BlueprintType::Inner {
        outer_blueprint: "NoneExistent".to_owned(),
    };
    bp_definition2.blueprint_type = BlueprintType::Inner {
        outer_blueprint: bp_name1.clone(),
    };

    definition.blueprints.insert(bp_name2, bp_definition2);
    definition.blueprints.insert(bp_name1, bp_definition1);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::OuterBlueprintCantBeAnInnerBlueprint { .. }
            ))
        )
    })
}

#[test]
fn publishing_of_package_where_outer_blueprint_is_self_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    let (bp_name, mut bp_definition) = definition.blueprints.pop().unwrap();
    bp_definition.blueprint_type = BlueprintType::Inner {
        outer_blueprint: bp_name.clone(),
    };
    definition.blueprints.insert(bp_name, bp_definition);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::MissingOuterBlueprint
            ))
        )
    })
}

#[test]
fn publishing_of_package_with_transient_blueprints_fails() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition
        .blueprints
        .values_mut()
        .for_each(|def| def.is_transient = true);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_failure_containing_error("Transient blueprints not supported");
}

#[test]
fn publishing_of_package_with_whitespace_in_blueprint_name_fails() {
    test_publishing_of_packages_with_invalid_names("\nHelloWorld")
}

#[test]
fn publishing_of_package_with_number_at_start_of_blueprint_name_fails() {
    test_publishing_of_packages_with_invalid_names("1000HelloWorld")
}

#[test]
fn publishing_of_package_with_a_hidden_ascii_character_fails() {
    test_publishing_of_packages_with_invalid_names("World")
}

#[test]
fn publishing_of_package_with_a_lookalike_character_fails() {
    test_publishing_of_packages_with_invalid_names("depοsit")
}

#[test]
fn test_error_path_when_package_definition_has_too_many_fields() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        if let Some(field) = bp_def.schema.state.fields.first() {
            bp_def.schema.state.fields = (0..MAX_NUMBER_OF_BLUEPRINT_FIELDS + 1)
                .map(|_| field.clone())
                .collect::<Vec<_>>();
        }
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::TooManySubstateSchemas
            ))
        )
    })
}

#[test]
fn test_error_path_field_requires_a_non_existent_feature_on_the_same_blueprint() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        if let Some(field) = bp_def.schema.state.fields.first_mut() {
            field.condition = Condition::IfFeature("Foo".to_owned())
        }
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::FeatureDoesNotExist(..)
            ))
        )
    })
}

#[test]
fn test_error_path_field_requires_a_feature_on_an_outer_blueprint_when_its_blueprint_is_outer() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        if let Some(field) = bp_def.schema.state.fields.first_mut() {
            field.condition = Condition::IfOuterFeature("Foo".to_owned())
        }
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::FeatureDoesNotExist(..)
            ))
        )
    })
}

#[test]
fn test_error_path_field_requires_a_feature_on_an_outer_blueprint_that_does_not_contain_this_feature(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    let (name1, mut def1) = definition.blueprints.pop().unwrap();
    let (name2, def2) = definition.blueprints.pop().unwrap();

    def1.blueprint_type = BlueprintType::Inner {
        outer_blueprint: name2.clone(),
    };
    def1.schema.state.fields.first_mut().unwrap().condition =
        Condition::IfOuterFeature("Foo".to_owned());

    definition.blueprints.insert(name1, def1);
    definition.blueprints.insert(name2, def2);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::FeatureDoesNotExist(..)
            ))
        )
    })
}

#[test]
fn test_error_path_transient_field_can_not_have_a_generic_type_pointer() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.schema.generics.push(GenericBound::Any);
        if let Some(field) = bp_def.schema.state.fields.first_mut() {
            field.transience = FieldTransience::TransientStatic {
                default_value: vec![],
            };
            field.field = TypeRef::Generic(0)
        }
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidTransientField
            ))
        )
    })
}

#[test]
fn test_error_path_can_not_have_an_event_with_invalid_local_type_id() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.schema.events.event_schema.insert(
            "Foo".to_owned(),
            TypeRef::Static(LocalTypeId::SchemaLocalIndex(usize::MAX)),
        );
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidLocalTypeId(LocalTypeId::SchemaLocalIndex(usize::MAX))
            ))
        )
    })
}

#[test]
fn test_error_path_can_not_have_an_event_with_a_generic_schema_pointer() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.schema.generics.push(GenericBound::Any);
        bp_def
            .schema
            .events
            .event_schema
            .insert("Foo".to_owned(), TypeRef::Generic(0));
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::EventGenericTypeNotSupported
            ))
        )
    })
}

#[test]
fn test_error_path_can_not_have_a_type_schema_with_a_non_existent_local_type_id() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def
            .schema
            .types
            .type_schema
            .insert("Foo".to_owned(), LocalTypeId::SchemaLocalIndex(usize::MAX));
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidLocalTypeId(LocalTypeId::SchemaLocalIndex(usize::MAX))
            ))
        )
    })
}

#[test]
fn test_error_path_royalties_must_be_specified_for_all_functions_not_just_the_right_number_of_functions(
) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.royalty_config = PackageRoyaltyConfig::Enabled(
            bp_def
                .schema
                .functions
                .functions
                .keys()
                .map(|func_name| (format!("altered_{}", func_name), RoyaltyAmount::Free))
                .collect(),
        );
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::MissingFunctionRoyalty { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_access_rules_must_be_defined_for_all_functions_if_enabled() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.auth_config.function_auth = FunctionAuth::AccessRules(Default::default());
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::UnexpectedNumberOfFunctionAuth { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_access_rules_must_be_defined_for_all_functions_if_enabled2() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.auth_config.function_auth = FunctionAuth::AccessRules(
            bp_def
                .schema
                .functions
                .functions
                .iter()
                .filter_map(|(func_name, func_def)| {
                    if func_def.receiver.is_none() {
                        Some((format!("altered_{}", func_name), AccessRule::AllowAll))
                    } else {
                        None
                    }
                })
                .collect(),
        );
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::MissingFunctionPermission { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_a_method_can_not_be_protected_by_a_role_not_in_the_role_specification() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.auth_config.method_auth =
            MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                roles: RoleSpecification::Normal(Default::default()), /* Empty - no roles */
                methods: bp_def
                    .schema
                    .functions
                    .functions
                    .iter()
                    .filter_map(|(func_name, func_def)| {
                        if func_def.receiver.is_some() {
                            Some((
                                MethodKey::new(func_name),
                                MethodAccessibility::RoleProtected(RoleList {
                                    list: vec![RoleKey::new("NonExistentRole")],
                                }),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect(),
            });
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::MissingRole { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_reserved_role_is_rejected_during_validation() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.auth_config.method_auth =
            MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                roles: RoleSpecification::Normal(indexmap! {
                    RoleKey::new("_reserved_key_") => RoleList::default()
                }),
                methods: bp_def
                    .schema
                    .functions
                    .functions
                    .iter()
                    .filter_map(|(func_name, func_def)| {
                        if func_def.receiver.is_some() {
                            Some((
                                MethodKey::new(func_name),
                                MethodAccessibility::RoleProtected(RoleList { list: vec![] }),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect(),
            });
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::DefiningReservedRoleKey { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_incorrect_number_of_method_auth_is_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.auth_config.method_auth =
            MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                roles: RoleSpecification::Normal(Default::default()),
                methods: Default::default(),
            });
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::UnexpectedNumberOfMethodAuth { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_incorrect_names_of_methods_in_method_auth_is_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.auth_config.method_auth =
            MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                roles: RoleSpecification::Normal(Default::default()),
                methods: bp_def
                    .schema
                    .functions
                    .functions
                    .iter()
                    .filter_map(|(func_name, func_def)| {
                        if func_def.receiver.is_some() {
                            Some((
                                MethodKey::new(format!("altered_{}", func_name)),
                                MethodAccessibility::RoleProtected(RoleList { list: vec![] }),
                            ))
                        } else {
                            None
                        }
                    })
                    .collect(),
            });
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::MissingMethodPermission { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_long_blueprint_name_is_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    let (_, def1) = definition.blueprints.pop().unwrap();
    definition
        .blueprints
        .insert(name(MAX_BLUEPRINT_NAME_LEN + 1, 'A'), def1);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::ExceededMaxBlueprintNameLen { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_long_function_name_is_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.schema.functions.functions = bp_def
            .schema
            .functions
            .functions
            .iter()
            .map(|(func_name, def)| {
                (
                    format!("{}_{}", func_name, name(MAX_FUNCTION_NAME_LEN, 'A')),
                    def.clone(),
                )
            })
            .collect()
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::ExceededMaxFunctionNameLen { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_long_feature_name_is_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.feature_set = indexset!(name(MAX_FEATURE_NAME_LEN + 1, 'A'));
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::ExceededMaxFeatureNameLen { .. }
            ))
        )
    })
}

#[test]
fn test_error_path_function_with_generic_inputs_is_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.schema.generics.push(GenericBound::Any);
        bp_def
            .schema
            .functions
            .functions
            .values_mut()
            .for_each(|def| def.input = TypeRef::Generic(0))
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| format!("{error:?}").contains("Generics not supported"))
}

#[test]
fn test_error_path_function_with_generic_outputs_is_rejected() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    definition.blueprints.values_mut().for_each(|bp_def| {
        bp_def.schema.generics.push(GenericBound::Any);
        bp_def
            .schema
            .functions
            .functions
            .values_mut()
            .for_each(|def| def.output = TypeRef::Generic(0))
    });

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| format!("{error:?}").contains("Generics not supported"))
}

fn test_publishing_of_packages_with_invalid_names(name: &str) {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (code, mut definition) = PackageLoader::get("address");

    let (_, value) = definition.blueprints.pop().unwrap();
    definition.blueprints.insert(name.to_owned(), value);

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            definition,
            MetadataInit::default(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|error| {
        matches!(
            error,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidName { .. }
            ))
        )
    })
}

fn name(len: usize, chr: char) -> String {
    (0..len).map(|_| chr).collect()
}

#[test]
fn test_long_role_key() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let mut blueprints = index_map_new();
    blueprints.insert(
        "Test".to_string(),
        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set: indexset!(),
            dependencies: indexset!(),

            schema: BlueprintSchemaInit {
                generics: vec![],
                schema: VersionedScryptoSchema::from_latest_version(SchemaV1 {
                    type_kinds: vec![],
                    type_metadata: vec![],
                    type_validations: vec![],
                }),
                state: BlueprintStateSchemaInit {
                    fields: vec![FieldSchema::static_field(LocalTypeId::WellKnown(UNIT_TYPE))],
                    collections: vec![],
                },
                events: BlueprintEventSchemaInit::default(),
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit {
                    functions: indexmap!(
                        "f".to_string() => FunctionSchemaInit {
                            receiver: Option::Some(ReceiverInfo { receiver: Receiver::SelfRefMut, ref_types: RefTypes::NORMAL }),
                            input: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                            output: TypeRef::Static(LocalTypeId::WellKnown(ANY_TYPE)),
                            export: "Test_f".to_string(),
                        }
                    ),
                },
                hooks: BlueprintHooksInit::default(),
            },
            royalty_config: PackageRoyaltyConfig::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(StaticRoleDefinition {
                    roles: RoleSpecification::Normal(indexmap!(
                        RoleKey { key: "abc".to_owned() } => RoleList { list: vec![] }
                    )),
                    methods: indexmap!(
                        MethodKey { ident: "f".to_owned() } => MethodAccessibility::RoleProtected(
                            RoleList {
                                list: vec![RoleKey { key: format!("_{}", "a".repeat(1024)) }]
                            }
                        )
                    ),
                }),
            },
        },
    );

    // Act
    let code = wat2wasm(include_local_wasm_str!("basic_package.wat"));
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            PackageDefinition { blueprints },
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::ReservedRoleKeyIsNotDefined(_)
            ))
        )
    });
}
