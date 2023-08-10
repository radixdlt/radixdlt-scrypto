use crate::system::system::*;
use crate::types::*;

pub trait FieldContent: Sized {
    type VersionedContent: From<Self>;

    fn into_locked_substate(self) -> FieldSubstate<Self::VersionedContent> {
        FieldSubstate::new_locked_field(self.into())
    }

    fn into_mutable_substate(self) -> FieldSubstate<Self::VersionedContent> {
        FieldSubstate::new_field(self.into())
    }
}

pub trait KVEntryContent: Sized {
    type VersionedContent: From<Self>;

    fn into_locked_substate(self) -> KeyValueEntrySubstate<Self::VersionedContent> {
        KeyValueEntrySubstate::entry(self.into())
    }

    fn into_mutable_substate(self) -> KeyValueEntrySubstate<Self::VersionedContent> {
        KeyValueEntrySubstate::locked_entry(self.into())
    }
}

pub trait IndexEntryContent: Sized {
    type VersionedContent: From<Self>;

    fn into_substate(self) -> Self::VersionedContent {
        self.into()
    }
}

pub trait SortedIndexEntryContent: Sized {
    type VersionedContent: From<Self>;

    fn into_substate(self) -> Self::VersionedContent {
        self.into()
    }
}


/// Generates types and typed-interfaces for native blueprints and their
/// interaction with the substate store.
///
/// * For fields, assumes the existence of a type called:
///    `<BlueprintIdent><FieldIdent>FieldV1`
/// * For collections, assumes the existence of types called:
///    `<BlueprintIdent><CollectionIdent>Key`
///    `<BlueprintIdent><CollectionIdent>ValueV1`
/// 
/// In future, resolve the `x_type` fields as a $x:tt and then
/// map in other macros into:
/// ```
///     StaticSingleVersioned,
///     Static {
///         the_type: x,
///     },
///     Instance {
///         ident: 
///     },
///     StaticMultiVersioned(V1, V2),
/// ```
#[allow(unused)]
macro_rules! declare_native_blueprint_state {
    (
        blueprint_ident: $blueprint_ident:ident,
        blueprint_snake_case: $blueprint_property_name:ident,
        instance_schema_types: [
            // If no types => instance schema disabled
            $(
                $instance_type_property_name:ident: {
                    ident: $instance_type_ident:ident,
                }
            ),*
            $(,)?
        ],
        fields: {
            $(
                $field_property_name:ident: {
                    ident: $field_ident:ident,
                    field_type: StaticSingleVersioned,
                    condition: $field_condition:expr
                    $(,)? // Optional trialing comma
                }
            ),*
            $(,)? // Optional trialing comma
        },
        collections: {
            $(
                $collection_property_name:ident: $collection_type:ident {
                    entry_ident: $collection_ident:ident,
                    key_type: Static { // Will be Static { type: X } or Instance { index: X }
                        the_type: $collection_key_type:ty,
                    },
                    value_type: StaticSingleVersioned,
                    can_own: $collection_can_own:expr
                    // Collection options for (eg) passing in a property name
                    // of the sorted index parameter for SortedIndex
                    $(, options: $collection_options:tt)?
                    $(,)? // Optional trialing comma
                }
            ),*
            $(,)? // Optional trialing comma
        }
        $(,)?
    ) => {
        paste::paste! {
            pub use [<$blueprint_property_name _models>]::*;

            #[allow(unused_imports, dead_code, unused_mut, unused_assignments, unused_variables, unreachable_code)]
            mod [<$blueprint_property_name _models>] {
                use super::*;
                use sbor::*;
                use $crate::types::*;
                use $crate::errors::RuntimeError;
                use $crate::system::system::*;
                use radix_engine_interface::api::*;
                //--------------------------------------------------------
                // MODELS
                //--------------------------------------------------------

                // Generate models for each field
                $(
                    // TODO: In future, expand this macro to support multi-versioned fields
                    sbor::define_single_versioned!(
                        #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
                        pub enum [<Versioned $blueprint_ident $field_ident Field>] => [<$blueprint_ident $field_ident Field>] = [<$blueprint_ident $field_ident FieldV1>]
                    );
                    generate_wrapped_substate_type_alias!(Field, $blueprint_ident, $field_ident);
                    impl FieldContent for [<Versioned $blueprint_ident $field_ident Field>] {
                        type VersionedContent = Self;
                    }
                    impl FieldContent for [<$blueprint_ident $field_ident Field>] {
                        type VersionedContent = [<Versioned $blueprint_ident $field_ident Field>];
                    }
                );*

                // Generate models for each collection
                $(
                    pub type [<$blueprint_ident $collection_ident Key>] = $collection_key_type;
                    // TODO: In future, expand this macro to support multi-versioned collection values
                    sbor::define_single_versioned!(
                        #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
                        pub enum [<Versioned $blueprint_ident $collection_ident Value>] => [<$blueprint_ident $collection_ident Value>] = [<$blueprint_ident $collection_ident ValueV1>]
                    );
                    generate_wrapped_substate_type_alias!($collection_type, $blueprint_ident, $collection_ident);
                    generate_collection_substate_content_trait!($collection_type, [<Versioned $blueprint_ident $collection_ident Value>], Self);
                    generate_collection_substate_content_trait!($collection_type, [<$blueprint_ident $collection_ident Value>], [<Versioned $blueprint_ident $collection_ident Value>]);
                )*

                //--------------------------------------------------------
                // Node Layout (to replace node_layout.rs)
                //--------------------------------------------------------
                #[repr(u8)]
                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
                pub enum [<$blueprint_ident Field>] {
                    $($field_ident,)*
                }

                impl From<[<$blueprint_ident Field>]> for SubstateKey {
                    fn from(value: [<$blueprint_ident Field>]) -> Self {
                        SubstateKey::Field(value as u8)
                    }
                }

                impl From<[<$blueprint_ident Field>]> for u8 {
                    fn from(value: [<$blueprint_ident Field>]) -> Self {
                        value as u8
                    }
                }

                impl TryFrom<&SubstateKey> for [<$blueprint_ident Field>] {
                    type Error = ();

                    fn try_from(key: &SubstateKey) -> Result<Self, Self::Error> {
                        match key {
                            SubstateKey::Field(x) => Self::from_repr(*x).ok_or(()),
                            _ => Err(()),
                        }
                    }
                }

                impl TryFrom<u8> for [<$blueprint_ident Field>] {
                    type Error = ();

                    fn try_from(offset: u8) -> Result<Self, Self::Error> {
                        Self::from_repr(offset).ok_or(())
                    }
                }

                #[repr(u8)]
                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
                pub enum [<$blueprint_ident Partition>] {
                    Field,
                    $([<$collection_ident $collection_type>],)*
                }

                impl [<$blueprint_ident Partition>] {
                    pub const fn offset(&self) -> PartitionOffset {
                        PartitionOffset(*self as u8)
                    }

                    pub const fn main_partition(&self) -> PartitionNumber {
                        match MAIN_BASE_PARTITION.at_offset(self.offset()) {
                            // Work around .unwrap() on Option not being const
                            Some(x) => x,
                            None => panic!("Offset larger than allowed value")
                        }
                    }
                }

                impl From<[<$blueprint_ident Partition>]> for PartitionOffset {
                    fn from(value: [<$blueprint_ident Partition>]) -> Self {
                        PartitionOffset(value as u8)
                    }
                }

                //---------------------------------
                // Typed Substate - Keys and Values
                //---------------------------------
                #[derive(Debug, Clone)]
                pub enum [<$blueprint_ident TypedSubstateKey>] {
                    Fields([<$blueprint_ident Field>]),
                    $([<$collection_ident $collection_type Entries>]([<$blueprint_ident $collection_ident Key>]),)*
                }

                #[derive(Debug)]
                pub enum [<$blueprint_ident TypedFieldSubstateValue>] {
                    $($field_ident([<$blueprint_ident $field_ident FieldSubstate>]),)*
                }

                #[derive(Debug)]
                pub enum [<$blueprint_ident TypedSubstateValue>] {
                    Field([<$blueprint_ident TypedFieldSubstateValue>]),
                    $([<$collection_ident $collection_type>]([<$blueprint_ident $collection_ident EntrySubstate>]),)*
                }

                //----------------------
                // Schema
                //----------------------
                pub struct [<$blueprint_ident StateSchemaInit>];

                impl [<$blueprint_ident StateSchemaInit>] {
                    pub fn create_schema_init(
                        type_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>,
                    ) -> BlueprintStateSchemaInit {
                        let mut fields = vec![];
                        $(
                            // TODO - Implement instance schema
                            fields.push(FieldSchema {
                                field: TypeRef::Static(
                                    type_aggregator.add_child_type_and_descendents::<[<Versioned $blueprint_ident $field_ident Field>]>()
                                ),
                                condition: $field_condition,
                            });
                        )*
                        let mut collections = vec![];
                        $(
                            // TODO - Implement instance schema
                            collections.push(map_collection_schema!(
                                $collection_type,
                                type_aggregator,
                                $collection_key_type,
                                [<Versioned $blueprint_ident $collection_ident Value>],
                                $collection_can_own
                            ));
                        )*
                        BlueprintStateSchemaInit {
                            fields,
                            collections,
                        }
                    }
                }

                //----------------------
                // Object Initialization
                //----------------------

                /// Used for initializing blueprint state.
                ///
                /// Note - this doesn't support:
                /// * Features
                /// * Instance schemas
                /// * Feature-dependent fields
                /// * IndexEntries (because the underlying new_object API doesn't support them)
                pub struct [<$blueprint_ident StateInit>] {
                    $(
                        pub $field_property_name: [<$blueprint_ident $field_ident FieldSubstate>],
                    )*
                    $(
                        pub $collection_property_name: IndexMap<
                            [<$blueprint_ident $collection_ident Key>],
                            [<$blueprint_ident $collection_ident EntrySubstate>]
                        >,
                    )*
                }

                impl [<$blueprint_ident StateInit>] {
                    pub fn into_system_substates(self) -> (Vec<FieldValue>, BTreeMap<u8, BTreeMap<Vec<u8>, KVEntry>>) {
                        let mut field_values = vec![];
                        $(
                            let field_content = scrypto_encode(&self.$field_property_name.value).unwrap();
                            let locked = match &self.$field_property_name.mutability {
                                SubstateMutability::Mutable => true,
                                SubstateMutability::Immutable => false,
                            };
                            field_values.push(FieldValue {
                                value: field_content,
                                locked,
                            });
                        )*
                        let mut all_collection_entries = BTreeMap::new();
                        let mut collection_index: u8 = 0;
                        $(
                            let this_collection_entries = self.$collection_property_name
                                .into_iter()
                                .map(|(key, entry_substate)| {
                                    let key = scrypto_encode(&key).unwrap();
                                    let value = map_entry_substate_to_kv_entry!($collection_type, entry_substate);
                                    (key, value)
                                })
                                .collect();
                            all_collection_entries.insert(collection_index, this_collection_entries);
                            collection_index += 1;
                        )*
                        (field_values, all_collection_entries)
                    }

                    pub fn into_new_object<E, Y: ClientObjectApi<E>>(self, api: &mut Y) -> Result<NodeId, E> {
                        let (field_values, all_collection_entries) = self.into_system_substates();
                        api.new_object(
                            stringify!($blueprint_ident),
                            vec![], // Features
                            None, // Instance schema
                            field_values,
                            all_collection_entries,
                        )
                    }
                }

                //-------------
                // State API (TODO!)
                //-------------

                pub struct [<$blueprint_ident StateApi>]<'a, Y: ClientApi<RuntimeError>> {
                    api: &'a mut Y,
                }

                impl<'a, Y: ClientApi<RuntimeError>> [<$blueprint_ident StateApi>]<'a, Y> {
                    pub fn with(client_api: &'a mut Y) -> Self {
                        Self {
                            api: client_api,
                        }
                    }
                }

                impl<'a, Y: ClientApi<$crate::errors::RuntimeError>> From<&'a mut Y> for [<$blueprint_ident StateApi>]<'a, Y> {
                    fn from(value: &'a mut Y) -> Self {
                        Self::with(value)
                    }
                }
            }
        }
    }
}

