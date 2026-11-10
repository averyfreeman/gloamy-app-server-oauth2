//! Shared, reloadable catalog of curated model metadata.
//!
//! The catalog is loaded from `model-catalog.toml` in the active config
//! directory when present. The bundled catalog is used only when that file is
//! absent. Readers acquire an immutable [`Arc<Catalog>`] snapshot, so a reload
//! never changes data already in use by an operation.

use anyhow::{Context, Result, bail};
use chrono::{Months, NaiveDate, Utc};
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock};

const BUNDLED_CATALOG: &str = include_str!("model_catalog.toml");
const CATALOG_FILE: &str = "model-catalog.toml";

/// A curated model and the metadata verified for it.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelEntry {
    /// Stable provider key used by Gloamy.
    pub provider: String,
    /// Exact provider model identifier.
    pub id: String,
    /// Human-readable label for menus.
    pub name: String,
    /// Official model release date. Aliases use this same date.
    pub released: NaiveDate,
    /// Whether this is the provider's recommended default.
    #[serde(default)]
    pub recommended: bool,
    /// Explicit identifiers that resolve to this model for this provider.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Input USD per million tokens, when verified and applicable.
    pub input_price_per_million: Option<f64>,
    /// Output USD per million tokens, when verified and applicable.
    pub output_price_per_million: Option<f64>,
    /// Direct sources used to verify this entry.
    pub sources: Vec<String>,
    /// Date the sources and metadata were last verified.
    pub verified: NaiveDate,
}

/// Validated model catalog loaded from a single TOML document.
#[derive(Debug, Clone)]
pub struct Catalog {
    models: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CatalogDocument {
    schema_version: u32,
    models: Vec<ModelEntry>,
}

impl Catalog {
    /// Parse and validate a catalog document.
    pub fn parse(contents: &str) -> Result<Self> {
        let document: CatalogDocument =
            toml::from_str(contents).context("parse model catalog TOML")?;
        if document.schema_version != 1 {
            bail!(
                "unsupported model catalog schema_version {}; expected 1",
                document.schema_version
            );
        }

        let mut identifiers: HashMap<&str, HashSet<&str>> = HashMap::new();
        let mut recommended: HashSet<&str> = HashSet::new();
        for model in &document.models {
            validate_text("provider", &model.provider)?;
            validate_text("id", &model.id)?;
            validate_text("name", &model.name)?;
            if model.released > model.verified {
                bail!(
                    "model {}/{} was verified before its release date",
                    model.provider,
                    model.id
                );
            }
            if model.verified > Utc::now().date_naive() {
                bail!(
                    "model {}/{} has a future verification date",
                    model.provider,
                    model.id
                );
            }
            if model.sources.is_empty() {
                bail!(
                    "model {}/{} must have at least one source",
                    model.provider,
                    model.id
                );
            }
            for source in &model.sources {
                let parsed = reqwest::Url::parse(source).with_context(|| {
                    format!(
                        "model {}/{} has an invalid verification source URL",
                        model.provider, model.id
                    )
                })?;
                if parsed.scheme() != "https" || parsed.host_str().is_none() {
                    bail!(
                        "model {}/{} has a non-HTTPS verification source",
                        model.provider,
                        model.id
                    );
                }
            }
            validate_price(
                &model.provider,
                &model.id,
                "input",
                model.input_price_per_million,
            )?;
            validate_price(
                &model.provider,
                &model.id,
                "output",
                model.output_price_per_million,
            )?;

            let provider_ids = identifiers.entry(&model.provider).or_default();
            if !provider_ids.insert(&model.id) {
                bail!("duplicate model identifier {}/{}", model.provider, model.id);
            }
            for alias in &model.aliases {
                validate_text("alias", alias)?;
                if !provider_ids.insert(alias) {
                    bail!(
                        "duplicate model identifier or alias {}/{}",
                        model.provider,
                        alias
                    );
                }
            }
            if model.recommended && !recommended.insert(&model.provider) {
                bail!(
                    "provider {} has more than one recommended model",
                    model.provider
                );
            }
        }

        Ok(Self {
            models: document.models,
        })
    }

    /// Return eligible curated models for the provider on `as_of`.
    pub fn curated(&self, provider: &str, as_of: NaiveDate) -> Vec<ModelEntry> {
        self.models
            .iter()
            .filter(|model| model.provider == provider && eligible(model.released, as_of))
            .cloned()
            .collect()
    }

