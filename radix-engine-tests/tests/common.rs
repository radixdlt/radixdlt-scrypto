
/// Defines globally for all tests paths for various assets used during the tests.
/// To use it in a test definition file include following statement:
/// use crate::common::path_macros::*;
pub mod path_macros {

    #[macro_export]
    macro_rules! include_workspace_asset_bytes{
        ($name: expr)=>{ include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../assets/", $name)) }
    }

    #[macro_export]
    macro_rules! include_workspace_transaction_examples_str{
        ($name: expr)=>{ include_str!(path_workspace_transaction_examples!($name)) }
    }

    #[macro_export]
    macro_rules! path_workspace_transaction_examples{
        ($name: expr)=>{ concat!(env!("CARGO_MANIFEST_DIR"), "/../transaction/examples/", $name) }
    }

    #[macro_export]
    macro_rules! include_local_wasm_str{
        ($name: expr)=>{ include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/wasm/", $name)) }
    }

    #[macro_export]
    macro_rules! include_local_meterng_csv_str{
        ($name: expr)=>{ include_str!(path_local_meterng_csv!($name)) }
    }

    #[macro_export]
    macro_rules! path_local_blueprint{
        ($name: expr)=>{ concat!(env!("CARGO_MANIFEST_DIR"), "/assets/blueprints/", $name) }
    }

    #[macro_export]
    macro_rules! path_local_meterng_csv{
        ($name: expr)=>{ concat!(env!("CARGO_MANIFEST_DIR"), "/assets/metering/", $name) }
    }

    pub use crate::include_local_meterng_csv_str;
    pub use crate::include_local_wasm_str;
    pub use crate::include_workspace_asset_bytes;
    pub use crate::include_workspace_transaction_examples_str;
    pub use crate::path_local_blueprint;
    pub use crate::path_local_meterng_csv;
    pub use crate::path_workspace_transaction_examples;
}
