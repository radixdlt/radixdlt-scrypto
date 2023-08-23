use crate::internal_prelude::*;

/// Generates types and typed-interfaces for native blueprints, their
/// state models, features, and schemas.
///
/// See the below structure for detail on how it should look - or check
/// out [../package/substates.rs](the package substates definition).
///
/// For each field, the following types will be created:
/// * `<BlueprintIdent><FieldIdent>FieldPayload` - a transparent new type for the field content
/// * `<BlueprintIdent><FieldIdent>FieldSubstate` - a type for the full system-wrapped substate
///
/// For each collection value, the following types will be created:
/// * `<BlueprintIdent><CollectionIdent>EntryPayload` - a transparent new type for the entry content
/// * `<BlueprintIdent><CollectionIdent>EntrySubstate` - a type for the full system-wrapped substate
///
/// For each collection key, the following types will be created:
/// * `<BlueprintIdent><CollectionIdent>KeyPayload` - a new type for the key payload (eg includes the u16 for a sorted index key)
///
/// The content of each of the above can take a number of forms. This is configured via specifying the type as one of the following.
/// Only Static is supported for keys at present. By default, you should choose StaticSingleVersioned for fields and collection values.
/// ```
///     {
///         kind: StaticSingleVersioned,
///     }
///     {
///         kind: Static,
///         content_type: x,
///     },
///     {
///         kind: Generic,
///         ident: BlueprintGenericParameterIdent,
///     },
///     // In future
///     {
///         kind: StaticMultiVersioned,
///         previous_versions: [V1, V2],
///         latest: V3,
///     }
/// ```
///
/// Choosing  `StaticSingleVersioned`, which will create a
/// forward-compatible enum wrapper with a single version for the content.
/// For Fields, it will assume the existence of a type called
/// `<BlueprintIdent><FieldIdent>V1` and will generate the following types:
/// * `<BlueprintIdent><FieldIdent>` - a type alias for the latest version (V1).
/// * `Versioned<BlueprintIdent><FieldIdent>` - the enum wrapper with a single version. This will be the content of `<BlueprintIdent><FieldIdent>FieldPayload`.
///
/// For collection values, it will assume the existence of `<BlueprintIdent><CollectionIdent>V1`
/// and generate the following types:
/// * `<BlueprintIdent><CollectionIdent>` - a type alias for the latest version (V1).
/// * `Versioned<BlueprintIdent><CollectionIdent>` - the enum wrapper with a single version. This will be the content of `<BlueprintIdent><CollectionIdent>EntryPayload`.
#[allow(unused)]
macro_rules! declare_native_blueprint_state {
    (
        blueprint_ident: $blueprint_ident:ident,
        blueprint_snake_case: $blueprint_property_name:ident,
        $(
            outer_blueprint: {
                ident: $outer_blueprint_ident:ident
                $(,)?
            },
        )?
        $(
            generics: {
                $(
                    $generic_property_name:ident: {
                        ident: $generic_ident:ident,
                        description: $generic_description:expr
                        $(,)?
                    }
                ),*
                $(,)?
            },
        )?
        features: {
            $(
                $feature_property_name:ident: {
                    ident: $feature_ident:ident,
                    description: $feature_description:expr,
                }
            ),*
            $(,)?
        },
        fields: {
            $(
                $field_property_name:ident: {
                    ident: $field_ident:ident,
                    field_type: $field_type:tt
                    $(, condition: $field_condition:expr)?
                    $(, transience: $field_transience:expr)?
                    $(,)? // Optional trailing comma
                }
            ),*
            $(,)? // Optional trailing comma
        },
        collections: {
            $(
                $collection_property_name:ident: $collection_type:ident {
                    entry_ident: $collection_ident:ident,
                    $(mapped_physical_partition: $mapped_physical_partition:expr,)?
                    key_type: $collection_key_type:tt,
                    // The full_key_content is required if it's a sorted index
                    $(full_key_content: $full_key_content:tt,)?
                    value_type: $collection_value_type:tt,
                    allow_ownership: $collection_allow_ownership:expr
                    $(,)? // Optional trailing comma
                }
            ),*
            $(,)? // Optional trailing comma
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
                use $crate::track::interface::*;
                use $crate::errors::*;
                use $crate::system::system::*;
                use radix_engine_interface::api::*;
                //--------------------------------------------------------
                // MODELS
                //--------------------------------------------------------

                // Generate models for each field
                $(
                    // Value
                    // > Set up Versioned types (if relevant). Assumes __FieldV1 exists and then creates
                    //   - Versioned__Field
                    //   - __Field (alias for __FieldV1)
                    // > Set up the (transparent) _FieldPayload new type for the content of the field
                    // > Set up the FieldContent trait for anything which can be resolved into the field payload
                    generate_content_type!(
                        content_trait: FieldContentSource,
                        payload_trait: FieldPayload,
                        ident_core: [<$blueprint_ident $field_ident>],
                        #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
                        struct [<$blueprint_ident $field_ident FieldPayload>] = $field_type
                    );

                    // > Set up the _FieldSubstate alias for the system-wrapped substate
                    generate_system_substate_type_alias!(
                        Field,
                        type [<$blueprint_ident $field_ident FieldSubstate>] = WRAPPED [<$blueprint_ident $field_ident FieldPayload>]
                    );
                )*

                // Generate models for each collection
                $(
                    // Key
                    generate_key_type!(
                        content_trait: [<$collection_type KeyContentSource>],
                        payload_trait: [<$collection_type KeyPayload>],
                        $(full_key_content: $full_key_content,)?
                        #[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, ScryptoSbor)]
                        struct [<$blueprint_ident $collection_ident KeyPayload>] = $collection_key_type
                    );

                    // Values
                    // > If relevant, set up Versioned types, which:
                    //   - Assumes [BlueprintCollection]V1 exists
                    //   - Creates Versioned[BlueprintCollection] enum
                    //   - Creates [BlueprintCollection] as a "latest" type alias for [BlueprintCollection]V1
                    // > Set up the [BlueprintCollection]EntryPayload transparent new type for the value content
                    // > Set up the [Collectiontype]EntryContent::<[BlueprintCollection]EntryPayload> trait for:
                    //   - The [BlueprintCollection] if it exists
                    //   - The Versioned[BlueprintCollection] if it exists
                    //   - The static content type, if it exists
                    generate_content_type!(
                        content_trait: [<$collection_type EntryContentSource>],
                        payload_trait: [<$collection_type EntryPayload>],
                        ident_core: [<$blueprint_ident $collection_ident>],
                        #[derive(Debug, PartialEq, Eq, ScryptoSbor)]
                        struct [<$blueprint_ident $collection_ident EntryPayload>] = $collection_value_type
                    );
                    // > Set up the _EntrySubstate alias for the system-wrapped substate
                    generate_system_substate_type_alias!(
                        $collection_type,
                        type [<$blueprint_ident $collection_ident EntrySubstate>] = WRAPPED [<$blueprint_ident $collection_ident EntryPayload>]
                    );
                )*

                //-------------------------------------
                // System - Generate schema definitions
                //-------------------------------------
                pub struct [<$blueprint_ident StateSchemaInit>];

                impl [<$blueprint_ident StateSchemaInit>] {
                    pub fn create_schema_init(
                        type_aggregator: &mut TypeAggregator<ScryptoCustomTypeKind>,
                    ) -> BlueprintStateSchemaInit {
                        let mut fields = vec![];
                        $(
                            fields.push(FieldSchema {
                                field: map_type_ref!(
                                    $blueprint_ident,
                                    type_aggregator,
                                    $field_type,
                                    [<$blueprint_ident $field_ident FieldPayload>],
                                ),
                                condition: optional_or_fallback!($({ $field_condition })?, { Condition::Always }),
                                transience: optional_or_fallback!($({ $field_transience})?, { FieldTransience::NotTransient }),
                            });
                        )*
                        let mut collections = vec![];
                        $(
                            collections.push(map_collection_schema!(
                                $collection_type,
                                $blueprint_ident,
                                type_aggregator,
                                $collection_key_type,
                                [<$blueprint_ident $collection_ident KeyPayload>],
                                $collection_value_type,
                                [<$blueprint_ident $collection_ident EntryPayload>],
                                $collection_allow_ownership
                            ));
                        )*
                        BlueprintStateSchemaInit {
                            fields,
                            collections,
                        }
                    }
                }

                //--------------------------------------------------------
                // System - Fields, Collections, Features and Generics
                //--------------------------------------------------------
                if_exists!(
                    TEST: [[$($field_ident)*]],
                    // Avoid https://doc.rust-lang.org/error_codes/E0084.html if no fields exist
                    [[
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

                        impl FieldDescriptor for [<$blueprint_ident Field>] {
                            fn field_index(&self) -> FieldIndex {
                                *self as u8
                            }
                        }
                    ]],
                    [[
                        #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
                        pub enum [<$blueprint_ident Field>] {}

                        impl FieldDescriptor for [<$blueprint_ident Field>] {
                            fn field_index(&self) -> FieldIndex {
                                unreachable!("No fields exist")
                            }
                        }
                    ]],
                );

                if_exists!(
                    TEST: [[$($collection_ident)*]],
                    [[
                        #[repr(u8)]
                        #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord, FromRepr)]
                        pub enum [<$blueprint_ident Collection>] {
                            $([<$collection_ident $collection_type>],)*
                        }

                        impl CollectionDescriptor for [<$blueprint_ident Collection>] {
                            fn collection_index(&self) -> CollectionIndex {
                                *self as u8
                            }
                        }
                    ]],
                    [[
                        #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, PartialOrd, Ord)]
                        pub enum [<$blueprint_ident Collection>] {}

                        impl CollectionDescriptor for [<$blueprint_ident Collection>] {
                            fn collection_index(&self) -> CollectionIndex {
                                unreachable!("No collections exist")
                            }
                        }
                    ]]
                );

                $(
                    #[repr(u8)]
                    #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, FromRepr)]
                    pub enum [<$blueprint_ident Generic>] {
                        $($generic_ident,)*
                    }

                    impl [<$blueprint_ident Generic>] {
                        pub const fn generic_index(&self) -> u8 {
                            *self as u8
                        }
                    }
                )?

                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash)]
                pub enum [<$blueprint_ident Feature>] {
                    $($feature_ident,)*
                }

                impl BlueprintFeature for [<$blueprint_ident Feature>] {
                    fn feature_name(&self) -> &'static str {
                        match *self {
                            $(
                                Self::$feature_ident => stringify!($feature_property_name),
                            )*
                        }
                    }
                }

                #[derive(Debug, Clone, Copy, Sbor, PartialEq, Eq, Hash, Default)]
                pub struct [<$blueprint_ident FeatureSet>] {
                    $(pub [<$feature_property_name>]: bool,)*
                }

                impl [<$blueprint_ident FeatureSet>] {
                    pub fn all_features() -> BTreeSet<String> {
                        let mut features = BTreeSet::new();
                        $(
                            features.insert(
                                [<$blueprint_ident Feature>]::$feature_ident.feature_name().to_string()
                            );
                        )*
                        features
                    }
                }

                impl HasFeatures for [<$blueprint_ident FeatureSet>] {
                    fn feature_names_str(&self) -> Vec<&'static str> {
                        let mut names = vec![];
                        $(
                            if self.[<$feature_property_name>] {
                                names.push([<$blueprint_ident Feature>]::$feature_ident.feature_name());
                            }
                        )*
                        names
                    }
                }

                //---------------------------------
                // Typed - Substate Keys and Values
                //---------------------------------

                enum_filter_out_ignored!(
                    /// All the SubstateKeys for all logical partitions for the $blueprint_ident blueprint.
                    /// Does not include mapped partitions, as these substates are mapped via their canonical partition.
                    #[derive(Debug, Clone)]
                    pub enum [<$blueprint_ident TypedSubstateKey>]
                    {
                        [[
                            Field([<$blueprint_ident Field>])
                        ]],
                        $(
                            $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                            [[
                                [<$collection_ident $collection_type Entry>]([<$blueprint_ident $collection_ident KeyPayload>])
                            ]],
                        )*
                    }
                );

                impl [<$blueprint_ident TypedSubstateKey>] {
                    pub fn for_key_in_partition(partition: &[<$blueprint_ident PartitionOffset>], substate_key: &SubstateKey) -> Result<Self, ()> {
                        let key = match_filter_out_ignored!(match partition {
                            [[
                                [<$blueprint_ident PartitionOffset>]::Field => {
                                    [<$blueprint_ident TypedSubstateKey>]::Field(
                                        [<$blueprint_ident Field>]::try_from(substate_key)?
                                    )
                                }
                            ]],
                            $(
                                $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                                [[
                                    [<$blueprint_ident PartitionOffset>]::[<$collection_ident $collection_type>] => {
                                        [<$blueprint_ident TypedSubstateKey>]::[<$collection_ident $collection_type Entry>](
                                            [<$blueprint_ident $collection_ident KeyPayload>]::try_from(substate_key)?,
                                        )
                                    }
                                ]],
                            )*
                        });
                        Ok(key)
                    }
                }

                #[derive(Debug)]
                pub enum [<$blueprint_ident TypedFieldSubstateValue>] {
                    $($field_ident([<$blueprint_ident $field_ident FieldSubstate>]),)*
                }

                enum_filter_out_ignored!(
                    /// All the Substate values for all logical partitions for the $blueprint_ident blueprint.
                    /// Does not include mapped partitions, as these substates are mapped via their canonical partition.
                    #[derive(Debug)]
                    pub enum [<$blueprint_ident TypedSubstateValue>]
                    {
                        [[
                            Field([<$blueprint_ident TypedFieldSubstateValue>])
                        ]],
                        $(
                            $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                            [[
                                [<$collection_ident $collection_type>]([<$blueprint_ident $collection_ident EntrySubstate>])
                            ]],
                        )*
                    }
                );

                impl [<$blueprint_ident TypedSubstateValue>] {
                    pub fn from_key_and_data(key: &[<$blueprint_ident TypedSubstateKey>], data: &[u8]) -> Result<Self, DecodeError> {
                        let substate_value = match_filter_out_ignored!(match key {
                            $(
                                [[
                                    [<$blueprint_ident TypedSubstateKey>]::Field([<$blueprint_ident Field>]::$field_ident) => {
                                        [<$blueprint_ident TypedSubstateValue>]::Field(
                                            [<$blueprint_ident TypedFieldSubstateValue>]::$field_ident(scrypto_decode(data)?)
                                        )
                                    }
                                ]],
                            )*
                            $(
                                $(|IGNORE_ENTRY| { $mapped_physical_partition })?
                                [[
                                    [<$blueprint_ident TypedSubstateKey>]::[<$collection_ident $collection_type Entry>](_) => {
                                        [<$blueprint_ident TypedSubstateValue>]::[<$collection_ident $collection_type>](
                                            scrypto_decode(data)?
                                        )
                                    }
                                ]],
                            )*
                        });
                        Ok(substate_value)
                    }
                }
            }
        }
    }
}