    /// Return the eligible recommended model ID, if one exists.
    pub fn recommended(&self, provider: &str, as_of: NaiveDate) -> Option<String> {
        self.models
            .iter()
            .find(|model| {
                model.provider == provider && model.recommended && eligible(model.released, as_of)
            })
            .map(|model| model.id.clone())
    }

    /// Preserve a configured ID or require an eligible recommended model.
    pub fn require_model(
        &self,
        provider: &str,
        configured: Option<&str>,
        as_of: NaiveDate,
    ) -> Result<String> {
        if let Some(model) = configured {
            if model.trim().is_empty() {
                bail!("configured model for provider {provider} must not be empty");
            }
            return Ok(model.to_owned());
        }
        self.recommended(provider, as_of).ok_or_else(|| {
            anyhow::anyhow!(
                "provider {provider} has no eligible recommended model; discover models live or enter a model ID"
            )
        })
    }

    /// Return verified exact-provider pricing for an ID or explicit alias.
    ///
    /// A complete input/output pair is required. Missing rates are unknown and
    /// are not inferred from another provider or a related model family.
    pub fn pricing(&self, provider: &str, model_id: &str, as_of: NaiveDate) -> Option<(f64, f64)> {
        self.models
            .iter()
            .find(|model| {
                model.provider == provider
                    && eligible(model.released, as_of)
                    && (model.id == model_id || model.aliases.iter().any(|alias| alias == model_id))
            })
            .and_then(|model| {
                model
                    .input_price_per_million
                    .zip(model.output_price_per_million)
            })
    }
}

/// Shared catalog whose replacements are validated before publication.
pub struct CatalogStore {
    current: RwLock<Arc<Catalog>>,
}

impl CatalogStore {
    /// Create a store from an already validated catalog.
    pub fn new(catalog: Catalog) -> Self {
        Self {
            current: RwLock::new(Arc::new(catalog)),
        }
    }

    /// Acquire the current immutable catalog snapshot.
    pub fn snapshot(&self) -> Arc<Catalog> {
        Arc::clone(&self.current.read())
    }

    /// Load `model-catalog.toml` from the active config directory.
    ///
    /// The bundled catalog is used only if the local file does not exist. Any
    /// read or parse error leaves the currently published snapshot unchanged.
    pub fn load_from_dir(&self, config_dir: &Path) -> Result<()> {
        let path = config_dir.join(CATALOG_FILE);
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                BUNDLED_CATALOG.to_owned()
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("read model catalog {}", path.display()));
            }
        };
        let replacement = Catalog::parse(&contents)
            .with_context(|| format!("validate model catalog {}", path.display()))?;
        *self.current.write() = Arc::new(replacement);
        Ok(())
    }

    /// Reload the local catalog, retaining the last valid snapshot on failure.
    pub fn reload(&self, config_dir: &Path) -> Result<()> {
        self.load_from_dir(config_dir)
    }
}

/// Process-wide catalog initialized from the validated bundled document.
pub fn global() -> &'static Arc<CatalogStore> {
    static STORE: OnceLock<Arc<CatalogStore>> = OnceLock::new();
    STORE.get_or_init(|| {
        let bundled =
            Catalog::parse(BUNDLED_CATALOG).expect("the bundled model catalog must be valid");
        Arc::new(CatalogStore::new(bundled))
    })
}

/// Return the latest eligible recommended model for `provider`.
pub fn recommended_model(provider: &str) -> Option<String> {
    global().snapshot().recommended(provider, today())
}

/// Return the latest eligible curated model choices for `provider`.
pub fn curated_models(provider: &str) -> Vec<ModelEntry> {
    global().snapshot().curated(provider, today())
}

/// Return the latest exact-provider price pair for a model ID or explicit alias.
pub fn exact_pricing(provider: &str, model_id: &str) -> Option<(f64, f64)> {
    global().snapshot().pricing(provider, model_id, today())
}

/// Resolve an existing user selection or require a catalog default.
///
/// Configured selections are deliberately preserved even when they are
/// user-defined, dynamically discovered, or no longer in the curated list.
pub fn require_model(provider: &str, configured: Option<&str>) -> Result<String> {
    global()
        .snapshot()
        .require_model(provider, configured, today())
}

fn validate_text(field: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() || value.trim() != value {
        bail!("model catalog {field} must be non-empty and have no surrounding whitespace");
    }
    Ok(())
}

