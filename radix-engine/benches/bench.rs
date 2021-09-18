#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::engine::*;
use scrypto::prelude::*;

fn cross_component_call(b: &mut Bencher) {
    let mut engine = InMemoryRadixEngine::new();
    let mut runtime = engine.start_runtime();
    let mut proc = runtime.start_process(false);

    proc.publish_at(
        include_bytes!("../../assets/gumball-machine.wasm"),
        "05a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876"
            .parse()
            .unwrap(),
    )
    .unwrap();

    let package = proc
        .publish(include_bytes!("../../assets/gumball-machine-vendor.wasm"))
        .unwrap();

    let component: Address = proc
        .call_function(package, "Vendor", "new", args!())
        .and_then(decode_return)
        .unwrap();

    b.iter(|| {
        let bucket =
            scrypto::resource::Bucket::from(proc.create_bucket(1.into(), Address::RadixToken));
        proc.call_method(component, "get_gumball", args!(bucket))
            .unwrap()
    });
}

benchmark_group!(radix_engine, cross_component_call);
benchmark_main!(radix_engine);
