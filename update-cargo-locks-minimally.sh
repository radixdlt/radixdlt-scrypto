#!/bin/bash

set -e

# This script causes minimal updates to sync up all the cargo locks

# This script can be run as a quick-fix for an error such as:
# > the lock file [PATH]/Cargo.lock needs to be updated but --locked was passed to prevent this
# > If you want to try to generate the lock file without accessing the network, remove the --locked flag and use --offline instead.

# This should align with format.sh, format-check.sh, check.sh, build.sh, test.sh

(set -x; cd .; cargo update --workspace)

(set -x; cd radix-engine-tests/assets/blueprints; cargo update --workspace)
(set -x; cd radix-clis/tests/blueprints; cargo update --workspace)
(set -x; cd scrypto-test/tests/blueprints; cargo update --workspace)
(set -x; cd scrypto-test/assets/blueprints; cargo update --workspace)
(set -x; cd scrypto-compiler/tests/assets/call_indirect; cargo update --workspace)
(set -x; cd scrypto-compiler/tests/assets/scenario_1; cargo update --workspace)
(set -x; cd scrypto-compiler/tests/assets/scenario_2; cargo update --workspace)

(
    find "examples" -mindepth 2 -maxdepth 2 -type f \( -name Cargo.toml \) -print \
    | xargs -I '{}' bash -c "set -x; cargo update --workspace --manifest-path {}"
)

# Finally, let's fix Cargo.lock_template for `scrypto new-package`.
# The lock_template needs to be equivalent to the Cargo.lock for a fresh
# scrypto package, with two dependencies (scrypto and scrypto-test), except
# with the a missing [[package]] section for the new package itself.
# This gets added back in the right place alphabetically during the
# templating process; to create a perfect Cargo.lock which satisfies a
# --locked command.
#
# To build the lock_template we:
# * Start with the hello-world example
# * Remove the [[package]] section for name = "hello-world" by:
#   * Finding the line number for 'name = "hello-world"'
#   * Using `tail` to look from that line number, and finding the next line number of a `name =` line
#   * Removing everything between those two lines
# * Write it to the Cargo.lock_template file
hello_world_line_num=$(grep -Fn "name = \"hello-world\"" "examples/hello-world/Cargo.lock" | cut -f 1 -d ":")
length_of_section_plus_one=$(tail -n +$hello_world_line_num "examples/hello-world/Cargo.lock" | grep -Fn "name = " | head -n 2 | tail -n 1 | cut -f 1 -d ":")
delete_up_to=$(($hello_world_line_num + $length_of_section_plus_one - 2))
sed "$hello_world_line_num,${delete_up_to}d" "examples/hello-world/Cargo.lock" > "radix-clis/assets/template/Cargo.lock_template"