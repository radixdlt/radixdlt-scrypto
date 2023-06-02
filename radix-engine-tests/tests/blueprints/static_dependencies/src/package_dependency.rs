use scrypto::prelude::*;

pub struct Sample2;

pub trait Sample2Functions {
    fn callee();
}

impl Sample2Functions for ::scrypto::component::Blueprint<Sample2> {
    fn callee() {
        Self::call_function_raw("callee", scrypto_args!())
    }
}

impl HasTypeInfo for Sample2 {
    const PACKAGE_ADDRESS: Option<PackageAddress> = Some(PackageAddress::new_or_panic([
        13, 144, 99, 24, 198, 49, 140, 100, 247, 152, 202, 204, 99, 24, 198, 49, 140, 247, 189,
        241, 172, 105, 67, 234, 38, 49, 140, 99, 24, 198,
    ]));
    const BLUEPRINT_NAME: &'static str = "Sample";
    const OWNED_TYPE_NAME: &'static str = "OwnedSample";
    const GLOBAL_TYPE_NAME: &'static str = "GlobalSample";
}

#[blueprint]
mod cross_package {
    const PACKAGE_ADDRESS_PLACE_HOLDER: PackageAddress = PackageAddress::new_or_panic([
        13, 144, 99, 24, 198, 49, 140, 100, 247, 152, 202, 204, 99, 24, 198, 49, 140, 247, 189,
        241, 172, 105, 67, 234, 38, 49, 140, 99, 24, 198,
    ]);

    struct Sample {}

    impl Sample {
        pub fn call_external_package() {
            Blueprint::<Sample2>::callee();
        }

        pub fn callee() {}
    }
}
