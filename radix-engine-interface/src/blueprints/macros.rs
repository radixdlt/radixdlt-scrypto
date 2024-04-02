// TODO: In the future, we would like to converge this macro (or something that provides its
// functionality) with the Scrypto blueprint macro such that they provide similar functionalities
// such as schema generation, generation of stubs, etc...
macro_rules! define_invocation {
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: struct { $($input_ident: ident: $input_type: ty),* $(,)? },
        output: struct { $($output_ident: ident: $output_type: ty),* $(,)? } $(,manifest_input: struct { $($manifest_input_ident: ident: $manifest_input_type: ty),* $(,)? } )?
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _EXPORT_NAME >]: &'static str = stringify!([<$function_name _ $blueprint_name:snake>]);

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                radix_common::ScryptoSbor,
                $($input_ident: $input_type),*
            }

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                radix_common::ScryptoSbor,
                $($output_ident: $output_type),*
            }

            $(
                $crate::blueprints::macros::resolve_struct_definition! {
                    [< $blueprint_name:camel $function_name:camel ManifestInput >],
                    radix_common::ManifestSbor,
                    $($manifest_input_ident: $manifest_input_type),*
                }
            )?
        }
    };
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: struct { $($input_ident: ident: $input_type: ty),* $(,)? },
        output: type $output_type:ty $(,manifest_input: struct { $($manifest_input_ident: ident: $manifest_input_type: ty),* $(,)? } )?
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _EXPORT_NAME >]: &'static str = stringify!([<$function_name _ $blueprint_name:snake>]);

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                radix_common::ScryptoSbor,
                $($input_ident: $input_type),*
            }

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                $output_type
            }

            $(
                $crate::blueprints::macros::resolve_struct_definition! {
                    [< $blueprint_name:camel $function_name:camel ManifestInput >],
                    radix_common::ManifestSbor,
                    $($manifest_input_ident: $manifest_input_type),*
                }
            )?
        }
    };
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: type $input_type:ty,
        output: struct { $($output_ident: ident: $output_type: ty),* $(,)? } $(,manifest_input: struct { $($manifest_input_ident: ident: $manifest_input_type: ty),* $(,)? } )?
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _EXPORT_NAME >]: &'static str = stringify!([<$function_name _ $blueprint_name:snake>]);

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                $input_type
            }

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                radix_common::ScryptoSbor,
                $($output_ident: $output_type),*
            }

            $(
                $crate::blueprints::macros::resolve_struct_definition! {
                    [< $blueprint_name:camel $function_name:camel ManifestInput >],
                    radix_common::ManifestSbor,
                    $($manifest_input_ident: $manifest_input_type),*
                }
            )?
        }
    };
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: type $input_type:ty,
        output: type $output_type:ty $(,manifest_input: struct { $($manifest_input_ident: ident: $manifest_input_type: ty),* $(,)? } )?
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _EXPORT_NAME >]: &'static str = stringify!([<$function_name _ $blueprint_name:snake>]);

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                $input_type
            }

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                $output_type
            }

            $(
                $crate::blueprints::macros::resolve_struct_definition! {
                    [< $blueprint_name:camel $function_name:camel ManifestInput >],
                    radix_common::ManifestSbor,
                    $($manifest_input_ident: $manifest_input_type),*
                }
            )?
        }
    };
}

macro_rules! resolve_struct_definition {
    ($name: ident, $derive: ty$(,)?) => {
        #[derive(sbor::rust::fmt::Debug, Eq, PartialEq, $derive)]
        pub struct $name;
    };
    ($name: ident, $derive: ty, $($ident: ident: $type: ty),* $(,)?) => {
        #[derive(sbor::rust::fmt::Debug, Eq, PartialEq, $derive)]
        pub struct $name {
            $(
                pub $ident: $type,
            )*
        }
    };
}

macro_rules! resolve_type_definition {
    ($name: ident, $type: ty) => {
        pub type $name = $type;
    };
}

pub(crate) use {define_invocation, resolve_struct_definition, resolve_type_definition};
