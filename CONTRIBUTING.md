# Contributing Guide


## Code of conduct

This project adheres to the Contributor Covenant [code of conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please report unacceptable behavior to [hello@radixdlt.com](mailto:hello@radixdlt.com).


## Getting started

### Reporting an issue

* **Ensure the bug was not already reported** by searching on GitHub under [Issues](https://github.com/radixdlt/radixdlt-scrypto/issues).
* If you're unable to find an open issue addressing the problem, [open a new one](https://github.com/radixdlt/radixdlt-scrypto/issues/new). Be sure to include:
  * a **title**,
  * a **clear description**, 
  * as much **relevant information** as possible,
  * a **code sample** or an **executable test case** demonstrating the expected behavior that is not occurring.

### Workflows

Development flow:
1. Create feature branches using develop as a starting point to start new work;
1. Submit a new pull request to the `develop` branch 
   * please ensure the PR description clearly describes the problem and solution and include the relevant issue number if applicable.

Release workflow:
1. Create a release branch;
1. Tag the commit on Github releases;
1. Update `main` branch to point to the "newest" release (by version number);
1. Update `docs` branch to include documentation based on the "newest" release (by version number).

## Branching strategy

### Branches

* Feature - `feature/cool-bananas`
* Development  - `develop`
* Release - `release/0.1.0`
* Hotfix - `release/0.1.1`

Branch `main` always points to the latest release.

#### Features

Feature branches are where the main work happens. The goal is to keep them as independent from each other as possible. They can be based on a previous release or from develop.

> develop branch is not a place to dump WIP features

It’s important to remark that feature branches should only be merged to develop once they are complete and ideally tested in a test network.

#### Develop

This branch acts as staging for new releases, and are where most of QA should happen.

When QA gives the green light, a new release branch is created

#### Releases

These branches will stay alive forever, or at least while we support the release, thereby allowing us to release security hotfixes for older versions.

If QA discovers a bug with any of the features before a release happens, it is fixed in the feature branch taken from the release branch and then merged into the release again. 

These changes should immediately be propagated to the current release candidate branch.

#### Hotfixes

Hotfix branches are for providing emergency security fixes to older versions and should be treated like release branches.

The hotifx should be created for the oldest affected release, and then merged downstream into the next release or release candidate, repeated until up to date.


## Conventions

### Code style

We use the default code style specified by [rustfmt](https://github.com/rust-lang/rustfmt).

A convenience script is also provided to format the whole code base:

```
./format.sh
```

### Commit messages

Please follow the convention below for commit messages:

*  Separate subject from body with a blank line
*  Limit the subject line to 50 characters
*  Capitalise the subject line
*  Do not end the subject line with a period
*  Use the imperative mood in the subject line
*  Wrap the body at 72 characters
*  Use the body to explain what and why vs. how, separating paragraphs with an empty line.