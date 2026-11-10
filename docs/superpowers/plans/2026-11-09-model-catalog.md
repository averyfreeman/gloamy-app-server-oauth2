# Reloadable model catalog implementation plan

**Goal:** Implement the user-supplied revised Stage 1 model catalog design.

**Architecture:** A complete local TOML catalog replaces the bundled catalog when present. A shared store publishes validated immutable `Arc` snapshots. Consumers acquire a current snapshot for each new lookup; live provider discovery remains independent.

**Tech stack:** Rust, serde/TOML, chrono calendar arithmetic, parking_lot locks, existing Axum gateway authentication. No new dependencies.

## Constraints and review focus

- Eight calendar months of eligibility at lookup time, with injected dates in tests; no unverified release dates or inferred aliases.
- Exact provider/model pricing, explicit overrides first, no inferred subscription OAuth prices.
- Existing selections and arbitrary discovered/entered IDs remain valid.
- Only missing files fall back; invalid replacement files retain the last good daemon snapshot and return an error.
- No network access during catalog loading or lookup.
- Check month-end boundaries, concurrent readers, expired recommendations, and config serialization.

## Tasks

- [x] Implement catalog parsing, validation, date eligibility, exact lookup, snapshot publication, and reload tests in `src/model_catalog.rs`; add bundled `src/model_catalog.toml` and register modules in library/binary.
- [x] Research eligible models from official vendor release/pricing sources and record direct sources, verification dates, and gaps.
- [x] Migrate config defaults and override serialization, onboarding/CLI choices, provider convenience lists, and cost observer. Test selection preservation and price precedence without prefix guessing.
- [x] Add bearer-authenticated `POST /api/models/catalog/reload`, load the active directory for new processes, and test authentication/failure behavior.
- [x] Audit tracked model literals across source, frontend, tests, templates, documentation, translations, and historical snapshots. Replace generic examples with neutral synthetic IDs and preserve necessary transport rules separately.
- [x] Update runtime-contract documentation and existing localized equivalents with schema, eligibility, loading/reload, overrides, and unknown rates.
- [x] Run formatting, focused tests, clippy, repository tests, documentation checks and a requirement-by-requirement completion audit. Record limitations honestly.

Rollback: revert the catalog change as one feature branch; user catalog files and explicit overrides remain user data.

Validation results and remaining source/environment limits are recorded in [model-catalog-validation.md](../../model-catalog-validation.md).
