set -ex

cargo test --package transaction-scenarios --test update_expected_scenario_output -- --exact --ignored