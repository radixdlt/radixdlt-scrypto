# CI / Release Workflows

Reference for the GitHub Actions workflows in [`.github/workflows/`](workflows/), grouped by whether they run automatically or need to be dispatched manually for a release.

## Automatic workflows

These trigger on push/PR/schedule — nothing to run by hand.

| Workflow | Trigger | Purpose | Action |
|---|---|---|---|
| `ci.yml` | Push to `main`, `develop`, `docs`, `develop/*`, `release/*`; every PR | Full test suite (nextest + doc tests) | [Runs](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci.yml) |
| `ci-scrypto-builder.yml` | Same push/PR triggers | Builds & tags the `scrypto-builder` image | [Runs](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci-scrypto-builder.yml) |
| `ci-scrypto-dev-container.yml` | Same push/PR triggers | Builds & tags the `scrypto-dev-container` image | [Runs](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/ci-scrypto-dev-container.yml) |
| `bench.yml` | Every PR | Benchmarks PR vs base branch | [Runs](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/bench.yml) |
| `on-push-to-main.yml` | Push to `main` | Auto-opens PR `main → develop` | [Runs](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/on-push-to-main.yml) |
| `publish-cargo-crates.yml` | GitHub Release published/prereleased | Publishes all crates to crates.io | [Runs](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/publish-cargo-crates.yml) |

## Manual workflows — release sequence

Per the release flow documented in [`CONTRIBUTING.md`](../CONTRIBUTING.md), run these in order for a new release:

1. **[`dispatch-release.yml`](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/dispatch-release.yml)** — run on the `release/x.y.z` branch with input `release-tag: vX.Y.Z`. Bumps all crate versions via `update-cargo-toml-versions.sh`, commits, pushes the git tag, and opens the PR `release/x.y.z → main`.
2. Merge that PR into `main` (manual review). This auto-triggers `on-push-to-main.yml` (opens `main → develop` PR).
3. **Create the GitHub Release** from the pushed tag (the `gh release create` line in `dispatch-release.yml` is commented out, so this step is manual — via `gh release create` or the GitHub UI). Publishing it as `released`/`prereleased` auto-fires `publish-cargo-crates.yml`.
4. **[`publish-scrypto-builder.yml`](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/publish-scrypto-builder.yml)** — run with `image-label: vX.Y.Z` to push the new `scrypto-builder` Docker image (amd64) to Docker Hub.
5. **[`publish-scrypto-dev-container.yml`](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/publish-scrypto-dev-container.yml)** — run with `docker_tag: vX.Y.Z` to build+push the multi-arch `scrypto-dev-container` image.
6. **[`publish-docs.yml`](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/publish-docs.yml)** — run (no inputs) to rebuild/deploy the mdBook docs site to GitHub Pages.
7. **[`dispatch-update-development-version.yml`](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/dispatch-update-development-version.yml)** — run on `develop` afterward with `development-tag: vX.Y.(Z+1)-dev` to bump develop's version past the released one.

### Optional / situational

| Workflow | Trigger | Purpose | Action |
|---|---|---|---|
| `cpu_instructions.yml` | `workflow_dispatch` | Regenerates CPU instruction cost benchmarks — only needed if the release changes execution costs | [Runs](https://github.com/radixdlt/radixdlt-scrypto/actions/workflows/cpu_instructions.yml) |

## `publish-cargo-crates.yml` details

Runs on `selfhosted-ec2-ubuntu-22-2core`, fetches the crates.io token via AWS secrets (`AWS_SCRYPTO_RELEASE_SECRET_ROLE`), then runs `cargo publish` sequentially in dependency order:

```
radix-rust → sbor-derive-common → sbor-derive → sbor → radix-sbor-derive
→ radix-common → radix-common-derive → radix-blueprint-schema-init
→ radix-engine-interface → scrypto-derive → scrypto
→ radix-substate-store-interface → radix-substate-store-impls
→ radix-engine-profiling → radix-engine-profiling-derive
→ radix-native-sdk → radix-transactions → radix-engine
→ radix-transaction-scenarios → radix-substate-store-queries
→ scrypto-bindgen → scrypto-compiler → scrypto-test → radix-clis
```
