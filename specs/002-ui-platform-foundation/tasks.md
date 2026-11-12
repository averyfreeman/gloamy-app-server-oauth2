# Tasks: Web UI Platform Foundation

**Input**: Design documents from specs/002-ui-platform-foundation/

**Prerequisites**: plan.md, spec.md, and research.md

**Tests**: Add focused backend coverage for stable IDs, catalog compatibility, and dispatch; use the UI repository's existing checks and browser tooling. Do not add a frontend test framework.

## Phase 1: Brownfield Inventory

- [x] T001 Record the worktree state and preserve pre-existing content; include only the `_deprecated/` moves and root `GLOSSARY.md` changes explicitly approved for this feature.
- [x] T002 Confirm the registry inventory count and ensure each catalog entry appears exactly once in research.md (74 ranked integrations plus four informational platform entries).
- [x] T003 Inventory parent CI, Docker, Compose, README, and deployment references to web before replacing it with a submodule.
- [x] T004 Inventory UI routes and data-heavy layouts; record the viewport and keyboard scenarios required by spec.md.
- [x] T005 Review moved-file links in `AGENTS.md`, `docs/SUMMARY.md`, `docs/docs-inventory.md`, and `docs/structure/README.md`; confirm `_deprecated/project/adr/` is historical, not active policy.

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
- [x] T015 [US3] Audit all 12 routes and record criterion-linked evidence and prioritized gaps in `web/docs/accessibility/wcag-2.2-aa-roadmap.md`, without claiming conformance.
- [x] T016 Link the accessibility roadmap from the UI repository documentation.

## Phase 4: Integration Setup Foundation

### User Story 2 - Shared integration catalog and templates

- [x] T017 [US2] Correct Email, iOS, and Android implementation families in `src/integrations/registry.rs`; preserve display categories and assert the four supported values in `src/gateway/codex_tests.rs`.
- [x] T018 Add optional setup-capability metadata to the existing catalog response without removing or renaming legacy fields.
- [x] T019 [US2] Replace metadata-only resolution in `src/integrations/setup.rs` with explicit executable Codex OAuth dispatch; route authenticated start/inspect/cancel through it in `src/gateway/codex.rs` and emit capabilities only for registered handlers in `src/gateway/api.rs`.
- [x] T020 Extract reusable integration-card and setup-dialog templates in the UI repository and register setup views by stable ID or capability.
- [x] T021 Preserve OpenAI Codex OAuth as distinct from OpenAI API-key setup and keep every unregistered setup card noninteractive.
- [x] T022 [US2] Add Rust coverage in `src/integrations/setup.rs` and `src/gateway/codex_tests.rs` for family values/roles, executable dispatch, unsupported capabilities, and authenticated OAuth transitions; do not add a frontend test framework.

## Phase 5: Ranked Readiness Inventory and Documentation

### User Story 4 - Select the next integrations

- [x] T023 Finalize the easiest-to-hardest inventory in research.md from the current 78 integration records, including wrappers, factories, generic connector coverage, platform prerequisites, effort rationale, and submodule assessment.
- [x] T024 Record Skills separately from integrations and note that no current integration is provided as a Git submodule.
- [x] T025 [US4] Include the six approved source moves under `_deprecated/`, root `GLOSSARY.md`, and corrected links in `AGENTS.md`, `docs/SUMMARY.md`, `docs/docs-inventory.md`, and `docs/structure/README.md`.
- [x] T026 Update supported-locale README/deployment equivalents where they document UI checkout or deployment; no localized equivalents currently describe these UI checkout steps.

## Phase 6: Validation and Delivery

- [x] T027 Verify a clean recursive parent clone obtains the exact UI repository commit; verify parent CI and Docker/Compose use that pinned commit.
- [x] T028 Run existing UI lint, typecheck, build, unit, and browser smoke checks plus Rust formatting, focused tests, full tests, and applicable lint checks; record any skipped checks and reasons.
- [x] T029 [US3] Manually audit all 12 routes at 320, 375, 768, 1024, and desktop widths; apply a WCAG text-spacing override; inspect the keyboard drawer and OAuth dialog paths, applicable states, and accessibility tree; record browser-zoom and assistive-technology checks that remain unverified in `web/docs/accessibility/wcag-2.2-aa-roadmap.md`.
- [x] T030 Review the final catalog response for backward compatibility and the inventory for complete, one-to-one registry coverage.
- [x] T031 Review the complete diff for secrets, unrelated edits, and accidental staging of existing user changes; prepare a rollback note for the submodule transition.

## Outstanding validation and delivery

- T028 is complete. Local UI lint/typecheck/build/unit checks and the 50-test Codex Rust subset pass. Browser smoke could not launch because Playwright Chromium is absent locally, while UI PR #2's GitHub CI passed browser smoke. A full local Rust run was interrupted by the workstation crash; Cargo metadata parses with `TMPDIR` configured to the disk-backed `$HOME/.cargo/tmp`. Parent CI run `43178454819` passed the full Rust tests on Ubuntu/macOS, Rust 1.96 check, three release builds, and three Tauri desktop bundles. Do not clean the existing target directory.
- Browser zoom, screen-reader operation, contrast measurements, and full visual overlap review remain explicitly unverified in the WCAG roadmap; the audit makes no conformance claim.
- T032, T033, and T034 are complete. Parent PR #15 is updated and ready for merge after the final documentation commit passes CI. UI PRs #1 and #2 are merged.

## Dependencies and Execution Order

- T001-T005 must complete before moving files or changing docs.
- T006 must complete before T007; T007 blocks parent CI and Compose updates.
- T011-T013 and T014-T016 may proceed in parallel after the UI repository is available.
- T017-T019 block UI metadata consumption in T020-T021.
- The inventory in T023-T024 can be reviewed early; app-specific setup flows remain deferred until the user returns the selected list.
- T017/T019/T022 implement the family/handler review findings; T032 updates the parent gitlink after the UI audit commit is published.
- T027-T031 and T033-T035 depend on implementation completion.

## Notes

- Effort bands in research.md are relative planning estimates; integrations within a band are ties.
- No specific SaaS, smart-home, social, or media integration flow is selected by this feature.
- Do not add native iOS, Android, or Windows support.

## Phase 7: Close Review Findings and Deliver

- [x] T032 [US1] Update the parent web submodule pointer to the merged UI main commit containing the responsive shell, WCAG roadmap, and preserved Tauri desktop shell.
- [x] T033 Review and stage the approved six-file `_deprecated/` move, root `GLOSSARY.md`, and linked reference edits in the parent repository.
- [x] T034 Run final Rust, UI, documentation, and manual-browser validation and record results in this feature's quickstart/evidence notes.
- [x] T035 Update parent PR #15 with completed changes and review responses; merge only after its required checks and review gates pass. UI PRs #1 and #2 are merged.
