mod event;
mod non_fungible_data;

pub use event::*;
pub use non_fungible_data::*;


pub trait TypeInfoMarker {
    const PACKAGE_ADDRESS: Option<super::types::PackageAddress>;
    const BLUEPRINT_NAME: &'static str;
    const OWNED_TYPE_NAME: &'static str;
    const GLOBAL_TYPE_NAME: &'static str;
}
