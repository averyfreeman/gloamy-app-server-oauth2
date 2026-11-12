# Implementation Plan: Web UI Platform Foundation

**Branch**: `feat/ui-platform-foundation` | **Date**: 2026-11-12 | **Spec**: [spec.md](spec.md)

## Summary

Complete the remaining documentation and accessibility audit work for the responsive UI and integration foundation, address the four findings on parent PR #15, and include the explicitly approved archive/glossary changes. UI PR #1 is merged; continue on parent PR #15 without creating a second feature branch or PR.

## Brownfield Findings

- The parent branch contains the Rust API/catalog and pins the standalone Vue repository at `web`; UI PR #1 merged as `f22a349` after its checks passed.
- Parent PR #15 has successful checks on its prior head and four Sourcery findings that are now addressed in the local working tree: inaccurate family labels for Email/iOS/Android, metadata-only setup resolution, incomplete family-value assertion, and the unfinished WCAG route audit.
- The router defines 12 routes. The existing WCAG roadmap lists 11 and omits `/cost-tracking`.
- The parent worktree already contains the approved move of six project/ADR/plan files into `_deprecated/`, a root `GLOSSARY.md`, and updated references in `AGENTS.md`, `docs/SUMMARY.md`, `docs/docs-inventory.md`, and `docs/structure/README.md`.
- Existing OAuth lifecycle and gateway code already provide safe flow views, bearer-owner checks, cleanup, encrypted persistence, and mockable backend behavior. The correction is to make setup registration executable and route operations through it.

## Technical Context

**Language / version**: Rust 2024; Vue 3/TypeScript with pnpm for the UI.

**Primary dependencies**: Existing Cargo workspace and UI package scripts; no new frontend test framework or runtime dependency.

**Storage**: Existing encrypted auth-profile store and configuration persistence remain unchanged.

**Testing and quality gates**: Rust unit tests, gateway mocked tests, UI lint/typecheck/build/smoke, manual browser and accessibility-tree review, and Markdown/link checks.

**Supported host targets**: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `aarch64-apple-darwin` only.

**Constraints**: Preserve additive catalog compatibility, bearer authentication, provider/channel/tool boundaries, all user configuration and secrets, and the exact archived file contents.

## Constitution Check

- [x] Least privilege and explicit errors preserved; OAuth operations remain bearer-authenticated and return no token material.
- [x] Module boundaries preserved; the setup dispatcher delegates Codex OAuth to the existing auth service and does not replace provider/channel/tool factories.
- [x] Changes are reversible and stay on the existing feature branches/PRs.
- [x] Validation uses existing UI/Rust tooling; no frontend test framework or new dependency is added.
- [x] No unsupported host target is introduced.

## Design and Implementation

1. **Setup registration**: Replace catalog-derived metadata resolution with an explicit registration for `(openai-codex, oauth_device_code)`. The registered handler must execute start, inspect, and cancel operations; catalog serialization emits a capability only when a matching executable registration exists. Keep current routes and response shapes.
2. **Family metadata**: Keep the four supported family values. Set Email to `communication_channel`; set iOS and Android to `communication_channel` because their catalog descriptions specify access through Telegram/Discord. Keep display category `platform` and document that they are not native Gloamy targets. Assert every serialized family is in the four-value set.
3. **WCAG route audit**: Add `/cost-tracking` to the roadmap, then audit all 12 routes, shared navigation, setup dialogs, and applicable loading/error/empty/connected states. Record criterion IDs, tested state, reproducible evidence, priority, and follow-up action. Use “no issue observed in tested scenarios” separately from “unverified”; make no conformance claim. Only fix issues blocking the responsive shell or current acceptance criteria.
4. **Documentation archive**: Preserve and include the six exact files under `_deprecated/` and the root `GLOSSARY.md`; repair links and verify that archived ADRs are not active policy.
5. **Delivery**: Pin the merged UI commit in the parent, update parent PR #15 and its review response, then wait for updated checks and review outcomes before merging.

### Project Structure

```text
src/integrations/setup.rs          # explicit setup registrations and dispatch
src/integrations/registry.rs       # stable catalog IDs and family metadata
src/gateway/api.rs                 # additive capability serialization
src/gateway/codex.rs               # authenticated operation adapters
src/gateway/codex_tests.rs         # catalog, auth, and dispatch regressions
web/docs/accessibility/             # route audit and criterion evidence
docs/                               # active docs and archived source files
specs/002-ui-platform-foundation/   # updated feature artifacts and validation guide
```

**Structure Decision**: Extend existing integration, gateway, and UI structures. Keep OAuth lifecycle and persistence in the current auth service.

## Validation Approach

- Rust: `cargo fmt --all -- --check`, focused setup/catalog/gateway tests, full `cargo test`, and Clippy where the known baseline permits; distinguish existing warnings from changed code.
- UI: `pnpm lint`, `pnpm typecheck`, `pnpm build`, and existing smoke tests. Manually inspect all 12 routes at 320, 375, 768, 1024, and desktop widths, apply text-spacing overrides, and inspect keyboard paths and the browser accessibility tree. Attempt 200% zoom; if browser tooling cannot set it, record it as unverified and keep it as follow-up work.
- Docs: Markdown lint/link checks; verify every old active path is absent while the six archive files and glossary content remain intact.
- GitHub: Confirm UI PR checks before merging it; update the parent submodule pointer to the merged UI commit, then verify parent checks and review findings before merging.

## Rollback

Revert the UI audit commit and parent changes independently. Restore the prior submodule pointer if needed. The archive and glossary content are part of the approved parent documentation diff and must be restored from that commit if the documentation changes are reverted.

## Complexity Tracking

| Violation | Why needed | Simpler alternative rejected because |
| --- | --- | --- |
| N/A | The single executable handler is explicit and directly used by the Codex routes. | A metadata-only catalog lookup cannot prove that an advertised capability can execute. |