fn validate_price(provider: &str, id: &str, side: &str, price: Option<f64>) -> Result<()> {
    if let Some(price) = price
        && (!price.is_finite() || price < 0.0)
    {
        bail!("model {provider}/{id} has an invalid {side} price");
    }
    Ok(())
}

fn eligible(released: NaiveDate, as_of: NaiveDate) -> bool {
    let cutoff = as_of
        .checked_sub_months(Months::new(8))
        .unwrap_or(NaiveDate::MIN);
    released <= as_of && released >= cutoff
}

/// Date used by consumers that do not inject a clock.
pub fn today() -> NaiveDate {
    Utc::now().date_naive()
}

#[cfg(test)]
mod tests {
    use super::{Catalog, CatalogStore};
    use chrono::NaiveDate;
    use std::fs;
    use std::sync::{Arc, Barrier};
    use std::thread;

    const AS_OF: &str = "2026-11-09";

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    fn entry(id: &str, release: &str, recommended: bool) -> String {
        format!(
            "[[models]]\nprovider = \"vendor-a\"\nid = \"{id}\"\nname = \"Model {id}\"\nreleased = \"{release}\"\nrecommended = {recommended}\naliases = [\"{id}-alias\"]\ninput_price_per_million = 1.25\noutput_price_per_million = 4.5\nsources = [\"https://vendor.example/models\"]\nverified = \"2026-11-09\"\n"
        )
    }

    fn catalog(entries: &[String]) -> Catalog {
        Catalog::parse(&format!("schema_version = 1\n{}", entries.join("\n"))).unwrap()
    }

    #[test]
    fn parsing_requires_known_schema_and_validates_alias_uniqueness() {
        assert!(Catalog::parse("schema_version = 2\nmodels = []").is_err());
        assert!(Catalog::parse("schema_version = 1\nmodels = []\nextra = true").is_err());
        let duplicate = format!(
            "schema_version = 1\n{}{}",
            entry("same", "2026-03-10", false),
            entry("same", "2026-03-10", false)
        );
        assert!(Catalog::parse(&duplicate).is_err());

        let alias_collision = entry("model-a", "2026-03-10", false)
            .replace("aliases = [\"model-a-alias\"]", "aliases = [\"model-b\"]");
        let colliding_id = format!(
            "schema_version = 1\n{alias_collision}{}",
            entry("model-b", "2026-03-10", false)
        );
        assert!(Catalog::parse(&colliding_id).is_err());

        let invalid_price = entry("priced", "2026-03-10", false).replace(
            "input_price_per_million = 1.25",
            "input_price_per_million = inf",
        );
        assert!(Catalog::parse(&format!("schema_version = 1\n{invalid_price}")).is_err());

        let invalid_source = entry("invalid-source", "2026-03-10", false)
            .replace("https://vendor.example/models", "https://");
        assert!(Catalog::parse(&format!("schema_version = 1\n{invalid_source}")).is_err());

        let future_verified = entry("future-verified", "2026-03-10", false)
            .replace("verified = \"2026-11-09\"", "verified = \"2999-11-09\"");
        assert!(Catalog::parse(&format!("schema_version = 1\n{future_verified}")).is_err());
    }

