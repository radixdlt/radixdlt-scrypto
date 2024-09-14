set -ex

cargo test --package radix-engine-tests --test system_folder -- system::execution_cost::update_flamegraph_of_faucet_lock_fee_method --exact  --ignored --show-output
cargo test --package radix-engine-tests --test system_folder -- system::execution_cost::update_flamegraph_of_faucet_lock_fee_and_free_xrd_method --exact  --ignored --show-output
