use scrypto::prelude::*;

#[blueprint]
mod metadata2 {
    struct M {}

    impl M {
        pub fn f() {
            let package_address = METADATA_MODULE_PACKAGE;
            let blueprint_name = METADATA_BLUEPRINT;
            let function_name = METADATA_CREATE_WITH_DATA_IDENT;

            /*
                // Use the following code to generate payload

                let mut urls: Vec<UncheckedUrl> = vec![];
                for _ in 0..10_000 {
                    urls.push(UncheckedUrl::of(format!("https://www.example.com/test?q=x")));
                }
                urls.push(UncheckedUrl::of("invalid"));

                let mut data = MetadataInit::default();
                data.set_metadata("urls", urls.clone());
                let args = scrypto_encode(&MetadataCreateWithDataInput { data }).unwrap();
            */
            let mut args = Vec::with_capacity(330033);
            args.extend([
                92, 33, 1, 35, 12, 33, 1, 4, 117, 114, 108, 115, 2, 34, 1, 1, 34, 141, 1, 32, 12,
                145, 78,
            ]);
            for _ in 0..10_000 {
                args.extend([
                    32, 104, 116, 116, 112, 115, 58, 47, 47, 119, 119, 119, 46, 101, 120, 97, 109,
                    112, 108, 101, 46, 99, 111, 109, 47, 116, 101, 115, 116, 63, 113, 61, 120,
                ]);
            }
            args.extend([7, 105, 110, 118, 97, 108, 105, 100, 1, 0]);

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