#[allow(unused)]
pub(crate) use declare_native_blueprint_state;

pub(crate) use helper_macros::*;

mod helper_macros {
    macro_rules! generate_wrapped_substate_type_alias {
        (SystemField, $module_ident:ident, $field_ident:ident) => {
            paste::paste! {
                pub type [<$module_ident $field_ident FieldSubstate>] = [<Versioned $module_ident $field_ident Field>];
            }
        };
        (Field, $blueprint_ident:ident, $field_ident:ident) => {
            paste::paste! {
                pub type [<$blueprint_ident $field_ident FieldSubstate>] = $crate::system::system::FieldSubstate<[<Versioned $blueprint_ident $field_ident Field>]>;
            }
        };
        (KeyValue, $blueprint_ident:ident, $collection_ident:ident) => {
            paste::paste! {
                pub type [<$blueprint_ident $collection_ident EntrySubstate>] = $crate::system::system::KeyValueEntrySubstate<[<Versioned $blueprint_ident $collection_ident Value>]>;
            }
        };
        (Index, $blueprint_ident:ident, $collection_ident:ident) => {
            // No wrapper around Index substates
            paste::paste! {
                pub type [<$blueprint_ident $collection_ident EntrySubstate>] = [<Versioned $blueprint_ident $collection_ident Value>];
            }
        };
        (SortedIndex, $blueprint_ident:ident, $collection_ident:ident) => {
            // There is no wrapper around Index substates
            paste::paste! {
                pub type [<$blueprint_ident $collection_ident EntrySubstate>] = [<Versioned $blueprint_ident $collection_ident Value>];
            }
        };
        ($unknown_system_substate_type:ident, $blueprint_ident:ident, $collection_ident:ident) => {
            paste::paste! {
                compile_error!(concat!(
                    "Unrecognized system substate type: `",
                    stringify!($unknown_system_substate_type),
                    "` - expected `Field`, `SystemField`, `KeyValue`, `Index` or `SortedIndex`"
                ));
            }
        };
    }
    
