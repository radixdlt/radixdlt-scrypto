<!--
> The purpose of this document is to be turned into:
> * Release Overviews (e.g. in github release notes or Discord/email announcements)
> * Detailed developer-facing release notes, to assist developers upgrading their scrypto or other integrations to the new version.
>
> It should be grouped by:
> # Protocol Update
> ## Version (Minor or patch)
> ### Subsections: Major Features | Breaking changes | Other changes
> 
> A new release should contain the following:

> [!NOTE]
> 
> This release is under development.

### Major Features

> Headline features that might be called out in the release overviews

* *Pending...*

### Breaking changes

> Changes which may cause compilation errors for Scrypto developers or other integrators

* *Pending...*

### Other changes

> Other incidental features or changes which shouldn't break existing integrations, but are worthy of mention to scrypto developers, dApp developers or other integrators.

* *Pending...*

-->

# v1.4.x - [Dugong](https://docs.radixdlt.com/docs/dugong)

## v1.4.0

> [!NOTE]
> 
> This release is under development.

### Major Features

> Headline features that might be called out in the release overviews

* *Pending...*

### Breaking changes

> Changes which may cause compilation errors for Scrypto developers or other integrators

* [#2035](https://github.com/radixdlt/radixdlt-scrypto/pull/2035) - types specifying multiple types in `#[sbor(categorize_types = "S, T")]` should now use a semi-colon as a separator: `#[sbor(categorize_types = "S; T")]`
* [#2017](https://github.com/radixdlt/radixdlt-scrypto/pull/2017) - Manual implementations of `ContextualDisplay` must now take a `&mut fmt::Formatter` instead of a `F: fmt::Write`.

### Other changes

> Other incidental features or changes which shouldn't break existing integrations, but are worthy of mention to scrypto developers, dApp developers or other integrators.

* Minor updates to improve the `name` and `description` of the native node module packages.

# v1.3.x - [Cuttlefish](https://docs.radixdlt.com/docs/cuttlefish)

## v1.3.0

We didn't have a formal changelog. Please see the [protocol updates](https://docs.radixdlt.com/docs/protocol-updates) section of the docs site for more information.

# v1.2.x - [Bottlenose](https://docs.radixdlt.com/docs/bottlenose) and before

We didn't have a formal changelog. Please see the [protocol updates](https://docs.radixdlt.com/docs/protocol-updates) section of the docs site for more information.