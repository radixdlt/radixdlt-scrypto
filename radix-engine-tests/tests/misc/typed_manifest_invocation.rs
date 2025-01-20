use crate::prelude::*;
use radix_blueprint_schema_init::{ReceiverInfo, RefTypes};
use radix_engine::system::system_db_reader::SystemDatabaseReader;
use radix_rust::rust::ops::*;
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
    // Constructing the function table of typed manifest invocation from the Scrypto function table.
    let typed_manifest_invocations_function_table =
        FunctionTable::construct_from_typed_invocation_function_table();

    /* Assert */

    // From this point onward I will refer to all of the information obtained from the ledger db as
    // being "canonical" and I will refer to the typed manifest invocation as being "constructed".
    // This should make the variable names and everything else much shorter.
    //
    // Note: since we do not wish to test that all blueprints are there we will take the typed
    // manifest invocation's set of blueprints as being canonical.
    let canonical_function_table = ledger_function_table;
    let constructed_function_table = typed_manifest_invocations_function_table;

    for (blueprint_name, constructed_blueprint_function_table) in
        constructed_function_table.into_iter()
    {
        let Some(canonical_blueprint_function_table) =
            canonical_function_table.get(&blueprint_name).cloned()
        else {
            panic!("Blueprint \"{blueprint_name:?}\" not found in canonical function table.");
        };

        // We use the all constant on the function type since we want to visit all of the function
        // types and not just the ones seen in the canonical function table. This is to ensure that
        // the constructed type doesn't have any additional ones.
        for function_type in FunctionType::ALL {
            let constructed_function_type_functions =
                constructed_blueprint_function_table.get(&function_type);
            let canonical_function_type_functions =
                canonical_blueprint_function_table.get(&function_type);

            let constructed_function_type_functions_idents =
                constructed_blueprint_function_table.function_idents_set(&function_type);
            let canonical_function_type_functions_idents =
                canonical_blueprint_function_table.function_idents_set(&function_type);

            match (
                constructed_function_type_functions,
                canonical_function_type_functions,
            ) {
                (
                    Some(constructed_function_type_functions),
                    Some(canonical_function_type_functions),
                ) => {
                    // Checking that the functions in both the constructed and the canonical tables
                    // are present. We check that both sets are subsets of each other. If they're
                    // not then we panic.
                    let functions_in_constructed_but_not_in_canonical =
                        constructed_function_type_functions_idents
                            .difference(&canonical_function_type_functions_idents)
                            .collect::<HashSet<_>>();
                    let functions_in_canonical_but_not_in_constructed =
                        canonical_function_type_functions_idents
                            .difference(&constructed_function_type_functions_idents)
                            .collect::<HashSet<_>>();

                    if !functions_in_constructed_but_not_in_canonical.is_empty() {
                        panic!("Some functions for {blueprint_name:?}::{function_type} are in typed manifest invocation but not on-ledger: {:?}", functions_in_constructed_but_not_in_canonical)
                    }
                    if !functions_in_canonical_but_not_in_constructed.is_empty() {
                        panic!("Some functions for {blueprint_name:?}::{function_type} are on ledger but not typed manifest invocation: {:?}", functions_in_canonical_but_not_in_constructed)
                    }

                    // At this point we know that both types have the same set of functions. We can
                    // now iterate over the function list and compare their schemas to ensure that
                    // they're equal.
                    let function_idents = constructed_function_type_functions_idents;
                    for function_ident in function_idents.iter() {
                        let _constructed_function_schema = constructed_function_type_functions
                            .get(function_ident)
                            .unwrap();
                        let _canonical_function_schema = canonical_function_type_functions
                            .get(function_ident)
                            .unwrap();

                        // TODO: We can currently compare the schemas together since some of them
                        // use typed addresses (references in particular) and others do not.
                        // compare_single_type_schemas(
                        //     &SchemaComparisonSettings::require_equality()
                        //         .with_metadata(|metadata| {
                        //             metadata.with_type_name_changes(NameChangeRule::AllowAllChanges)
                        //         })
                        //         .with_completeness(|completeness| {
                        //             completeness
                        //                 .with_allow_root_unreachable_types_in_base_schema()
                        //                 .with_allow_root_unreachable_types_in_compared_schema()
                        //         }),
                        //     constructed_function_schema,
                        //     canonical_function_schema,
                        // )
                        // .assert_valid("Typed Manifest Invocation", "Blueprint Definition");
                    }
                }
                (Some(v), None) if !v.is_empty() => {
                    panic!("{blueprint_name:?}::{function_type} is defined in typed manifest invocation but not on ledger")
                }
                (None, Some(v)) if !v.is_empty() => {
                    panic!("{blueprint_name:?}::{function_type} is defined on ledger but not in typed manifest invocation")
                }
                (Some(_), None) | (None, Some(_)) | (None, None) => continue,
            }
        }
    }
}

mod test_types {
    use super::*;
    use radix_rust::rust::collections::hash_map::Entry;

    #[derive(Clone, Debug, Default)]
    pub struct FunctionTable(HashMap<BlueprintName, BlueprintFunctionTable>);

