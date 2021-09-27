#[macro_use]
extern crate bencher;
use bencher::Bencher;

use radix_engine::engine::*;
use radix_engine::utils::*;
use scrypto::prelude::*;

fn create_account(engine: &mut InMemoryRadixEngine) -> Address {
    let mut track = engine.start_transaction();
    let mut proc = track.start_process(false);

    // Publish Account blueprint
    let acc_bp = Address::Package([0u8; 26]);
    proc.publish_at(include_bytes!("../../assets/account.wasm"), acc_bp)
        .unwrap();

    // Create account
    let account: Address = proc
        .call_function((acc_bp, "Account".to_owned()), "new", args!())
        .and_then(decode_return)
        .unwrap();

    // Allocate 1 XRD
    let bid = proc.reserve_bucket_id();
    proc.put_bucket(
        bid,
        radix_engine::model::Bucket::new(1.into(), Address::RadixToken),
    );
    proc.call_method(account, "deposit", args!(Bucket::from(bid)))
        .unwrap();

    // Commit
    proc.finalize().unwrap();
    track.commit();

    account
}

fn create_gumball_machine(engine: &mut InMemoryRadixEngine) -> Address {
    let mut track = engine.start_transaction();
    let mut proc = track.start_process(false);

    let package = proc
        .publish(include_bytes!("../../assets/gumball-machine.wasm"))
        .unwrap();

    let component: Address = proc
        .call_function((package, "GumballMachine".to_owned()), "new", args!())
        .and_then(decode_return)
        .unwrap();

    proc.finalize().unwrap();
    track.commit();

    component
}

fn bench_swap_transaction(b: &mut Bencher) {
    let mut engine = InMemoryRadixEngine::new();
    let account = create_account(&mut engine);
    let component = create_gumball_machine(&mut engine);

    b.iter(|| {
        let mut track = engine.start_transaction();
        let mut proc = track.start_process(false);
        let xrd: Bucket = proc
            .call_method(
                account,
                "withdraw",
                args!(Amount::one(), Address::RadixToken),
            )
            .and_then(decode_return)
            .unwrap();
        let gum: Bucket = proc
            .call_method(component, "get_gumball", args!(xrd))
            .and_then(decode_return)
            .unwrap();
        proc.call_method(account, "deposit", args!(gum)).unwrap();
        proc.finalize().unwrap();
        //track.commit();
    });
}

benchmark_group!(radix_engine, bench_swap_transaction);
benchmark_main!(radix_engine);
