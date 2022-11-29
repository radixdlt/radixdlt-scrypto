#!/bin/bash

# Add '-p' refix to each create in list.
# This is for cargo command.
to_cargo_crates() {
    # input must be space-separated list of crates
    local crates="$1"
    local out=
    for c in $crates
    do
        out+="-p $c "
    done
    echo "$out"
}

test_crates_features() {
    local crates=$(to_cargo_crates "$1")
    local args="${2:-}"

    cargo test $crates $args
}

test_packages() {
    # input must be space-separated list of packages
    local packages="${1:-}"

    for p in $packages
    do
        (cd $p; scrypto test)
    done
    (cd assets/blueprints/account; scrypto test)
    (cd assets/blueprints/faucet; scrypto test)
    (cd examples/hello-world; scrypto test)
    (cd examples/no-std; scrypto test)
}

test_cli() {
    # input must be space-separated list of bash scripts
    local clis="${1:-}"
    for c in $clis
    do
        (cd simulator; bash $c)
    done
}

test_benchmark() {
    local crates=$(to_cargo_crates "$1")
    local args="${2:-}"
    cargo bench $crates $args
}

