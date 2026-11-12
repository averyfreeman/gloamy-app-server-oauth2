# Feature Specification: Independent, Responsive Web UI and Integration Setup Foundation

**Feature Branch**: `feat/ui-platform-foundation`

**Created**: 2026-11-11

**Status**: Draft

**Input**: User request to split the web UI into a public repository, improve mobile use, record WCAG requirements, standardize integration setup, and rank listed integrations by implementation readiness. Remaining work is limited to the unfinished documentation and WCAG audit tasks, review findings on the open PRs, and the user-approved archive/glossary changes.

## User Scenarios & Testing

### User Story 1 - Work with the UI on a phone or tablet (Priority: P1)

As a Gloamy user on a small screen, I want every existing route to remain usable without desktop-width layout assumptions.

**Independent Test**: Visit each current route at 320, 375, 768, 1024, and desktop CSS-pixel widths and verify that controls and content remain usable.

**Acceptance Scenarios**:

1. **Given** a viewport narrower than 768 CSS pixels, **When** the authenticated UI opens, **Then** the sidebar is initially out of the content flow and can be opened from a clearly labeled topbar toggle.
2. **Given** a narrow viewport, **When** a user views a card grid, form, dialog, or data view, **Then** its layout adapts and does not cause unintended page-level horizontal scrolling.
3. **Given** a data view that requires two-dimensional layout, **When** it cannot reflow, **Then** its scrollable region is explicit, keyboard reachable, and does not force the entire page to scroll horizontally.
4. **Given** the sidebar is open, **When** a keyboard user closes it, **Then** focus returns to the toggle and the expanded state is announced.

### User Story 2 - Set up only integrations with implemented capability (Priority: P1)

As a user, I want integration cards to show a consistent status and setup action, and I want unsupported cards to remain visibly noninteractive.

**Independent Test**: Load the integration catalog against both a current server and a response without the new optional fields; confirm that entries render and only registered setup capabilities can open a dialog.

**Acceptance Scenarios**:

1. **Given** an integration has a recognized setup capability, **When** the user activates its card by pointer or keyboard, **Then** the matching reusable setup UI opens.
2. **Given** an integration has no recognized setup capability, **When** the user browses the catalog, **Then** its card is not focusable as a button and does not imply that setup is available.
3. **Given** an existing client reads the integration catalog, **When** the server adds IDs and implementation metadata, **Then** the existing response wrapper and fields remain valid.
4. **Given** the catalog contains OpenAI Codex and OpenAI API entries, **When** either is displayed, **Then** ChatGPT OAuth remains clearly distinct from API-key configuration.

### User Story 3 - Understand the accessibility work still needed (Priority: P1)

As a user or maintainer, I want testable accessibility requirements and known gaps recorded so the UI can move toward WCAG conformance without overstating its current state.

**Independent Test**: Review the UI repository's accessibility document and confirm that each WCAG 2.2 Level A and AA success criterion is mapped to applicable UI surfaces, a verification method, and a current status or a justified not-applicable reason; confirm that all 12 routes are included in the audit record.

**Acceptance Scenarios**:

1. **Given** the responsive UI and setup dialog, **When** keyboard and assistive-technology requirements are reviewed, **Then** focus, names, states, errors, status announcements, contrast, reflow, target size, and motion are recorded with verification guidance.
2. **Given** the audit finds a failing or unverified criterion, **When** the roadmap is updated, **Then** it records route/state evidence, priority, and next action without claiming full conformance.

### User Story 4 - Review which integrations are ready to implement (Priority: P1)

As the maintainer choosing the next integration cards, I want a source-backed list ordered from easiest to hardest with current wrappers, setup prerequisites, and submodule needs identified.

**Independent Test**: Compare the ranked inventory with every entry returned by the current backend integration registry and confirm that Skills are reported separately.

**Acceptance Scenarios**:

1. **Given** the current integration registry, **When** the inventory is reviewed, **Then** each integration is listed in an effort band with the existing backend infrastructure and the remaining work described.
2. **Given** an entry has only a generic One or Composio connector, **When** it is assessed, **Then** the inventory does not claim that a dedicated Gloamy wrapper or provider-side platform support exists.
3. **Given** a catalog status is Active, Available, or Coming Soon, **When** implementation readiness is assessed, **Then** the assessment checks actual provider/channel/tool code because status alone does not prove setup support.

