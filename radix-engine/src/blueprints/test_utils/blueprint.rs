use crate::errors::RuntimeError;
use crate::internal_prelude::*;

use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::test_utils::*;

pub struct TestUtilsBlueprint;

impl TestUtilsBlueprint {
    pub fn get_definition() -> BlueprintDefinitionInit {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let functions = indexmap! {
            TEST_UTILS_PANIC_IDENT.to_owned() => FunctionSchemaInit {
                receiver: None,
                input: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TestUtilsPanicInput>(),
                ),
                output: TypeRef::Static(
                    aggregator.add_child_type_and_descendents::<TestUtilsPanicOutput>(),
                ),
                export: TEST_UTILS_PANIC_IDENT.to_string(),
            }
        };
        let schema = generate_full_schema(aggregator);

        BlueprintDefinitionInit {
            blueprint_type: BlueprintType::default(),
            is_transient: false,
            feature_set: Default::default(),
            dependencies: Default::default(),
            schema: BlueprintSchemaInit {
                generics: Default::default(),
                schema,
                state: BlueprintStateSchemaInit {
                    fields: Default::default(),
                    collections: Default::default(),
                },
                events: BlueprintEventSchemaInit {
                    event_schema: Default::default(),
                },
                types: BlueprintTypeSchemaInit::default(),
                functions: BlueprintFunctionsSchemaInit { functions },
                hooks: BlueprintHooksInit {
                    hooks: Default::default(),
                },
            },
            royalty_config: Default::default(),
            auth_config: AuthConfig {
                function_auth: FunctionAuth::AllowAll,
                method_auth: MethodAuthTemplate::StaticRoleDefinition(roles_template!()),
            },
        }
    }

    pub fn panic<Y: SystemApi<RuntimeError>>(
        message: &str,
        _api: &mut Y,
    ) -> Result<(), RuntimeError> {
        panic!("{}", message);
    }
}
