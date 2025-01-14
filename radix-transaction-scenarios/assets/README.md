## Scenario Blueprints

To create a new blueprint:
* Install the relevant scrypto compiler
* Go to `/assets/blueprints`, and use `scrypto init`
* Work on your blueprint
* Build your blueprint locally, or reproducibly with the `build.sh` script, and copy the `.rpd` and `.wasm` assets to this folder.
* Use the built assets in your scenario

And then do some clear-up:
* Remove the `lib.rs` to `historic_blueprint_sources` and rename.
* Remove the blueprints folder

This clear-up stops us having old sources and Cargo.lock files in this repo, which give us bad dependabot ratings.