    #[allow(unused)]
    pub(crate) use generate_wrapped_substate_type_alias;
    
    macro_rules! generate_collection_substate_content_trait {
        (KeyValue, $type:ident, $versioned:ident) => {
            impl KVEntryContent for $type {
                type VersionedContent = $versioned;
            }
        };
        (Index, $type:ident, $versioned:ident) => {
            impl IndexEntryContent for $type {
                type VersionedContent = $versioned;
            }
        };
        (SortedIndex, $type:ident, $versioned:ident) => {
            impl SortedIndexEntryContent for $type {
                type VersionedContent = $versioned;
            }
        };
        ($unknown_system_substate_type:ident, $type:ident, $versioned:ident) => {
            paste::paste! {
                compile_error!(concat!(
                    "Unrecognized system collection substate type: `",
                    stringify!($unknown_system_substate_type),
                    "` - expected `KeyValue`, `Index` or `SortedIndex`"
                ));
            }
        };
    }
    
    #[allow(unused)]
    pub(crate) use generate_collection_substate_content_trait;
    
    macro_rules! map_collection_schema {
        (KeyValue, $aggregator:ident, $collection_key_type:ty, $collection_value_type:ty, $collection_can_own:expr$(,)?) => {
            paste::paste! {
                BlueprintCollectionSchema::KeyValueStore(
                    BlueprintKeyValueSchema {
                        key: TypeRef::Static(
                            $aggregator.add_child_type_and_descendents::<$collection_key_type>(),
                        ),
                        value: TypeRef::Static(
                            $aggregator.add_child_type_and_descendents::<$collection_value_type>()
                        ),
                        can_own: $collection_can_own,
                    },
                )
            }
        };
        (Index, $aggregator:ident, $collection_key_type:ty, $collection_value_type:ty, $collection_can_own:expr$(,)?) => {
            BlueprintCollectionSchema::Index(
                BlueprintKeyValueSchema {
                    key: TypeRef::Static(
                        $aggregator.add_child_type_and_descendents::<$collection_key_type>(),
                    ),
                    value: TypeRef::Static(
                        $aggregator.add_child_type_and_descendents::<$collection_value_type>()
                    ),
                    can_own: $collection_can_own,
                },
            )
        };
        (SortedIndex, $aggregator:ident, $collection_key_type:ty, $collection_value_type:ty, $collection_can_own:expr$(,)?) => {
            BlueprintCollectionSchema::SortedIndex(
                BlueprintKeyValueSchema {
                    key: TypeRef::Static(
                        $aggregator.add_child_type_and_descendents::<$collection_key_type>(),
                    ),
                    value: TypeRef::Static(
                        $aggregator.add_child_type_and_descendents::<$collection_value_type>()
                    ),
                    can_own: $collection_can_own,
                },
            )
        };
        ($unknown_system_substate_type:ident, $aggregator:ident, $collection_key_type:ty, $collection_value_type:ty, $collection_can_own:expr$(,)?) => {
            paste::paste! {
                compile_error!(concat!(
                    "Unrecognized system collection substate type: `",
                    stringify!($unknown_system_substate_type),
                    "` - expected `KeyValue`, `Index` or `SortedIndex`"
                ));
            }
        };
    }
    
