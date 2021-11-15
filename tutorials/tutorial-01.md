# First Steps

All done with the main setup contained in the [README](../README.md)?  Great, let's walk through the steps of running some code, starting with a new project.

First, we'll create a new package.
```bash
scrypto new-package tutorial
```
This will scaffold a simple package which contains a single `Hello` blueprint.  Open the new package directory in an IDE with Rust support.  We mostly use VS Code or IntelliJ...both have free Rust extensions/plugins which work fine.

Open up `src/lib.rs` to see the source that we'll be compiling and deploying to the simulator.  `Hello` provides a function which instantiates a new component with a new supply of tokens, and a method which allows the caller to get one of those tokens.

Before we can deploy, we first need to create an account on the simulator.  Most of commands offered by the `resim` tool act within the context of an account.
```bash
resim new-account
```
You should get a success status, and at the bottom of the output you should see something like this:
```
================================================================================
A new account has been created!
Public key: 04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
Account address: 02526629b90e1142492e934fbe807b446935407064db3ea2fcf856
As this is the first account, it has been set as your default account.
================================================================================
```
If you do _not_ see the line about seeting your default account set, then you created an account previously.  Either reset your entire simulator with the `resim reset` command and try again, or set this new account as your default account with the `resim set-default-account` command.

Save the address of your new account component to an environment variable to make your life easier later.  E.g.,
```bash
export account=02526629b90e1142492e934fbe807b446935407064db3ea2fcf856
export pubkey=04005feceb66ffc86f38d952786c6d696c79c2dbc239dd4e91b46729d73a27fb57e9
```

You can look at your new account by running
```bash
resim show $account
```
Which will show that your account comes pre-supplied with 1 million XRD.

Next, we'll publish our package.  Note that it is not necessary to perform a `scrypto build` operation before doing so...publishing will take care of that if a build is needed.  Switch to your `tutorial` directory and run
```bash
resim publish .
```
At the bottom of the output, you should see `└─ Package: <package address>`.  Save that address in the `package` environment variable.

Now that our package is "on ledger" in the simulator, we'll need to instantiate a `Hello` component by calling the `new` function on the `Hello` blueprint.
```bash
resim call-function $package Hello new
```
This will create two new entities with two new addresses: a resource definition for your new `HelloToken` (save this to the `token` environment variable), and your fresh `Hello` component (save this to the `component` environment variable).

You can use `resim show` to investigate these new addresses, if you wish.

Next, we'll call the `free_token` method on our new component.  Note that this requires a different resim command than the one we just used.
```bash
resim call-method $component free_token
```
And now you have a shiny new HelloToken in your account, and your `Hello` component has one less.  You can verify this with some `resim show` investigation of each.

You can create more accounts, and use `resim set-default-account` to change which one you're acting as.

If you make changes to the structs within your code, then unfortunately you will have to run through the entire publish-instantiate-call flow from scratch, saving the new addresses as they appear.  (We are working on an option to let you generate addresses deterministically, which would remove some of the hassle here.) If you only make code changes then it is possible to update your package with `resim publish --address $package`.

At any point you can use `resim reset` to instantly get a clean slate in the simulator. You almost certainly need to do this if you switch to working on a different project.

One more useful tip...if you ever pull the latest and see that the core Scrypto implementation has changed (i.e., stuff above the `examples` directory), you will usually need to re-run `cargo install --path ./simulator` from the `radixdlt-scrypto` root directory in order to pick up the changes. And sometimes you will also need to run `resim reset` to clear the simulator state. 

That concludes this most basic tutorial on using `resim`.  More content to come!