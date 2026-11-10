use super::traits::{Observer, ObserverEvent, ObserverMetric};
use crate::config::schema::ModelPricing;
use crate::cost::CostTracker;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Observer wrapper that intercepts `LlmResponse` events and records token
/// usage to a [`CostTracker`], then delegates all events to an inner observer.
pub struct CostTrackingObserver {
    inner: Arc<dyn Observer>,
    cost_tracker: Arc<CostTracker>,
    prices: HashMap<String, ModelPricing>,
    catalog: Arc<crate::model_catalog::CatalogStore>,
}

impl CostTrackingObserver {
    pub fn new(
        inner: Arc<dyn Observer>,
        cost_tracker: Arc<CostTracker>,
        prices: HashMap<String, ModelPricing>,
    ) -> Self {
        Self {
            inner,
            cost_tracker,
            prices,
            catalog: Arc::clone(crate::model_catalog::global()),
        }
    }

    fn model_prices(&self, provider: &str, model: &str) -> (f64, f64) {
        let canonical_provider = crate::providers::canonical_provider_name(provider);
        // Explicit provider/model keys win. A slash-containing model ID must
        // use a qualified key so another provider's override cannot leak into it.
        let price = self
            .prices
            .get(&format!("{provider}/{model}"))
            .or_else(|| self.prices.get(&format!("{canonical_provider}/{model}")))
            .or_else(|| {
                if model.contains('/') {
                    None
                } else {
                    self.prices.get(model)
                }
            });
        if let Some(price) = price {
            return (price.input, price.output);
        }
        self.catalog
            .snapshot()
            .pricing(canonical_provider, model, chrono::Utc::now().date_naive())
            .unwrap_or((0.0, 0.0))
    }
}

impl Observer for CostTrackingObserver {
    fn record_event(&self, event: &ObserverEvent) {
        if let ObserverEvent::LlmResponse {
            provider,
            model,
            input_tokens,
            output_tokens,
            success,
            ..
        } = event
        {
            if *success {
                let input = input_tokens.unwrap_or(0);
                let output = output_tokens.unwrap_or(0);
                if input > 0 || output > 0 {
                    let (input_price, output_price) = self.model_prices(provider, model);
                    let usage = crate::cost::TokenUsage::new(
                        model,
                        input,
                        output,
                        input_price,
                        output_price,
                    );
                    if let Err(e) = self.cost_tracker.record_usage(usage) {
                        tracing::warn!("Failed to record cost usage: {e}");
                    }
                }
            }
        }
        self.inner.record_event(event);
    }

    fn record_metric(&self, metric: &ObserverMetric) {
        self.inner.record_metric(metric);
    }

    fn flush(&self) {
        self.inner.flush();
    }

    fn name(&self) -> &str {
        "cost-tracking"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observability::NoopObserver;
    use std::time::Duration;
    use tempfile::TempDir;

    fn observer_with_prices(
        tracker: Arc<CostTracker>,
        prices: HashMap<String, ModelPricing>,
    ) -> CostTrackingObserver {
        let inner: Arc<dyn Observer> = Arc::new(NoopObserver);
        CostTrackingObserver::new(inner, tracker, prices)
    }

    fn test_tracker() -> (TempDir, Arc<CostTracker>) {
        let tmp = TempDir::new().unwrap();
        let config = crate::config::schema::CostConfig {
            enabled: true,
            ..Default::default()
        };
        let tracker = CostTracker::new(config, tmp.path()).unwrap();
        (tmp, Arc::new(tracker))
    }

    #[test]
    fn records_usage_on_successful_llm_response() {
        let (_tmp, tracker) = test_tracker();
        let mut prices = HashMap::new();
        prices.insert(
            "openai/synthetic-model-032".to_string(),
            ModelPricing {
                input: 2.0,
                output: 4.0,
            },
        );
        let obs = observer_with_prices(Arc::clone(&tracker), prices);

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "synthetic-model-032".into(),
            duration: Duration::from_millis(500),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
        assert!((summary.session_cost_usd - 0.0004).abs() < 1e-10);
    }

    #[test]
    fn skips_failed_llm_responses() {
        let (_tmp, tracker) = test_tracker();
        let obs = observer_with_prices(Arc::clone(&tracker), HashMap::new());

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "synthetic-model-032".into(),
            duration: Duration::from_millis(100),
            success: false,
            error_message: Some("rate limited".into()),
            input_tokens: Some(100),
            output_tokens: None,
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 0);
        assert_eq!(summary.request_count, 0);
    }

    #[test]
    fn delegates_all_events_to_inner() {
        let (_tmp, tracker) = test_tracker();
        let obs = observer_with_prices(Arc::clone(&tracker), HashMap::new());

        // Should not panic on non-LLM events
        obs.record_event(&ObserverEvent::HeartbeatTick);
        obs.record_metric(&ObserverMetric::TokensUsed(42));
        obs.flush();
        assert_eq!(obs.name(), "cost-tracking");
    }