    #[allow(unused)]
    pub(crate) use map_collection_schema;
    
    macro_rules! map_entry_substate_to_kv_entry {
        (KeyValue, $entry_substate:ident) => {
            paste::paste! {
                KVEntry {
                    value: $entry_substate.value.map(|v| scrypto_encode(&v).unwrap()),
                    locked: match $entry_substate.mutability {
                        SubstateMutability::Mutable => true,
                        SubstateMutability::Immutable => false,
                    },
                }
            }
        };
        (Index, $entry_substate:ident) => {
            // This code still needs to compile, but it shouldn't be possible to execute
            panic!("Not possible to map an Index entry to a KVEntry")
        };
        (SortedIndex, $entry_substate:ident) => {
            // This code still needs to compile, but it shouldn't be possible to execute
            panic!("Not possible to map a SortedIndex entry to a KVEntry")
        };
        ($unknown_system_substate_type:ident, $entry_substate:ident) => {
            paste::paste! {
                compile_error!(concat!(
                    "Unrecognized system collection substate type: `",
                    stringify!($unknown_system_substate_type),
                    "` - expected `KeyValue`, `Index` or `SortedIndex`"
                ));
            }
        };
    }
    
    #[allow(unused)]
    pub(crate) use map_entry_substate_to_kv_entry;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check that the below compiles
    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct PackageRoyaltyFieldV1;
    
    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct PackageBlueprintDefinitionValueV1;
    
    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct PackageMyCoolIndexValueV1;
    
    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct PackageMyCoolSortedIndexValueV1;
    
    use radix_engine_interface::blueprints::package::*;
    
    declare_native_blueprint_state! {
        blueprint_ident: Package,
        blueprint_snake_case: package,
        instance_schema_types: [],
        fields: {
            royalty:  {
                ident: Royalty,
                field_type: StaticSingleVersioned,
                condition: Condition::Always,
            }
        },
        collections: {
            blueprint_definitions: KeyValue {
                entry_ident: BlueprintDefinition,
                key_type: Static {
                    the_type: BlueprintVersion,
                },
                value_type: StaticSingleVersioned,
                can_own: true,
            },
            abc: Index {
                entry_ident: MyCoolIndex,
                key_type: Static {
                    the_type: BlueprintVersion,
                },
                value_type: StaticSingleVersioned,
                can_own: true,
            },
            def: SortedIndex {
                entry_ident: MyCoolSortedIndex,
                key_type: Static {
                    the_type: BlueprintVersion,
                },
                value_type: StaticSingleVersioned,
                can_own: true,
            },
        }
    }    
}
