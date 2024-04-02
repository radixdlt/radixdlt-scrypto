set -ex

cargo test --package radix-engine-tests --test application -- application::metering::update_expected_costs --exact --ignored --nocapture