    #[test]
    fn falls_back_to_zero_price_when_model_is_unmapped() {
        let (_tmp, tracker) = test_tracker();
        let obs = observer_with_prices(Arc::clone(&tracker), HashMap::new());

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "synthetic-model-034".into(),
            duration: Duration::from_millis(300),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
        assert!(summary.session_cost_usd.abs() < f64::EPSILON);
    }

    #[test]
    fn does_not_infer_snapshot_suffix_prices() {
        let (_tmp, tracker) = test_tracker();
        let mut prices = HashMap::new();
        prices.insert(
            "openai/synthetic-model-040".to_string(),
            ModelPricing {
                input: 0.75,
                output: 4.5,
            },
        );
        let obs = observer_with_prices(Arc::clone(&tracker), prices);

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "synthetic-model-041".into(),
            duration: Duration::from_millis(300),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
        assert_eq!(summary.session_cost_usd, 0.0);
    }

    #[test]
    fn does_not_infer_provider_prefixed_snapshot_prices() {
        let (_tmp, tracker) = test_tracker();
        let mut prices = HashMap::new();
        prices.insert(
            "openai/synthetic-model-040".to_string(),
            ModelPricing {
                input: 0.75,
                output: 4.5,
            },
        );
        let obs = observer_with_prices(Arc::clone(&tracker), prices);

        obs.record_event(&ObserverEvent::LlmResponse {
            provider: "openai".into(),
            model: "openai/synthetic-model-041".into(),
            duration: Duration::from_millis(300),
            success: true,
            error_message: None,
            input_tokens: Some(100),
            output_tokens: Some(50),
        });

        let summary = tracker.get_summary().unwrap();
        assert_eq!(summary.total_tokens, 150);
        assert_eq!(summary.request_count, 1);
        assert_eq!(summary.session_cost_usd, 0.0);
    }
    #[test]
    fn qualified_override_does_not_leak_to_subscription_provider() {
        let (_tmp, tracker) = test_tracker();
        let prices = HashMap::from([
            (
                "openai/synthetic-model".into(),
                ModelPricing {
                    input: 2.0,
                    output: 4.0,
                },
            ),
            (
                "gemini/synthetic-model".into(),
                ModelPricing {
                    input: 3.0,
                    output: 5.0,
                },
            ),
        ]);
        let observer = observer_with_prices(tracker, prices);
        assert_eq!(
            observer.model_prices("openai", "synthetic-model"),
            (2.0, 4.0)
        );
        assert_eq!(
            observer.model_prices("openai-codex", "synthetic-model"),
            (0.0, 0.0)
        );
        assert_eq!(
            observer.model_prices("openai-codex", "openai/synthetic-model"),
            (0.0, 0.0)
        );
        assert_eq!(
            observer.model_prices("google", "synthetic-model"),
            (3.0, 5.0)
        );
    }

    #[test]
    fn subsequent_lookups_observe_reloads_and_overrides_win() {
        let (_tmp, tracker) = test_tracker();
        let directory = TempDir::new().unwrap();
        let path = directory.path().join("model-catalog.toml");
        let date = chrono::Utc::now().date_naive();
        let document = |rate: f64| {
            format!(
                r#"schema_version = 1
[[models]]
provider = "provider-a"
id = "synthetic-model"
name = "Synthetic model"
released = "{date}"
verified = "{date}"
sources = ["https://example.com/release"]
input_price_per_million = {rate}
output_price_per_million = 4.0
"#
            )
        };
        let store = Arc::new(crate::model_catalog::CatalogStore::new(
            crate::model_catalog::Catalog::parse(&document(2.0)).unwrap(),
        ));
        let mut observer = observer_with_prices(tracker, HashMap::new());
        observer.catalog = Arc::clone(&store);
        assert_eq!(
            observer.model_prices("provider-a", "synthetic-model"),
            (2.0, 4.0)
        );
        std::fs::write(&path, document(3.0)).unwrap();
        store.reload(directory.path()).unwrap();
        assert_eq!(
            observer.model_prices("provider-a", "synthetic-model"),
            (3.0, 4.0)
        );
        observer.prices.insert(
            "synthetic-model".into(),
            ModelPricing {
                input: 8.0,
                output: 9.0,
            },
        );
        observer.prices.insert(
            "provider-a/synthetic-model".into(),
            ModelPricing {
                input: 5.0,
                output: 6.0,
            },
        );
        assert_eq!(
            observer.model_prices("provider-a", "synthetic-model"),
            (5.0, 6.0)
        );
        assert_eq!(
            observer.model_prices("provider-b", "synthetic-model"),
            (8.0, 9.0)
        );
        observer.prices.clear();
        assert_eq!(
            observer.model_prices("openai-codex", "synthetic-model"),
            (0.0, 0.0)
        );
        std::fs::write(path, "invalid").unwrap();
        assert!(store.reload(directory.path()).is_err());
        assert_eq!(
            observer.model_prices("provider-a", "synthetic-model"),
            (3.0, 4.0)
        );
    }
}
