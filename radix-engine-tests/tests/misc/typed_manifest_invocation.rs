use crate::prelude::*;
use radix_blueprint_schema_init::RefTypes;
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_transactions::manifest::static_resource_movements::*;

/// This is a test to ensure that the [`TypedManifestNativeInvocation`] type has all the functions
/// that blueprints have. To be clear, we don't check that it has all of the blueprints but that for
/// the blueprints that it has that all the functions (a generic term we use for functions and
/// methods) are present in the type. This is because we use this type heavily in places like the
/// radix engine toolkit where we make the assumption that the type has all of the functions for
/// the blueprints it supports.
#[test]
fn test_that_all_functions_are_in_the_typed_invocation_type() {
    /* Arrange */
    // We will be using a latest ledger to obtain the set of functions that we currently have. This
    // is to ensure that this test is resistant to protocol updates and adding new functions to the
    // various blueprints we have.
    let ledger = LedgerSimulatorBuilder::new().without_kernel_trace().build();
    let db = ledger.substate_db();
    let ledger_function_table = FunctionTable::construct_from_substate_database(db);

    /* Act */
    // Constructing the function table of the typed manifest invocations type from the SBOR schema.
    let typed_manifest_invocations_function_table =
        FunctionTable::construct_from_typed_invocation_function_table();

    /* Assert */
    // We're taking the blueprint names in the typed invocations enum as the canonical set of
    // blueprints since we don't want to check that we support all blueprints, just all functions.
    for (blueprint_name, typed_native_invocation_blueprint_function_table) in
        typed_manifest_invocations_function_table.0.into_iter()
    {
        let Some(canonical_blueprint_function_table) =
            ledger_function_table.0.get(&blueprint_name).cloned()
        else {
            panic!(
                "Blueprint \"{blueprint_name:?}\" is in the typed manifest enum but not on ledger"
            );
        };

        for function_type in [
            FunctionType::Function,
            FunctionType::Method,
            FunctionType::DirectMethod,
        ] {
            let typed_invocations_blueprint_function_type_functions =
                typed_native_invocation_blueprint_function_table
                    .0
                    .get(&function_type);
            let canonical_blueprint_function_type_functions =
                canonical_blueprint_function_table.0.get(&function_type);

            match (
                typed_invocations_blueprint_function_type_functions,
                canonical_blueprint_function_type_functions,
            ) {
                (
                    Some(typed_invocations_blueprint_function_type_functions),
                    Some(canonical_blueprint_function_type_functions),
                ) => {
                    let functions_in_typed_invocation_but_not_in_ledger =
                        typed_invocations_blueprint_function_type_functions
                            .difference(canonical_blueprint_function_type_functions)
                            .collect::<HashSet<_>>();
                    let functions_in_ledger_but_not_in_typed_invocation =
                        canonical_blueprint_function_type_functions
                            .difference(typed_invocations_blueprint_function_type_functions)
                            .collect::<HashSet<_>>();

                    if !functions_in_typed_invocation_but_not_in_ledger.is_empty() {
                        panic!("For blueprint \"{blueprint_name:?}\" and function type \"{function_type:?}\" the following set of functions are defined in the typed invocation enum but are not present on ledger: {functions_in_typed_invocation_but_not_in_ledger:?}")
                    }
                    if !functions_in_ledger_but_not_in_typed_invocation.is_empty() {
                        panic!("For blueprint \"{blueprint_name:?}\" and function type \"{function_type:?}\" the following set of functions are defined in ledger but are not present in the typed invocations enum: {functions_in_ledger_but_not_in_typed_invocation:?}")
                    }
                }
                (Some(_), None) => {
                    panic!("Typed manifest invocation has {function_type:?} functions defined for {blueprint_name:?} but ledger doesnt")
                }
                (None, Some(_)) => {
                    panic!("Ledger has {function_type:?} functions defined for {blueprint_name:?} but typed invocation doesnt")
                }
                (None, None) => continue,
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct FunctionTable(HashMap<BlueprintName, BlueprintFunctionTable>);

impl FunctionTable {
    /// Constructs the function table of the [`TypedManifestNativeInvocation`] type based on its
    /// SBOR schema.
    fn construct_from_typed_invocation_function_table() -> Self {
        let function_table = typed_native_invocation_function_table();
        Self(
            function_table
                .into_iter()
                .map(|(blueprint_name, functions)| {
                    (
                        BlueprintName(blueprint_name.to_owned()),
                        BlueprintFunctionTable(
                            functions
                                .into_iter()
                                .filter_map(|(function_type, functions)| {
                                    if !functions.is_empty() {
                                        let function_type = match function_type {
                                            "Function" => FunctionType::Function,
                                            "Method" => FunctionType::Method,
                                            "DirectMethod" => FunctionType::DirectMethod,
                                            _ => unreachable!(),
                                        };
                                        Some((
                                            function_type,
                                            functions
                                                .into_iter()
                                                .map(ToOwned::to_owned)
                                                .map(FunctionIdent)
                                                .collect(),
                                        ))
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        ),
                    )
                })
                .collect(),
        )
    }

    fn construct_from_substate_database<S: SubstateDatabase + ListableSubstateDatabase>(
        database: &S,
    ) -> Self {
        let mut this = Self::default();
        let reader = SystemDatabaseReader::new(database);

        let packages = database
            .list_partition_keys()
            .map(|value| SpreadPrefixKeyMapper::from_db_node_key(&value.node_key))
            .filter_map(|node_id| PackageAddress::try_from(node_id.0.as_slice()).ok())
            .collect::<HashSet<_>>();

        for package_address in packages.into_iter() {
            for (BlueprintVersionKey { blueprint, .. }, blueprint_definition) in
                reader.get_package_definition(package_address).into_iter()
            {
                for (function_ident, FunctionSchema { receiver, .. }) in
                    blueprint_definition.interface.functions.into_iter()
                {
                    let function_type = match receiver {
                        Some(receiver) if receiver.ref_types.contains(RefTypes::DIRECT_ACCESS) => {
                            FunctionType::DirectMethod
                        }
                        Some(_) => FunctionType::Method,
                        None => FunctionType::Function,
                    };
                    this.0
                        .entry(BlueprintName(blueprint.clone()))
                        .or_default()
                        .0
                        .entry(function_type)
                        .or_default()
                        .insert(FunctionIdent(function_ident));
                }
            }
        }

        this
    }
}

#[derive(Clone, Debug, Default)]
struct BlueprintFunctionTable(HashMap<FunctionType, HashSet<FunctionIdent>>);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct FunctionIdent(String);

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct BlueprintName(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum FunctionType {
    Function,
    Method,
    DirectMethod,
}