    #[test]
    fn lookup_includes_calendar_cutoff_and_excludes_older_or_future_models() {
        let value = catalog(&[
            entry("boundary", "2026-03-09", true),
            entry("old", "2026-03-08", false),
        ]);
        let choices = value.curated("vendor-a", date(AS_OF));
        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].id, "boundary");
        assert_eq!(
            value.recommended("vendor-a", date(AS_OF)).as_deref(),
            Some("boundary")
        );
    }

    #[test]
    fn future_release_is_not_eligible_before_its_release_date() {
        let value = catalog(&[entry("scheduled", "2026-11-09", true)]);
        assert!(value.curated("vendor-a", date("2026-11-08")).is_empty());
        assert_eq!(value.recommended("vendor-a", date("2026-11-08")), None);
        assert_eq!(
            value.pricing("vendor-a", "scheduled", date("2026-11-08")),
            None
        );
    }

    #[test]
    fn eight_month_cutoff_clamps_to_last_day_of_shorter_month() {
        let value = catalog(&[
            entry("boundary", "2026-02-28", true),
            entry("old", "2026-02-27", false),
        ]);
        assert_eq!(value.curated("vendor-a", date("2026-10-31")).len(), 1);
    }

    #[test]
    fn no_eligible_default_is_explicitly_absent() {
        let value = catalog(&[entry("stale", "2025-01-01", true)]);
        assert_eq!(value.recommended("vendor-a", date(AS_OF)), None);
        let error = value
            .require_model("vendor-a", None, date(AS_OF))
            .unwrap_err()
            .to_string();
        assert!(error.contains("discover models live or enter a model ID"));
        assert_eq!(
            value
                .require_model("vendor-a", Some("custom-model-583"), date(AS_OF))
                .unwrap(),
            "custom-model-583"
        );
    }

    #[test]
    fn prices_resolve_only_exact_provider_ids_and_explicit_aliases() {
        let value = catalog(&[entry("model-a", "2026-03-09", false)]);
        assert_eq!(
            value.pricing("vendor-a", "model-a", date(AS_OF)),
            Some((1.25, 4.5))
        );
        assert_eq!(
            value.pricing("vendor-a", "model-a-alias", date(AS_OF)),
            Some((1.25, 4.5))
        );
        assert_eq!(value.pricing("vendor-b", "model-a", date(AS_OF)), None);
        assert_eq!(
            value.pricing("vendor-a-codex", "model-a", date(AS_OF)),
            None
        );
        assert_eq!(
            value.pricing("vendor-a", "model-a-related", date(AS_OF)),
            None
        );
        assert_eq!(
            value.pricing("vendor-a", "model-a", date("2026-03-08")),
            None
        );
    }

    #[test]
    fn missing_local_file_loads_bundle_and_invalid_file_keeps_snapshot() {
        let directory = tempfile::tempdir().unwrap();
        let store = CatalogStore::new(catalog(&[entry("before", "2026-03-09", true)]));
        let old = store.snapshot();
        store.load_from_dir(directory.path()).unwrap();
        assert!(store.snapshot().curated("vendor-a", date(AS_OF)).is_empty());

        fs::write(
            directory.path().join("model-catalog.toml"),
            "schema_version = 1\nmodels = [",
        )
        .unwrap();
        let bundled = store.snapshot();
        let error = store.reload(directory.path()).unwrap_err().to_string();
        assert!(error.contains("model-catalog.toml"));
        assert!(Arc::ptr_eq(&bundled, &store.snapshot()));
        assert!(!Arc::ptr_eq(&old, &store.snapshot()));
    }

    #[test]
    fn successful_reload_publishes_new_snapshot_without_changing_acquired_one() {
        let directory = tempfile::tempdir().unwrap();
        let store = CatalogStore::new(catalog(&[entry("before", "2026-03-09", true)]));
        let acquired = store.snapshot();
        fs::write(
            directory.path().join("model-catalog.toml"),
            format!("schema_version = 1\n{}", entry("after", "2026-03-09", true)),
        )
        .unwrap();
        store.reload(directory.path()).unwrap();
        assert_eq!(
            acquired.recommended("vendor-a", date(AS_OF)).as_deref(),
            Some("before")
        );
        assert_eq!(
            store
                .snapshot()
                .recommended("vendor-a", date(AS_OF))
                .as_deref(),
            Some("after")
        );
    }

    #[test]
    fn concurrent_readers_observe_complete_snapshots_during_reload() {
        let directory = tempfile::tempdir().unwrap();
        let store = Arc::new(CatalogStore::new(catalog(&[entry(
            "model-a",
            "2026-03-09",
            true,
        )])));
        let barrier = Arc::new(Barrier::new(5));
        let mut readers = Vec::new();
        for _ in 0..4 {
            let store = Arc::clone(&store);
            let barrier = Arc::clone(&barrier);
            readers.push(thread::spawn(move || {
                barrier.wait();
                for _ in 0..200 {
                    let snapshot = store.snapshot();
                    let selected = snapshot.recommended("vendor-a", date(AS_OF)).unwrap();
                    assert!(selected == "model-a" || selected == "model-b");
                    assert!(
                        snapshot
                            .pricing("vendor-a", &selected, date(AS_OF))
                            .is_some()
                    );
                }
            }));
        }
        barrier.wait();
        for (id, _) in [("model-b", true), ("model-a", true)] {
            fs::write(
                directory.path().join("model-catalog.toml"),
                format!("schema_version = 1\n{}", entry(id, "2026-03-09", true)),
            )
            .unwrap();
            store.reload(directory.path()).unwrap();
        }
        for reader in readers {
            reader.join().unwrap();
        }
    }
}
