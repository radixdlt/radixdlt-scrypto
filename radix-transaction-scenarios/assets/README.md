## Scenario Blueprints

These blueprints are created for use in the scenarios.

We care less about the builds being reproducible than for the mainnet deployed assets - really we just care about the built assets being fixed, and having the source available for reference.

You may wish to either use an officially released scrypto compiler, or a locally installed one (if the feature has not been released yet).

### Using an officially released compiler

* Install the relevant scrypto compiler
* Go to `/assets/blueprints`, and use `scrypto init`
* Work on your blueprint
* Use the `build.sh` script to build the scenario reproducible, and copy the `.rpd` and `.wasm` assets to this folder.
* Once you're happy with the scenario, before committing, run through the clearing up steps below.

### Using a local compiler

You can use this to add scenarios against scrypto features which are yet to be included in an official release.

* Install the current compiler with `cargo install radix-clis`.
* Go to `/assets/blueprints`, and use `scrypto init`
* Work on your blueprint
* Build your blueprint with `scrypto build`, and copy the `.rpd` and `.wasm` assets to this folder.
* Use the built assets in your scenario.
* Once you're happy with the scenario, before committing, run through the clearing up steps below.

### Clearing up

And then do some clear-up:
* Remove the `lib.rs` to `historic_blueprint_sources` and rename.
* Remove the blueprints folder

This clear-up stops us having old sources and Cargo.lock files in this repo, which give us bad dependabot ratings. But we keep the historic sources for reference.