    impl IntoIterator for FunctionTable {
        type Item = <HashMap<BlueprintName, BlueprintFunctionTable> as IntoIterator>::Item;
        type IntoIter = <HashMap<BlueprintName, BlueprintFunctionTable> as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    impl FunctionTable {
        pub fn get(&self, key: &BlueprintName) -> Option<&BlueprintFunctionTable> {
            self.0.get(key)
        }

        pub fn entry(
            &mut self,
            key: BlueprintName,
        ) -> Entry<'_, BlueprintName, BlueprintFunctionTable> {
            self.0.entry(key)
        }

        /// Constructs the function table of the [`TypedManifestNativeInvocation`] typed based on the
        /// function table defined by the macro.
        pub fn construct_from_typed_invocation_function_table() -> Self {
            let mut this = Self::default();

            let function_table = typed_native_invocation_function_table();
            for (blueprint_name, functions_map) in function_table.into_iter() {
                let blueprint_name = BlueprintName(blueprint_name.to_owned());

                for (function_type, functions_map) in functions_map.into_iter() {
                    let function_type = FunctionType::from_str(function_type).unwrap();

                    for (function_ident, input_schema) in functions_map.into_iter() {
                        let function_ident = FunctionIdent(function_ident.to_owned());

                        this.entry(blueprint_name.clone())
                            .or_default()
                            .entry(function_type)
                            .or_default()
                            .insert(function_ident, input_schema);
                    }
                }
            }

            this
        }

        pub fn construct_from_substate_database<S: SubstateDatabase + ListableSubstateDatabase>(
            database: &S,
        ) -> Self {
            let mut this = Self::default();
            let reader = SystemDatabaseReader::new(database);

            // Getting all of the packages on ledger.
            let packages = database
                .list_partition_keys()
                .map(|value| SpreadPrefixKeyMapper::from_db_node_key(&value.node_key))
                .filter_map(|node_id| PackageAddress::try_from(node_id.0.as_slice()).ok())
                .collect::<HashSet<_>>();

            // Constructing the function table from all of the packages on-ledger
            for package_address in packages.into_iter() {
                for (BlueprintVersionKey { blueprint, .. }, blueprint_definition) in
                    reader.get_package_definition(package_address).into_iter()
                {
                    let blueprint_name = BlueprintName(blueprint);

                    for (
                        function_ident,
                        FunctionSchema {
                            receiver, input, ..
                        },
                    ) in blueprint_definition.interface.functions.into_iter()
                    {
                        let function_type = FunctionType::from(receiver);
                        let function_ident = FunctionIdent(function_ident);

                        let BlueprintPayloadDef::Static(ScopedTypeId(schema_hash, type_id)) = input
                        else {
                            unreachable!()
                        };
                        let schema = reader
                            .get_schema(package_address.as_node_id(), &schema_hash)
                            .unwrap()
                            .deref()
                            .clone();

                        let single_type_schema = SingleTypeSchema { schema, type_id };

                        this.entry(blueprint_name.clone())
                            .or_default()
                            .entry(function_type)
                            .or_default()
                            .insert(function_ident, single_type_schema);
                    }
                }
            }

            this
        }
    }

    #[derive(Clone, Debug, Default)]
    pub struct BlueprintFunctionTable(
        HashMap<FunctionType, HashMap<FunctionIdent, SingleTypeSchema<ScryptoCustomSchema>>>,
    );

    impl BlueprintFunctionTable {
        pub fn get(
            &self,
            key: &FunctionType,
        ) -> Option<&HashMap<FunctionIdent, SingleTypeSchema<ScryptoCustomSchema>>> {
            self.0.get(key)
        }

        pub fn entry(
            &mut self,
            key: FunctionType,
        ) -> Entry<'_, FunctionType, HashMap<FunctionIdent, SingleTypeSchema<ScryptoCustomSchema>>>
        {
            self.0.entry(key)
        }

        pub fn function_idents_set(&self, function_type: &FunctionType) -> HashSet<&FunctionIdent> {
            self.0
                .get(function_type)
                .into_iter()
                .flat_map(|value| value.keys())
                .collect()
        }
    }

    impl IntoIterator for BlueprintFunctionTable {
        type Item = <HashMap<
            FunctionType,
            HashMap<FunctionIdent, SingleTypeSchema<ScryptoCustomSchema>>,
        > as IntoIterator>::Item;
        type IntoIter = <HashMap<
            FunctionType,
            HashMap<FunctionIdent, SingleTypeSchema<ScryptoCustomSchema>>,
        > as IntoIterator>::IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            self.0.into_iter()
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct FunctionIdent(String);

    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct BlueprintName(String);

    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum FunctionType {
        Function,
        Method,
        DirectMethod,
    }

    impl FunctionType {
        pub const ALL: [Self; 3] = [Self::Function, Self::Method, Self::DirectMethod];
    }

    impl Display for FunctionType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            Debug::fmt(&self, f)
        }
    }

    impl FromStr for FunctionType {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "Function" => Ok(Self::Function),
                "Method" => Ok(Self::Method),
                "DirectMethod" => Ok(Self::DirectMethod),
                _ => Err(()),
            }
        }
    }

    impl From<Option<ReceiverInfo>> for FunctionType {
        fn from(value: Option<ReceiverInfo>) -> Self {
            match value {
                Some(value) if value.ref_types.contains(RefTypes::DIRECT_ACCESS) => {
                    FunctionType::DirectMethod
                }
                Some(_) => FunctionType::Method,
                None => FunctionType::Function,
            }
        }
    }
}
use test_types::*;
