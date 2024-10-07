set -ex

cargo test --package radix-engine-tests -- application::metering::update_expected_costs --exact --ignored --nocapture