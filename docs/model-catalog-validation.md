# Model catalog implementation and validation

Implementation and source audit date: **2026-11-09**. Final native validation: **2026-11-10 UTC**.

## Requirement coverage

| Requirement | Implementation and evidence |
| --- | --- |
| Complete local catalog; bundled fallback only when absent | `Config::load_or_init` loads the active directory. Catalog tests cover missing files, invalid existing files, and complete replacement without merging. An invalid active-directory catalog also fails config loading. |
| Immutable snapshots and atomic validated publication | `CatalogStore` publishes `Arc<Catalog>` under a short write lock after validation. Tests retain an old snapshot across successful reload and exercise concurrent readers. Failed reload leaves the valid snapshot intact. |
| Offline catalog loading and lookup | The catalog reads local TOML and parses source URLs without fetching them. Provider discovery remains a separate explicit fetch/cache path. |
| Curated choices, defaults, exact prices and explicit aliases | Shared catalog methods serve onboarding, CLI listing, runtime defaults, diagnostics, provider convenience choices, and cost observation. Tests check exact provider isolation, aliases, unknown rates, and rejected inferred snapshot suffixes. |
| Eight calendar months at lookup time | Injected-date tests cover the inclusive cutoff, the day before it, future releases, month-end clamping, expired recommendations, and explicit selection preservation. Runtime lookup uses UTC dates. |
| Reload authentication and failure behavior | Gateway handler tests cover missing/invalid bearer tokens with pairing enforcement disabled, successful reload, rejected invalid replacement, and a generic failure response that omits file contents. |
| Explicit overrides before catalog rates | Config round-trip tests prove only explicit overrides are persisted. Cost tests cover qualified versus legacy bare overrides, provider aliases, no inferred OpenAI API prices for Codex subscription usage, and later lookups observing reloaded prices. |
| Current file observed without recompilation | A CLI smoke test used the same executable for two different local catalogs, verified the changed curated IDs, preserved an arbitrary configured selection, and rejected an invalid existing catalog. The executable hash stayed unchanged. |
| Literal cleanup and documentation | See the [literal audit](model-catalog-literal-audit.md), [source verification](model-catalog-sources.md), and [runtime contract](config-reference.md#curated-model-catalog). Existing Python English/Vietnamese examples were synchronized. Other localized runtime references are absent from this checkout. |

## Validation results

- `cargo test --locked --no-fail-fast`: **passed**, 22 targets, 6,884 test executions, 11 ignored, zero failures. The final run used two build jobs and a disk-backed `TMPDIR`. An earlier doctest link crashed with a bus error in the memory-backed temporary filesystem; the full rerun passed.
- `cargo build --locked --bin gloamy`: **passed**.
- Python tests: **11 passed** using an isolated `uv` environment with pytest, pytest-asyncio, LangGraph and langchain-openai. Tests used `-c /dev/null --asyncio-mode=auto` because the pre-existing Python project TOML has an unclosed array. Python syntax compilation also passed.
- Formatting checks on all 50 changed Rust files: **passed**. Repository-wide `cargo fmt --all -- --check` still reports existing drift in 77 unchanged files; none of the changed files appear in its findings.
- `cargo clippy --locked --all-targets -- -D warnings`: **failed on 11 pre-existing findings**, concerning sorting, conversions, formatting borrows, return bindings, chunk iteration, loop shape, and a match. The expressions are present in the base revision; this change does not suppress their lints or modify those expressions to bypass the check.
- Markdown lint: **zero errors on changed lines**. Full-file lint reports 12 existing findings outside changed lines in the root README and operations runbook.
- Local Markdown links: **94 destinations checked, zero missing**. Bootstrap shell syntax and `git diff --check`: **passed**.

## Compatibility and limits

Existing configured, discovered, and manually entered IDs remain accepted even after their catalog eligibility expires. New configs leave `default_model` unset and resolve an eligible recommendation when needed. Providers without a recommendation require discovery or explicit input. Default delegate agents use the selected provider/model.

Existing `[cost.prices]` entries remain explicit overrides. Resolved catalog rates are no longer serialized into config. Prefix, case, suffix, and cross-provider guesses are removed. Slash-containing model IDs require provider-qualified override keys; legacy bare keys remain supported for slashless IDs. Unknown rates retain numeric-zero accounting while tokens are counted, so totals and budget estimates can be lower bounds.

Transcription, speech, and embedding models now require explicit IDs when those features are enabled. The old Telnyx model constants are replaced by a current catalog lookup. Remote Ollama discovery now requests the configured endpoint instead of presenting a static list as live discovery.

The bundled starter catalog has 11 eligible entries across three vendor families, three recommendations, and three flat price pairs. Other provider families and subscription prices remain unverified. Tiered or temporary prices that cannot be represented accurately are omitted. No additional model aliases were verified for the bundled entries. See the source record for exact conditions and gaps.

Native validation ran on Linux amd64; container and cross-platform builds were not run. Transport compatibility identifiers remain in the two OpenAI transport modules so existing configured models keep their protocol behavior. The text audit did not OCR raster screenshots.

## Risk and rollback

The highest-risk surface is the new gateway mutation. It requires a stored bearer token even when general pairing enforcement is disabled, reads only the active directory's catalog, fetches no remote metadata, publishes only validated replacements, and omits file payloads from daemon reload logs. Cost resolution stays isolated by provider and model.

Rollback is to revert this feature commit and restore a previous valid catalog file if desired. The change does not rewrite existing user selections or explicit override values. Do not merge until required CI checks and human review are complete.
