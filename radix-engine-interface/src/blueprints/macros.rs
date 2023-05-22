macro_rules! define_invocation {
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: struct { $($input_ident: ident: $input_type: ty),* },
        output: struct { $($output_ident: ident: $output_type: ty),* }
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                $($input_ident: $input_type),*
            }

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                $($output_ident: $output_type),*
            }
        }
    };
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: struct { $($input_ident: ident: $input_type: ty),* },
        output: type $output_type:ty
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                $($input_ident: $input_type),*
            }

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                $output_type
            }
        }
    };
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: type $input_type:ty,
        output: struct { $($output_ident: ident: $output_type: ty),* }
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                $input_type
            }

            $crate::blueprints::macros::resolve_struct_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                $($output_ident: $output_type),*
            }
        }
    };
    (
        blueprint_name: $blueprint_name: ident,
        function_name: $function_name: ident,
        input: type $input_type:ty,
        output: type $output_type:ty
    ) => {
        paste::paste! {
            pub const [< $blueprint_name:snake:upper _ $function_name:snake:upper _IDENT >]: &'static str = stringify!($function_name);

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Input >],
                $input_type
            }

            $crate::blueprints::macros::resolve_type_definition! {
                [< $blueprint_name:camel $function_name:camel Output >],
                $output_type
            }
        }
    };
}

macro_rules! resolve_struct_definition {
    ($name: ident $(,)?) => {
        #[derive(sbor::rust::fmt::Debug, Eq, PartialEq, crate::ScryptoSbor)]
        pub struct $name;
    };
    ($name: ident, $($ident: ident: $type: ty),*) => {
        #[derive(sbor::rust::fmt::Debug, Eq, PartialEq, crate::ScryptoSbor)]
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
