pub mod wizard;

// Re-exported for CLI and external use
#[allow(unused_imports)]
pub use wizard::{
    OnboardingOutcome, run_channels_repair_wizard, run_channels_repair_wizard_with_outcome,
    run_models_list, run_models_refresh, run_models_refresh_all, run_models_set, run_models_status,
    run_quick_setup, run_wizard, run_wizard_with_outcome,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_reexport_exists<F>(_value: F) {}

    #[test]
    fn wizard_functions_are_reexported() {
        assert_reexport_exists(run_wizard);
        assert_reexport_exists(run_channels_repair_wizard);
        assert_reexport_exists(run_quick_setup);
        assert_reexport_exists(run_models_refresh);
        assert_reexport_exists(run_models_list);
        assert_reexport_exists(run_models_set);
        assert_reexport_exists(run_models_status);
        assert_reexport_exists(run_models_refresh_all);
    }
}