#[allow(unused)]
pub(crate) use declare_native_blueprint_state;

pub(crate) use helper_macros::*;

#[allow(unused_macros)]
mod helper_macros {
    macro_rules! ignore_arg {
        ($($ignored:tt)*) => {};
    }
    #[allow(unused)]
    pub(crate) use ignore_arg;

    macro_rules! generate_content_type {
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: StaticSingleVersioned
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                sbor::define_single_versioned!(
                    $(#[$attributes])*
                    pub enum [<Versioned $ident_core>] => $ident_core = [<$ident_core V1>]
                );
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    $(#[$attributes])*
                    pub struct $payload_type_name([<Versioned $ident_core>]);
                );

                impl HasLatestVersion for $payload_type_name
                {
                    type Latest = <[<Versioned $ident_core>] as HasLatestVersion>::Latest;
                    fn into_latest(self) -> Self::Latest {
                        self.into_content().into_latest()
                    }

                    fn as_latest_ref(&self) -> Option<&Self::Latest> {
                        self.as_ref().as_latest_ref()
                    }
                }

                // Now implement other relevant content traits, for:
                // > The "latest" type: $ident_core
                impl $content_trait<$payload_type_name> for $ident_core {
                    fn into_content(self) -> [<Versioned $ident_core>] {
                        self.into()
                    }
                }
            }
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Static,
                content_type: $static_type:ty
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    $(#[$attributes])*
                    pub struct $payload_type_name($static_type);
                );
            }
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            ident_core: $ident_core:ident,
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Generic,
                ident: $generic_ident:ident
                $(,)?
            }
        ) => {
            paste::paste! {
                declare_payload_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    $(#[$attributes])*
                    pub struct $payload_type_name<$generic_ident: [<$ident_core ContentMarker>] = ScryptoValue>($generic_ident);
                );
                // We choose to create an explicit marker trait, as an alternative to a blanket impl
                // over ScryptoEncode + ScryptoDecode. Any explicit types can implement this trait.
                // This avoids every type getting implementations for every such generic type,
                // which would require disambiguation everywhere `to_substate()` is used.
                // Anyone needing a type implementing content can use the payload type itself
                pub trait [<$ident_core ContentMarker>]: ScryptoEncode + ScryptoDecode {}
                impl [<$ident_core ContentMarker>] for ScryptoValue {}
                impl [<$ident_core ContentMarker>] for RawScryptoValue<'_> {}
            }
        };
        // TODO - Add support for some kind of StaticMultiVersioned type here
    }

    #[allow(unused)]
    pub(crate) use generate_content_type;

    macro_rules! generate_key_type {
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            $(full_key_content: $full_key_content:tt,)?
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: StaticSingleVersioned
                $(,)?
            }$(,)?
        ) => {
            compile_error!(
                "A StaticSingleVersioned key is not supported, because keys cannot be lazily updated, because they need to be static"
            );
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            $(full_key_content: $full_key_content:tt,)?
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Static,
                content_type: $static_type:ty
                $(,)?
            }$(,)?
        ) => {
            paste::paste! {
                declare_key_new_type!(
                    content_trait: $content_trait,
                    payload_trait: $payload_trait,
                    $(full_key_content: $full_key_content,)?
                    $(#[$attributes])*
                    pub struct $payload_type_name($static_type);
                );
            }
        };
        (
            content_trait: $content_trait:ident,
            payload_trait: $payload_trait:ident,
            $(full_key_content: $full_key_content:tt,)?
            $(#[$attributes:meta])*
            struct $payload_type_name:ident = {
                kind: Generic,
                ident: $generic_ident:ident
                $(,)?
            }
        ) => {
            paste::paste! {
                compile_error!(
                    "A Generic key is not currently supported by these macros"
                );
            }
        };
    }

    #[allow(unused)]
    pub(crate) use generate_key_type;

    macro_rules! generate_system_substate_type_alias {
        (SystemField, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around SystemField substates
            pub type $alias = $content;
        };
        (Field, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            pub type $alias = FieldSubstate<$content>;
        };
        (KeyValue, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            pub type $alias = KeyValueEntrySubstate<$content>;
        };
        (Index, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around Index substates
            pub type $alias = $content;
        };
        (SortedIndex, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            // There is no system wrapper around SortedIndex substates
            pub type $alias = $content;
        };
        ($unknown_system_substate_type:ident, type $alias:ident = WRAPPED $content:ty$(,)?) => {
            compile_error!(concat!(
                "Unrecognized system substate type: `",
                stringify!($unknown_system_substate_type),
                "` - expected `Field`, `SystemField`, `KeyValue`, `Index` or `SortedIndex`"
            ));
        };
    }

    #[allow(unused)]
    pub(crate) use generate_system_substate_type_alias;

    macro_rules! map_collection_schema {
        (KeyValue, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
            BlueprintCollectionSchema::KeyValueStore(BlueprintKeyValueSchema {
                key: map_type_ref!($blueprint_ident, $aggregator, $key_type, $key_payload_alias),
                value: map_type_ref!(
                    $blueprint_ident,
                    $aggregator,
                    $value_type,
                    $value_payload_alias
                ),
                allow_ownership: $allow_ownership,
            })
        };
        (Index, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
            BlueprintCollectionSchema::Index(BlueprintKeyValueSchema {
                key: map_type_ref!($blueprint_ident, $aggregator, $key_type, $key_payload_alias),
                value: map_type_ref!(
                    $blueprint_ident,
                    $aggregator,
                    $value_type,
                    $value_payload_alias
                ),
                allow_ownership: $allow_ownership,
            })
        };
        (SortedIndex, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
            BlueprintCollectionSchema::SortedIndex(BlueprintKeyValueSchema {
                key: map_type_ref!($blueprint_ident, $aggregator, $key_type, $key_payload_alias),
                value: map_type_ref!(
                    $blueprint_ident,
                    $aggregator,
                    $value_type,
                    $value_payload_alias
                ),
                allow_ownership: $allow_ownership,
            })
        };
        ($unknown_system_substate_type:ident, $blueprint_ident:ident, $aggregator:ident, $key_type:tt, $key_payload_alias:ident, $value_type:tt, $value_payload_alias:ident, $allow_ownership:expr$(,)?) => {
            compile_error!(concat!(
                "Unrecognized system collection substate type: `",
                stringify!($unknown_system_substate_type),
                "` - expected `KeyValue`, `Index` or `SortedIndex`"
            ));
        };
    }

    #[allow(unused)]
    pub(crate) use map_collection_schema;

    macro_rules! map_type_ref {
        (
            $blueprint_ident:ident,
            $aggregator:ident,
            {
                kind: StaticSingleVersioned
                $(,)?
            },
            $payload_alias:ident$(,)?
        ) => {
            TypeRef::Static($aggregator.add_child_type_and_descendents::<$payload_alias>())
        };
        (
            $blueprint_ident:ident,
            $aggregator:ident,
            {
                kind: Static,
                content_type: $static_type:ty
                $(,)?
            },
            $payload_alias:ident$(,)?
        ) => {
            TypeRef::Static($aggregator.add_child_type_and_descendents::<$payload_alias>())
        };
        (
            $blueprint_ident:ident,
            $aggregator:ident,
            {
                kind: Generic,
                ident: $generic_ident:ident
                $(,)?
            },
            $payload_alias:ident$(,)?
        ) => {
            paste::paste! {
                TypeRef::Generic([<$blueprint_ident Generic>]::$generic_ident.generic_index())
            }
        }; // TODO - Add support for some kind of StaticMultiVersioned type here
    }

    #[allow(unused)]
    pub(crate) use map_type_ref;

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

    macro_rules! optional_or_fallback {
        ($value:tt, $fallback:tt$(,)?) => {
            $value
        };
        (, $fallback:tt$(,)?) => {
            $fallback
        };
    }

    #[allow(unused)]
    pub(crate) use optional_or_fallback;

    macro_rules! enum_filter_out_ignored {
        (
            $(#[$attributes:meta])*
            pub enum $enum_name:ident
            {$(
                $(|IGNORE_ENTRY| { $($misc:tt)* } [[ $($ignored:tt)* ]])?
                $([[ $($present:tt)* ]])?
                ,
            )*}
        ) => {
            $(#[$attributes])*
            pub enum $enum_name
            {$(
                $($($present)*,)?
            )*}
        };
    }
    #[allow(unused)]
    pub(crate) use enum_filter_out_ignored;

    macro_rules! match_filter_out_ignored {
        (
            match $value_to_match:ident {$(
                $(|IGNORE_ENTRY| { $($misc:tt)*} [[ $($ignored:tt)* ]])?
                $([[ $($present:tt)* ]])?
                ,
            )*}
        ) => {
            match $value_to_match {$(
                $($($present)*)?
            )*}
        };
    }
    #[allow(unused)]
    pub(crate) use match_filter_out_ignored;

    macro_rules! if_exists {
        (
            TEST: [[]],
            [[ $($present:tt)* ]],
            [[ $($not_present:tt)* ]]$(,)?
        ) => {
            $($not_present)*
        };
        (
            TEST: [[ $($exists:tt)* ]],
            [[ $($present:tt)* ]],
            [[ $($not_present:tt)* ]]$(,)?
        ) => {
            $($present)*
        };
    }
    #[allow(unused)]
    pub(crate) use if_exists;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check that the below compiles
    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintRoyaltyV1;

    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintMyCoolKeyValueStoreV1;

    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintMyCoolIndexV1;

    #[derive(Debug, PartialEq, Eq, Sbor)]
    pub struct TestBlueprintMyCoolSortedIndexV1;

    use radix_engine_interface::blueprints::package::*;

    #[allow(dead_code)]
    pub enum TestBlueprintPartitionOffset {
        Field,
        MyCoolKeyValueStoreKeyValue,
        MyCoolIndexIndex,
        MyCoolSortedIndexSortedIndex,
    }

    declare_native_blueprint_state! {
        blueprint_ident: TestBlueprint,
        blueprint_snake_case: package,
        generics: {
            abc: {
                ident: Abc,
                description: "Some generic parameter called Abc",
            }
        },
        features: {},
        fields: {
            royalty:  {
                ident: Royalty,
                field_type: {
                    kind: StaticSingleVersioned,
                },
                condition: Condition::Always,
            },
            some_generic_field:  {
                ident: GenericField,
                field_type: {
                    kind: Generic,
                    ident: Abc,
                },
            }
        },
        collections: {
            some_key_value_store: KeyValue {
                entry_ident: MyCoolKeyValueStore,
                key_type: {
                    kind: Static,
                    content_type: BlueprintVersion,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                allow_ownership: true,
            },
            abc: Index {
                entry_ident: MyCoolIndex,
                key_type: {
                    kind: Static,
                    content_type: BlueprintVersion,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                allow_ownership: true,
            },
            def: SortedIndex {
                entry_ident: MyCoolSortedIndex,
                key_type: {
                    kind: Static,
                    content_type: BlueprintVersion,
                },
                full_key_content: {
                    full_content_type: ExampleSortedIndexKey,
                    sort_prefix_property_name: sort_prefix,
                },
                value_type: {
                    kind: StaticSingleVersioned,
                },
                allow_ownership: true,
            },
        }
    }

    pub struct ExampleSortedIndexKey(u16, BlueprintVersion);

    impl SortedIndexKeyFullContent<TestBlueprintMyCoolSortedIndexKeyPayload> for ExampleSortedIndexKey {
        fn from_sort_key_and_content(sort_key: u16, content: BlueprintVersion) -> Self {
            ExampleSortedIndexKey(sort_key, content)
        }
    }

    impl SortedIndexKeyContentSource<TestBlueprintMyCoolSortedIndexKeyPayload>
        for ExampleSortedIndexKey
    {
        fn into_sort_key_and_content(self) -> (u16, BlueprintVersion) {
            (self.0, self.1)
        }
    }
}
