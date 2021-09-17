#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::engine::InMemoryRadixEngine;
use scrypto::prelude::*;

fn cross_component_call(b: &mut Bencher) {
    let mut engine = InMemoryRadixEngine::new(false);
    engine
        .publish_at(
            include_bytes!("../../assets/gumball-machine.wasm"),
            "05a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876"
                .parse()
                .unwrap(),
        )
        .unwrap();
    let package = engine
        .publish(include_bytes!("../../assets/gumball-machine-vendor.wasm"))
        .unwrap();

    let component = engine
        .call_function::<Address>(package, "Vendor", "new", args!())
        .unwrap();

    b.iter(|| {
        let bucket: Bucket = engine.prepare_bucket(1.into(), Address::RadixToken).into();
        engine
            .call_method::<Bucket>(component, "get_gumball", args!(bucket))
            .unwrap()
    });
}

benchmark_group!(radix_engine, cross_component_call);
benchmark_main!(radix_engine);
