use scrypto::prelude::*;

#[blueprint]
mod metadata_component {
    struct M {}

    impl M {
        pub fn f() {
            let package_address = METADATA_MODULE_PACKAGE;
            let blueprint_name = METADATA_BLUEPRINT;
            let function_name = METADATA_CREATE_WITH_DATA_IDENT;
            let args = include_bytes!("/tmp/args.bin");

            unsafe {
                scrypto::prelude::wasm_api::blueprint::blueprint_call(
                    package_address.as_bytes().as_ptr(),
                    package_address.as_bytes().len(),
                    blueprint_name.as_ptr(),
                    blueprint_name.len(),
                    function_name.as_ptr(),
                    function_name.len(),
                    args.as_ptr(),
                    args.len(),
                );
            }
        }
    }
}
