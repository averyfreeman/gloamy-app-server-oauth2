# Implementation Plan: Web UI Platform Foundation

**Branch**: feat/ui-platform-foundation | **Date**: 2026-11-11 | **Spec**: spec.md

## Summary

Move the existing Vue application into a public history-preserving repository and pin it as the parent's web submodule. Make the entire application responsive, record a WCAG 2.2 AA roadmap, and standardize integration metadata and setup dispatch without implementing unselected integrations.

## Brownfield Findings

- **Current behavior and architecture**: Rust API and runtime code live in the parent repository. The Vue 3, TypeScript, Vite, and pnpm application is tracked in web. The backend integration catalog is in src/integrations/registry.rs; provider, channel, and tool factories are separate. The UI fetches /api/integrations and /api/skills separately. The integration cards already have a responsive grid, while the sidebar has persisted collapse state without a visible toggle. Docker Compose uses ./web as its build context.
- **Repository and dependency state**: The parent has no Git submodules. A read-only gh repo view found no existing averyfreeman/gloamy-fork-ui repository. The existing project checkout has intentional uncommitted archive moves and GLOSSARY.md; implementation must not stage or overwrite them.
- **Existing UI checks**: web/package.json defines typecheck, build, lint, Vitest unit tests, and Playwright smoke tests. Keep the existing toolchain; do not add a frontend test framework.
- **Naming and governance**: Follow AGENTS.md, the Rust provider/channel/tool factory boundaries, pnpm lockfile discipline, supported host targets, and docs locale rules. Treat _deprecated/ as historical archive, not active architecture policy.
- **Current documentation drift**: docs/SUMMARY.md, docs/docs-inventory.md, AGENTS.md, and docs/structure/README.md still point to the moved docs/project collection, ADR, or glossary. Repair those references without restoring archived content.

## Technical Decisions

1. Create a public GitHub repository named gloamy-fork-ui under the current authenticated account. Use a path-only history split of web and push that history to the new repository's main branch.
2. Replace the tracked web tree in the parent with a submodule at web. Keep the parent history unchanged. Use the stable public HTTPS repository URL in .gitmodules.
3. Make the UI repository responsible for its own UI-only workflow. Parent workflows and deployment instructions must initialize the pinned submodule wherever they build the UI; Docker Compose keeps the existing web build context.
4. Keep the UI's same-origin /api, /pair, and /health requests and GLOAMY_DAEMON_URL proxy model. A UI repo deployment must document how its reverse proxy reaches the daemon.
5. Use responsive behavior below 768 CSS pixels as an off-canvas navigation drawer controlled from the topbar. Retain a persistent, collapsible desktop sidebar at and above that breakpoint. Use existing Tailwind responsive conventions and avoid new dependencies.
6. Record all applicable WCAG 2.2 Level A and AA success criteria in the UI repository, using the [W3C Recommendation](https://www.w3.org/TR/WCAG22/) as normative source. Map criteria to route/components and manual or automated verification, then prioritize gaps. Implement the accessibility behavior required for the new sidebar and responsive changes, but make no site-wide conformance claim.
7. Extend the existing backend catalog with unique stable IDs and an implementation_type distinct from display category. Preserve the current /api/integrations wrapper and all legacy fields. Use optional setup_capability only when a backend handler and matching UI registration exist.
8. Create one IntegrationSetupDispatcher over four families: ai_provider, communication_channel, remote_service, and local_runtime. Reuse provider/channel/tool factories; register only implemented capabilities. Do not create integrations-specific config forms until the user chooses them from the inventory.
9. Keep Skills outside the integrations inventory because they come from /api/skills.

## Project Structure and Change Inventory

### UI repository changes

- Move the current web tree into the repository root while preserving its path-specific history.
- Update the UI README, independent CI workflow, and existing proxy/deployment documentation.
- Add docs/accessibility/wcag-2.2-aa-roadmap.md with a complete Level A/AA criterion matrix, per-route applicability, verification guidance, gap priority, and no-conformance-claim notice.
- Update the application shell, sidebar, page layouts, and dialogs for responsive behavior.
- Extract the card and setup presentation into reusable components while retaining the setup registry keyed by stable integration identity/capability.

### Parent repository changes

- Add .gitmodules and replace web's tracked contents with a gitlink to the new UI repository.
- Update the parent CI checkout and Docker/Compose setup to use the pinned submodule; move UI-only checks to the UI repository workflow while retaining any required parent integration build.
- Update root and localized README/deployment docs where UI checkout instructions exist.
- Update docs/SUMMARY.md, docs/docs-inventory.md, AGENTS.md, and docs/structure/README.md to remove references to archived docs/project content and link GLOSSARY.md where appropriate.
- Extend IntegrationEntry/catalog serialization with stable ID and implementation type; add the four-family dispatcher under src/integrations and keep provider/channel/tool implementations in their existing subsystems.
- Update the web catalog types and page to consume optional additive metadata.

### Preserve

- Parent history, existing config and credentials, gateway bearer authentication, same-origin API behavior, local UI storage, and all unrelated routes.
- Existing OpenAI Codex OAuth and OpenAI API-key identities as separate integrations.
- The intentional _deprecated/ archive and root GLOSSARY.md changes; never include them implicitly in this feature's staging or commits.
- Only the three currently supported host targets. iOS and Android remain client platforms for existing channels, not native Gloamy targets.

## Constitution Check

- [x] Secure and explicit boundaries: setup capabilities are opt-in, UI secrets remain governed by the existing authenticated API, and unimplemented cards stay inert.
- [x] Trait and factory boundaries: four setup families delegate to existing provider, channel, and tool surfaces.
- [x] Small and reversible: the parent history remains intact; the submodule pointer can be reverted.
- [x] Reproducibility: preserve pnpm lockfile and pin the submodule commit.
- [x] Supported targets: no native mobile or Windows support is introduced.

## Validation Approach

- Confirm a fresh recursive clone checks out the exact UI commit and the documented parent Compose build uses it.
- Use the existing UI typecheck, build, lint, and browser smoke-test commands; add only focused tests for catalog serialization/IDs and dispatcher behavior.
- Manually inspect every route at 320, 375, 768, 1024, and desktop CSS-pixel widths; inspect the drawer with keyboard-only interaction and browser accessibility tools.
- Verify every WCAG 2.2 A/AA criterion is mapped, and state plainly which criteria remain unverified or fail.
- Verify every current catalog item appears once in the readiness inventory; verify the /api/skills section remains separate.

## Rollback

Revert the parent submodule pointer and restore the previous web tree from parent history to roll back the repository split. The public UI repository can remain as a standalone repository. Revert responsive and integration-dispatch changes independently; preserve user configuration and archived files.
