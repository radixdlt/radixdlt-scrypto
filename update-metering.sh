set -ex

cargo test --package radix-engine-tests --test metering -- update_expected_costs --exact --ignored