## Edge Cases

- The UI repository can be updated independently of the backend; missing optional catalog fields must degrade to a visible, noninteractive card.
- A fresh parent clone may not initialize the UI submodule unless clone instructions and CI explicitly request it.
- Existing integration status values may describe configuration state rather than whether a setup flow exists.
- Some integrations have code despite a Coming Soon catalog status; the inventory must note that mismatch.
- iOS and Android appear as platform entries but are not supported native host targets. Mobile compatibility in this feature means responsive web UI, not native mobile applications.
- The route inventory includes `/cost-tracking` even though it is a placeholder page; all 12 routes receive an audit disposition.
- Skills are loaded from a separate endpoint and must not be counted as integration catalog entries.
- The six moved project/ADR/plan files under `_deprecated/` and the root `GLOSSARY.md` are explicitly included in the parent documentation changes for this feature.

## Requirements

### Functional Requirements

- **FR-001**: The Vue application MUST have a public repository named gloamy-fork-ui, created under the currently authenticated GitHub account, with the existing web-path history preserved.
- **FR-002**: The parent repository MUST reference the Vue repository as a submodule at the existing web path without rewriting parent history.
- **FR-003**: The Vue repository MUST own its UI-only build and quality workflow. Parent checkout, CI, Docker Compose, and setup documentation MUST initialize or clearly require the pinned submodule where the UI is built.
- **FR-004**: All current UI routes MUST support responsive layouts from 320 CSS pixels upward. Cards, forms, dialogs, navigation, and data views MUST adapt to available width.
- **FR-005**: At widths below 768 CSS pixels, the left navigation MUST use an accessible drawer controlled by a visible topbar button. The button MUST expose its label, expanded state, and controlled region; closing the drawer MUST restore focus.
- **FR-006**: The responsive shell and changed controls MUST support keyboard operation, visible focus, meaningful accessible names, reduced-motion preferences, and announced state or error changes.
- **FR-007**: The Vue repository MUST document WCAG 2.2 Level A and AA requirements as a criterion-by-criterion checklist with applicable surfaces, verification method, status, and prioritized gaps. Use the [W3C WCAG 2.2 Recommendation](https://www.w3.org/TR/WCAG22/) as the normative source. This feature MUST NOT claim full conformance.
- **FR-008**: The existing integration catalog MUST remain the source for display metadata and MUST gain stable IDs and an implementation type for every entry. The existing display category MUST remain distinct from implementation type.
- **FR-009**: The catalog response MUST change additively. Existing wrappers, field names, and meanings MUST remain compatible; setup capability metadata MUST be optional when a setup flow is not implemented.
- **FR-010**: Backend setup dispatch MUST use one central registry/dispatcher with executable handlers and no more than five implementation families. A capability MUST be advertised only when its registered handler can execute the advertised operation. This feature selects four: AI inference providers, communication channels, remote service connectors, and local runtime capabilities.
- **FR-011**: The four families MUST reuse the existing Provider, Channel, and Tool boundaries and factories where applicable; the dispatcher MUST NOT replace them with one monolithic integration implementation.
- **FR-012**: The UI MUST provide reusable integration-card and setup-dialog templates keyed by stable integration identity or setup capability. A card MUST be interactive only when the server advertises a capability that the UI has registered.
- **FR-013**: This feature MUST establish the shared catalog, dispatch, and UI templates without implementing setup forms for integrations the user has not yet selected. Existing Codex OAuth setup remains available; all other unimplemented cards remain noninteractive.
- **FR-014**: The implementation inventory MUST cover all 78 entries currently returned by the integration registry, ordered by rough implementation effort, and record current wrapper/factory support, remaining setup work, platform or external prerequisites, and whether a Git submodule is actually required.
- **FR-015**: Skills MUST be reported separately because the UI loads them from /api/skills rather than the integration catalog.
- **FR-016**: Documentation references to moved project files MUST be removed or corrected. The six moved files MUST remain under `_deprecated/`, and the root `GLOSSARY.md` MUST retain the glossary content.
- **FR-017**: The feature MUST preserve existing provider and channel behavior, API authentication, runtime configuration, and supported host targets. It MUST NOT add native iOS, Android, or Windows support.
- **FR-018**: Every catalog `implementation_type` MUST be one of the four selected families and match the entry's documented implementation role. Email MUST use the communication-channel family; iOS and Android MUST describe access through existing communication channels, not local native runtimes.
- **FR-019**: The WCAG audit MUST cover `/`, `/cowork`, `/tools`, `/scheduled`, `/integrations`, `/memory`, `/configuration`, `/cost-tracking`, `/logs`, `/doctor`, `/authentication`, and `/settings`, including applicable dialogs and loading, error, empty, and connected states.

### Existing Project Impact

- **User-facing surfaces**: Web UI routes, integration catalog, documentation, Docker Compose setup, and repository checkout workflow. CLI behavior and unrelated runtime configuration are unaffected.
- **Brownfield paths**: The UI is a submodule at web; server catalog and setup dispatch are under src/integrations and src/gateway; provider/channel/tool factories remain in their respective modules; Docker Compose builds from ./web. The six historical files moved from docs/project and docs/superpowers/plans are retained under _deprecated/, with active references repaired in AGENTS.md, docs/SUMMARY.md, docs/docs-inventory.md, and docs/structure/README.md.
- **Compatibility and migration**: Parent Git history is not rewritten. The extracted UI repository retains web-only history. The parent pins a UI commit as a submodule. The integration API change is additive, and old servers without new metadata continue to render noninteractive cards.
- **Preservation boundary**: Keep the existing OpenAI Codex OAuth setup separate from OpenAI API-key setup; keep the same-origin API proxy and bearer authentication; preserve all existing user configuration, auth profiles, routes, security policy, the full archived file contents, and the root glossary.

### Key Entities

- **Integration catalog entry**: Stable ID, user-facing category and metadata, runtime status, implementation type, and optional setup capability.
- **Setup capability**: A server-advertised operation for which a matching UI setup registration exists.
- **Implementation family**: One of the four selected backend setup handler types; not the same as a display category.
- **Accessibility criterion**: A WCAG success criterion mapped to affected UI surfaces and a verification method.

## Success Criteria

- **SC-001**: The public gloamy-fork-ui repository contains the current Vue application and preserves its web-path commit history.
- **SC-002**: A fresh recursive parent clone contains the pinned UI submodule and can build the documented UI deployment.
- **SC-003**: Every current route remains usable at 320, 375, 768, 1024, and desktop CSS-pixel widths, with no unintended page-level horizontal overflow.
- **SC-004**: Keyboard users can open, navigate, and close the mobile sidebar with visible focus and correctly announced state.
- **SC-005**: The accessibility document contains all WCAG 2.2 Level A and AA criteria and an audit disposition for all 12 routes, with verification evidence, prioritized gaps, and no claim of full conformance.
- **SC-006**: Every integration catalog entry has a unique stable ID and an implementation type from the selected four-family set; old catalog consumers remain compatible.
- **SC-007**: Only registered setup capabilities produce interactive cards, and the existing Codex setup remains distinct from OpenAI API-key setup.
- **SC-008**: The ranked inventory covers the 78 current integration entries and distinguishes dedicated code, generic connectors, partial support, and absent wrappers.
- **SC-009**: Every advertised setup capability resolves to and executes a registered handler; unsupported IDs and capabilities remain unavailable.
- **SC-010**: All catalog entries serialize one of the four supported implementation families, with Email and mobile client platform roles represented accurately.
- **SC-011**: The six moved historical files remain intact under `_deprecated/`, active documentation has no stale links to their old locations, and the root glossary retains its content.

## Assumptions

- The public repository slug is gloamy-fork-ui and the owner is the currently authenticated GitHub account used for the parent repository.
- Preserve the UI subtree history; do not rewrite the parent repository's history.
- Mobile means responsive browser UI for every existing route, not native iOS or Android applications.
- WCAG 2.2 Level AA is the target for the recorded roadmap; full conformance verification remains follow-up work.
- The integration architecture uses four concrete handler families and leaves app-specific setup flows for a later user-selected list.
- Remaining implementation work continues on the existing feature branch and open pull requests; do not create a second feature directory.
- The route audit records gaps and verified evidence; it does not attempt broad WCAG remediation or certify conformance.
