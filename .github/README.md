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

## External dependencies per job

Shared building blocks used across nearly every workflow (not repeated per-row below):
- **`RDXWorks-actions/*`** — the org's own mirror of common third-party actions (`checkout`, `cache`, `toolchain`, `install-action`, `configure-aws-credentials`, `login-action`, etc.). All workflows depend on this org being reachable.
- **`radixdlt/public-iac-resuable-artifacts`** — external repo providing reusable workflows (`docker-build.yml`, `fetch-secrets`, `join-docker-images-all-tags.yml`) used by the Docker/publish jobs.
- **`./.github/actions/setup-env`** — local composite action (installs Rust toolchain/nextest/cmake/clang); no external secrets itself.

| Workflow | Secrets | AWS / Secrets Manager | Self-hosted runner(s) | GH environment | Other external deps |
|---|---|---|---|---|---|
| `ci.yml` | none | none | `ubuntu-16-cores-selfhosted` (engine build/test/release/no-std, cargo-check, radix-clis*, determinism-test jobs) | none | `apt.llvm.org` (LLVM 22 installer in scrypto-coverage job); `packagecloud.io` (git-lfs installer) + Git LFS store for `assets-lfs/*.tar.gz` (determinism-test job) |
| `ci-scrypto-builder.yml` | `DOCKERHUB_RELEASER_ROLE` | OIDC role `arn:aws:iam::308190735829:role/gh-common-secrets-read-access` (hardcoded, no stored secret) → Secrets Manager path `github-actions/common/dockerhub-credentials-read-only` (read-only Docker Hub pull creds) | none (`ubuntu-latest`) | none | Docker Hub (`docker.io/radixdlt/private-scrypto-builder`) |
| `ci-scrypto-dev-container.yml` | `DOCKERHUB_RELEASER_ROLE` | (delegated to reusable `docker-build.yml`) | `gh-runner-scrypto-ubuntu-jammy-16-cores` | none | Docker Hub (`private-scrypto-dev-container`) |
| `bench.yml` | none | none | `ubuntu-16-cores-selfhosted` | none | `radixdlt/criterion-compare-action` (external action) |
| `cpu_instructions.yml` | none | none | none (`ubuntu-22.04`) | none | `download.qemu.org` (QEMU 8.0.3 source tarball); PyPI (`lxml`, `tabulate`, `numpy`, `scikit-learn`, `statsmodels`) |
| `on-push-to-main.yml` | default `GITHUB_TOKEN` | none | none | none | `gh pr create` (GitHub CLI, needs `write-all` permissions) |
| `dispatch-release.yml` | `GITHUB_TOKEN`; `CRATES_TOKEN` declared but currently unused by the script | none | none | **`release`** (manual-approval gate) | pushes tags/commits directly to the release branch |
| `dispatch-update-development-version.yml` | `GITHUB_TOKEN` | none | none | none | restricted to `develop` branch via `if:` guard |
| `publish-cargo-crates.yml` | none directly; role/path names come via `AWS_SCRYPTO_RELEASE_SECRET_ROLE` and `AWS_CRATES_TOKEN_SECRET_PATH` | IAM role (`AWS_SCRYPTO_RELEASE_SECRET_ROLE`) fetches the crates.io token from the Secrets Manager path named in `AWS_CRATES_TOKEN_SECRET_PATH`, via `public-iac-resuable-artifacts/fetch-secrets` | `selfhosted-ec2-ubuntu-22-2core` | none | crates.io (25 sequential `cargo publish` calls) |
| `publish-docs.yml` | default `GITHUB_TOKEN` (implicit, via Pages actions) | none | none | **`github-pages`** | GitHub Pages must be enabled on the repo; `sh.rustup.rs` + crates.io (installs `mdbook` via `cargo install`) |
| `publish-scrypto-builder.yml` | `DOCKERHUB_RELEASER_ROLE` | (delegated to reusable `docker-build.yml`) | `ubuntu-16-cores-selfhosted` | **`release`** | Docker Hub (`docker.io/radixdlt/scrypto-builder`, public) |
| `publish-scrypto-dev-container.yml` | `DOCKERHUB_RELEASER_ROLE` (amd64 + arm64 build jobs, and as `role-to-assume` in the join job) | Secrets Manager path `github-actions/rdxworks/dockerhub-images/release-credentials` (join-multiarch step) | `ubuntu-16-cores-selfhosted` (amd64), `selfhosted-ubuntu-22.04-arm` (arm64) | **`release`** (both build jobs) | Docker Hub multi-arch manifest join via `join-docker-images-all-tags.yml` |

Notes:
- Any workflow gated behind the `release` GitHub environment requires whatever manual-approval/reviewer rule is configured for that environment in repo settings before the job proceeds.
- Jobs on self-hosted runners (`*-selfhosted`, `gh-runner-*`) will queue indefinitely if that runner pool is offline or deregistered — worth checking runner health before relying on these for a release.
- `CRATES_TOKEN` in `dispatch-release.yml` is currently dead weight — the script doesn't call `cargo publish`; actual crate publishing happens later via `publish-cargo-crates.yml` triggered by the GitHub Release event.

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
