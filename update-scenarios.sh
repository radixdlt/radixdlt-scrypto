set -ex

cargo test --package radix-transaction-scenarios --lib -- runners::dumper::test::update_expected_scenario_output --exact --nocapture