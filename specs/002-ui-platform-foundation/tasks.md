# Tasks: Web UI Platform Foundation

**Input**: Design documents from specs/002-ui-platform-foundation/

**Prerequisites**: plan.md, spec.md, and research.md

**Tests**: Add focused backend coverage for stable IDs, catalog compatibility, and dispatch; use the UI repository's existing checks and browser tooling. Do not add a frontend test framework.

## Phase 1: Brownfield Inventory

- [x] T001 Record the current worktree state and keep the pre-existing _deprecated/ and GLOSSARY.md changes out of this feature's staged diff.
- [x] T002 Confirm the registry inventory count and ensure each catalog entry appears exactly once in research.md (74 ranked integrations plus four informational platform entries).
- [x] T003 Inventory parent CI, Docker, Compose, README, and deployment references to web before replacing it with a submodule.
- [x] T004 Inventory UI routes and data-heavy layouts; record the viewport and keyboard scenarios required by spec.md.
- [ ] T005 Review all maintained references to the moved docs/project files and confirm no archived ADR is treated as active policy.

## Phase 2: Extract the UI Repository

### User Story 1 - Independent UI checkout

- [x] T006 Create public repository gloamy-fork-ui under the current authenticated GitHub account; preserve web-only history and push the UI tree to its main branch.
- [x] T007 Replace the parent web tree with a submodule pinned to the new repository's published commit; keep parent history unchanged.
- [x] T008 Move UI-only CI checks into the UI repository and update parent checkout/build paths to initialize the pinned submodule.
- [x] T009 Update parent Docker Compose, root/localized README, and deployment instructions for recursive checkout and UI-to-daemon proxy configuration.
- [x] T010 Update the UI repository README with independent development, build, deployment, and supported daemon API expectations.

## Phase 3: Responsive UI and Accessibility

### User Story 1 - Mobile route support

- [x] T011 Add a labeled topbar sidebar toggle and an off-canvas drawer below 768 CSS pixels; implement keyboard close, focus restoration, and announced expanded state.
- [x] T012 Make the shared shell, dashboard grids, tables, configuration lists, dialogs, authentication, and integrations responsive from 320 CSS pixels upward.
- [x] T013 Respect reduced-motion preferences and preserve visible focus and accessible names for controls changed by the responsive work.

### User Story 3 - WCAG requirements and roadmap

- [x] T014 Add docs/accessibility/wcag-2.2-aa-roadmap.md to the UI repository with every Level A and AA success criterion mapped to applicable surfaces, verification method, and current status.
- [ ] T015 Audit the current routes against the criteria and record prioritized gaps without claiming full conformance.
- [x] T016 Link the accessibility roadmap from the UI repository documentation.

## Phase 4: Integration Setup Foundation

### User Story 2 - Shared integration catalog and templates

- [x] T017 Assign a unique stable ID and one of four implementation types to each backend integration catalog entry; keep display categories separate.
- [x] T018 Add optional setup-capability metadata to the existing catalog response without removing or renaming legacy fields.
- [x] T019 Implement the central IntegrationSetupDispatcher for ai_provider, communication_channel, remote_service, and local_runtime; delegate to existing subsystem factories and register only implemented capabilities.
- [x] T020 Extract reusable integration-card and setup-dialog templates in the UI repository and register setup views by stable ID or capability.
- [x] T021 Preserve OpenAI Codex OAuth as distinct from OpenAI API-key setup and keep every unregistered setup card noninteractive.
- [x] T022 Add focused Rust coverage for unique IDs, additive serialization, unknown/missing capability behavior, and family dispatch; use existing frontend tooling without adding a new test framework.

## Phase 5: Ranked Readiness Inventory and Documentation

### User Story 4 - Select the next integrations

- [x] T023 Finalize the easiest-to-hardest inventory in research.md from the current 78 integration records, including wrappers, factories, generic connector coverage, platform prerequisites, effort rationale, and submodule assessment.
- [x] T024 Record Skills separately from integrations and note that no current integration is provided as a Git submodule.
- [ ] T025 Repair docs/SUMMARY.md, docs/docs-inventory.md, AGENTS.md, and docs/structure/README.md references to moved project docs; preserve _deprecated/ and link the root GLOSSARY.md.
- [x] T026 Update supported-locale README/deployment equivalents where they document UI checkout or deployment; no localized equivalents currently describe these UI checkout steps.

## Phase 6: Validation and Delivery

- [x] T027 Verify a clean recursive parent clone obtains the exact UI repository commit; verify parent CI and Docker/Compose use that pinned commit.
- [x] T028 Run the existing UI typecheck, build, lint, unit, and browser smoke checks; run focused Rust coverage and relevant formatting/lint checks, recording any skipped checks and reasons.
- [x] T029 Manually review every route at 320, 375, 768, 1024, and desktop widths; verify keyboard-only sidebar operation and the accessibility roadmap's audit evidence.
- [x] T030 Review the final catalog response for backward compatibility and the inventory for complete, one-to-one registry coverage.
- [x] T031 Review the complete diff for secrets, unrelated edits, and accidental staging of existing user changes; prepare a rollback note for the submodule transition.

## Remaining verification and user-owned worktree changes

- T005/T025: stale project-document references were cleaned in the local worktree, but those edits and the copied root `GLOSSARY.md` remain unstaged per T001; the PR does not claim to carry the user's archive move.
- T015: the full WCAG criterion-by-criterion and assistive-technology audit remains follow-up work; the roadmap records the required checks and no conformance claim is made.
- T028: the local Playwright browser revision was unavailable, but UI CI installed Chromium and passed `pnpm test:smoke`; local manual browser verification also passed. Full local `cargo test` and targeted rustfmt checks passed with `TMPDIR` outside the quota-limited `/tmp` mount. Full Clippy was attempted but reports 11 existing warnings in untouched modules, so unrelated lint cleanup was left out of this change.

## Dependencies and Execution Order

- T001-T005 must complete before moving files or changing docs.
- T006 must complete before T007; T007 blocks parent CI and Compose updates.
- T011-T013 and T014-T016 may proceed in parallel after the UI repository is available.
- T017-T019 block UI metadata consumption in T020-T021.
- The inventory in T023-T024 can be reviewed early; app-specific setup flows remain deferred until the user returns the selected list.
- T027-T031 depend on implementation completion.

## Notes

- Effort bands in research.md are relative planning estimates; integrations within a band are ties.
- No specific SaaS, smart-home, social, or media integration flow is selected by this feature.
- Do not add native iOS, Android, or Windows support.
