# Gloamy Release Process

This runbook defines the maintainers' standard release flow for the workflows currently present in this repository.

Last verified: **November 9, 2026**.

## Release Goals

- Keep releases predictable and repeatable.
- Publish only from code already in `main`.
- Verify multi-target artifacts before publish.
- Keep release cadence regular even with high PR volume.

## Workflow Contract

Release automation currently lives in:

- `.github/workflows/release.yml` (`Beta Release`)
- `.github/workflows/promote-release.yml` (`Promote Release`)

### Beta release path

- Trigger: manual `workflow_dispatch` of `Beta Release`.
- Output:
  - prerelease GitHub tag/version in form `v<cargo-version>-beta.<run-number>`
  - build artifacts (`tar.gz`) and `SHA256SUMS`
  - Docker images tagged with beta tag and `beta`

### Stable release path

- Trigger: manual `workflow_dispatch` of `Promote Release` with input `version` (`X.Y.Z`).
- Guardrails enforced by workflow:
  - input must be semver `X.Y.Z`
  - `Cargo.toml` version must exactly match input version
  - stable tag `vX.Y.Z` must not already exist on origin
  - `docker_only: true` is allowed only for an existing stable tag and release
- Output:
  - stable GitHub release (`vX.Y.Z`) with build artifacts and `SHA256SUMS`
  - Docker images tagged with `vX.Y.Z` and `latest`

## Maintainer Procedure

### 1) Preflight on `main`

1. Ensure CI is green for the target commit.
2. Confirm no known high-priority regressions are unresolved.
3. Confirm the release matrix targets remain `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `aarch64-apple-darwin`.

### 2) Beta verification (manual)

1. Ensure main-push CI is green for the intended commit.
2. Dispatch `Beta Release` when a prerelease is needed.
3. Verify prerelease assets and Docker `beta` tag update.

### 3) Promote stable release (manual)

1. Ensure `Cargo.toml` has the intended stable version.
2. Run `Promote Release` workflow manually with `version=<Cargo.toml version>`.
3. Monitor `validate`, `build`, `publish`, and `docker` jobs. The Docker build has a 90-minute limit because it compiles both Linux architectures.

### 4) Post-release validation

1. Verify GitHub Release assets are downloadable.
2. Verify checksum file is present and non-empty.
3. Verify GHCR tags (`vX.Y.Z`, `latest`) are published.
4. Smoke-test one installation path that consumes release assets.

## Emergency / Recovery Path

If stable promotion fails:

1. Fix the issue on `main`.
2. Re-run `Promote Release` with the same target version if no tag was created.
3. If the stable GitHub Release was published but Docker publication failed, dispatch `Promote Release` with the same version and `docker_only: true`. This validates the existing stable release and retries only the Docker image job.

## Operational Notes

- Keep release changes small and reversible.
- Prefer one release checklist/issue per version so handoff is clear.
- Avoid ad-hoc release operations outside the documented workflows.
