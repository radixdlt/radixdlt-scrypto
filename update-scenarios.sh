set -ex

cargo test --package transaction-scenarios --lib -- runners::dumper::test::update_expected_scenario_output --exact --nocapture