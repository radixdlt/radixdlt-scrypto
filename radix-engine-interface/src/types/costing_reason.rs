use radix_engine_common::types::PackageAddress;

#[derive(Clone, Copy, Debug)]
pub enum ClientCostingEntry<'a> {
    RunNativeCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
    },
    RunWasmCode {
        package_address: &'a PackageAddress,
        export_name: &'a str,
        gas: u32,
    },
    PrepareWasmCode {
        size: usize,
    },
}
