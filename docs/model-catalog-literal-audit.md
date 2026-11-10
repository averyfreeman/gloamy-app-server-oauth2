# Tracked Model Literal Audit

Audit date: 2026-11-09 (implementation date). The eight-month eligibility window begins 2026-03-09. This audit is a cleanup inventory, not a claim that every real model string can be removed: the runtime must continue accepting arbitrary existing user IDs and retain provider-specific transport compatibility.

## Coverage

Scanned the 778 tracked files across Rust source, tests, templates, Python, web, docs, translations, and historical snapshots with case-insensitive searches for common model families and IDs, including OpenAI, Claude, Gemini, GLM, Qwen, Llama, MiniMax, Kimi, Grok, Whisper, embedding, speech, Mistral, Mixtral, Codestral, BGE, Phi, Ernie, Yi, Cohere, and Perplexity names. Excluded the newly researched catalog and its source record from stale-literal detection. Also excluded dependency lockfiles, compressed image assets, generated SVG artwork, and a base64 encoded workflow payload that matched only by chance.

Supported documentation locales in the repository are incomplete: tracked docs include English `README.md` and `docs/README.md`, a placeholder `docs/i18n/README.md`, and Python English/Vietnamese READMEs. There are no localized runtime-reference equivalents for `fr`, `vi`, `zh-CN`, `ja`, or `ru` in this checkout. Web UI locale JSON files contain no model IDs. Runtime reference changes therefore have no existing translated counterparts to synchronize.

## Findings

The final source-text scan found no remaining stale IDs in onboarding labels, generic test fixtures, auxiliary model defaults, docs, templates, Python examples, or web configuration examples. The old model-specific local-serving commands and transcription examples now use placeholders. Provider menus describe vendors without recommending release-specific IDs.

The remaining textual IDs are limited to the two OpenAI transport modules described below, the expected dated catalog/source record, and false positives from `command-logger` and the compressed workflow payload. Raster screenshots were excluded from text scanning and were not OCR-scanned.

## Compatibility exceptions

- `src/providers/openai.rs` retains exact GPT-family predicates and tests because request temperature behavior differs for those model families. These checks implement OpenAI transport compatibility, not user-facing catalog recommendations.
- `src/providers/openai_codex.rs` retains model-family checks and tests because Codex OAuth reasoning-effort constraints depend on those identifiers. They must remain separate from OpenAI API pricing and catalog recommendations.
- Other provider protocol fixtures must use synthetic IDs unless a concrete model string is required to assert a wire-format or protocol rule. Configured, live-discovered, and manually entered IDs remain opaque user values and are not invalidated by catalog recency.

## Verification

The scan used `git grep -niE` and `rg -ni` across tracked files, then manually classified matches. A final rerun after the doc updates left only the compatibility exceptions, the expected catalog/source entries, and known false positives. Runtime references have no localized equivalents in this checkout, so there are no translated reference files to update. This is a source-text audit; embedded text in raster screenshots was not OCR-scanned.
