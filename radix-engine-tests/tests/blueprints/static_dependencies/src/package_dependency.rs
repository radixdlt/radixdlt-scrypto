use scrypto::prelude::*;

pub struct Sample2;

pub trait Sample2Functions {
    fn callee() -> u32 {
        let package_address = PackageAddress::new_or_panic([
            13, 144, 99, 24, 198, 49, 140, 100, 247, 152, 202, 204, 99, 24, 198, 49, 140, 247, 189,
            241, 172, 105, 67, 234, 38, 49, 140, 99, 24, 198,
        ]);
        ::scrypto::runtime::Runtime::call_function(package_address, "Sample", "y", scrypto_args!())
    }
}

impl Sample2Functions for ::scrypto::component::Blueprint<Sample2> {}

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
