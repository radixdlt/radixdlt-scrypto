// build.rs
fn main() {
    // If some debugging needed then set below `debug()` and `cargo_debug()` to true
    cc::Build::new()
        .debug(false)
        .cargo_debug(false)
        .file("c_src/call_indirect.c")
        .compile("call_indirect");
}
