# GitHub Actions source policy

Workflow changes should use reviewed action sources listed here. Add a source to
this list in the same change that introduces it, and note the source change in
the PR description.

## Current sources

- `actions/checkout@v4` — repository checkout.
- `actions/download-artifact@v4` — retrieve build artifacts.
- `actions/setup-node@v4` — select the web job's Node.js runtime and cache pnpm.
- `actions/upload-artifact@v4` — preserve build artifacts.
- `docker/build-push-action@v6` — build and publish container images.
- `docker/login-action@v3` — authenticate container publishing.
- `docker/setup-buildx-action@v3` — configure container builds.
- `dtolnay/rust-toolchain@stable` — install Rust toolchains and targets.
- `pnpm/action-setup@v4` — install the repository-pinned pnpm version.
- `softprops/action-gh-release@v2` — create release assets.
