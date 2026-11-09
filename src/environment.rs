//! Safe, in-memory environment sources for configuration and runtime code.
//!
//! Real process values take precedence over workspace `.env` values. Managed
//! proxy overlays are separate and only affect Gloamy environment reads and
//! explicitly allowlisted child processes; they never mutate the parent.

use std::collections::HashMap;
use std::env::VarError;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::{OnceLock, RwLock};

#[derive(Clone, Debug, Default)]
pub struct EnvironmentSource {
    explicit: HashMap<OsString, OsString>,
    process: HashMap<OsString, OsString>,
    dotenv: HashMap<OsString, OsString>,
    masked: std::collections::HashSet<OsString>,
}

impl EnvironmentSource {
    /// Snapshot the real process environment without mutating it.
    pub fn from_process() -> Self {
        let mut process: HashMap<OsString, OsString> = std::env::vars_os().collect();
        let config_dir_override = match runtime_environment().read() {
            Ok(environment) => environment.config_dir_override.clone(),
            Err(poisoned) => poisoned.into_inner().config_dir_override.clone(),
        };
        if let Some(value) = config_dir_override {
            process.insert(OsString::from("GLOAMY_CONFIG_DIR"), value);
        }
        #[allow(unused_mut)]
        let mut source = Self {
            explicit: HashMap::new(),
            process,
            dotenv: HashMap::new(),
            masked: std::collections::HashSet::new(),
        };
        #[cfg(test)]
        TEST_OVERRIDES.with(|overrides| {
            for (key, value) in overrides.borrow().iter() {
                match value {
                    Some(value) => {
                        source.process.insert(key.clone(), value.clone());
                    }
                    None => {
                        source.process.remove(key);
                        source.masked.insert(key.clone());
                    }
                }
            }
        });
        source
    }

    /// Create a source from explicit maps. Primarily useful for deterministic tests.
    pub fn from_maps(
        process: HashMap<OsString, OsString>,
        dotenv: HashMap<OsString, OsString>,
    ) -> Self {
        Self {
            explicit: HashMap::new(),
            process,
            dotenv,
            masked: std::collections::HashSet::new(),
        }
    }

    /// Add dotenv defaults. Existing process values always win.
    pub fn with_dotenv(mut self, values: HashMap<OsString, OsString>) -> Self {
        self.dotenv = values;
        self
    }

    fn with_explicit(mut self, key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        self.explicit
            .insert(key.as_ref().to_os_string(), value.as_ref().to_os_string());
        self
    }

    pub fn var_os(&self, key: &OsStr) -> Option<OsString> {
        self.explicit
            .get(key)
            .or_else(|| self.process.get(key))
            .or_else(|| {
                (!self.masked.contains(key))
                    .then(|| self.dotenv.get(key))
                    .flatten()
            })
            .cloned()
    }

    pub fn var(&self, key: &str) -> Result<String, VarError> {
        let Some(value) = self.var_os(OsStr::new(key)) else {
            return Err(VarError::NotPresent);
        };
        value.into_string().map_err(VarError::NotUnicode)
    }

    pub fn dotenv_values(&self) -> HashMap<OsString, OsString> {
        self.dotenv.clone()
    }

    pub fn process_values(&self) -> HashMap<OsString, OsString> {
        self.process.clone()
    }
}

#[derive(Default)]
struct RuntimeEnvironment {
    dotenv: HashMap<OsString, OsString>,
    config_dir_override: Option<OsString>,
    /// `Some(value)` overrides an inherited value; `None` masks it.
    proxy_overlay: HashMap<OsString, Option<OsString>>,
}

static RUNTIME_ENVIRONMENT: OnceLock<RwLock<RuntimeEnvironment>> = OnceLock::new();

fn runtime_environment() -> &'static RwLock<RuntimeEnvironment> {
    RUNTIME_ENVIRONMENT.get_or_init(|| RwLock::new(RuntimeEnvironment::default()))
}

/// Parse dotenv assignments without evaluating shell syntax or changing process state.
pub fn parse_dotenv(contents: &str) -> HashMap<OsString, OsString> {
    let mut values = HashMap::new();
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").map(str::trim).unwrap_or(line);
        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        let raw_value = strip_dotenv_comment(raw_value.trim());
        let value = if raw_value.len() >= 2
            && ((raw_value.starts_with('"') && raw_value.ends_with('"'))
                || (raw_value.starts_with('\'') && raw_value.ends_with('\'')))
        {
            &raw_value[1..raw_value.len() - 1]
        } else {
            raw_value
        };
        values.insert(OsString::from(key), OsString::from(value));
    }
    values
}

fn strip_dotenv_comment(value: &str) -> &str {
    let mut quote = None;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match (quote, character) {
            (Some('"'), '\\') => escaped = true,
            (Some(current), character) if current == character => quote = None,
            (None, '"' | '\'') => quote = Some(character),
            (None, '#')
                if value[..index]
                    .chars()
                    .next_back()
                    .is_some_and(char::is_whitespace) =>
            {
                return value[..index].trim_end();
            }
            _ => {}
        }
    }
    value.trim_end()
}

pub fn parse_dotenv_file(path: &Path) -> std::io::Result<HashMap<OsString, OsString>> {
    let contents = std::fs::read_to_string(path)?;
    Ok(parse_dotenv(&contents))
}

/// Install workspace dotenv values for runtime components. Process values still win.
pub fn install_dotenv(values: HashMap<OsString, OsString>) {
    match runtime_environment().write() {
        Ok(mut environment) => environment.dotenv = values,
        Err(poisoned) => poisoned.into_inner().dotenv = values,
    }
}

/// A configuration snapshot that includes process values, installed dotenv values,
/// and test-local overrides, but not the managed proxy overlay.
pub fn runtime_source() -> EnvironmentSource {
    let mut source = EnvironmentSource::from_process();
    let dotenv = match runtime_environment().read() {
        Ok(environment) => environment.dotenv.clone(),
        Err(poisoned) => poisoned.into_inner().dotenv.clone(),
    };
    source = source.with_dotenv(dotenv);
    #[cfg(test)]
    TEST_OVERRIDES.with(|overrides| {
        for (key, value) in overrides.borrow().iter() {
            match value {
                Some(value) => {
                    source.process.insert(key.clone(), value.clone());
                }
                None => {
                    source.process.remove(key);
                    source.dotenv.remove(key);
                    source.masked.insert(key.clone());
                }
            }
        }
    });
    let config_dir_override = match runtime_environment().read() {
        Ok(environment) => environment.config_dir_override.clone(),
        Err(poisoned) => poisoned.into_inner().config_dir_override.clone(),
    };
    if let Some(value) = config_dir_override {
        source = source.with_explicit("GLOAMY_CONFIG_DIR", value);
    }
    source
}

/// Store the explicit `--config-dir` CLI option without writing to process env.
pub fn set_config_dir_override(value: Option<&OsStr>) {
    match runtime_environment().write() {
        Ok(mut environment) => environment.config_dir_override = value.map(OsStr::to_os_string),
        Err(poisoned) => {
            poisoned.into_inner().config_dir_override = value.map(OsStr::to_os_string);
        }
    }
}

pub fn var_os(key: impl AsRef<OsStr>) -> Option<OsString> {
    let key = key.as_ref();
    let runtime = match runtime_environment().read() {
        Ok(environment) => environment,
        Err(poisoned) => poisoned.into_inner(),
    };
    if key == OsStr::new("GLOAMY_CONFIG_DIR") {
        if let Some(value) = &runtime.config_dir_override {
            return Some(value.clone());
        }
    }
    if let Some(value) = runtime.proxy_overlay.get(key) {
        return value.clone();
    }
    #[cfg(test)]
    if let Some(value) = TEST_OVERRIDES.with(|overrides| overrides.borrow().get(key).cloned()) {
        return value;
    }
    std::env::var_os(key).or_else(|| runtime.dotenv.get(key).cloned())
}

pub fn var(key: &str) -> Result<String, VarError> {
    let Some(value) = var_os(OsStr::new(key)) else {
        return Err(VarError::NotPresent);
    };
    value.into_string().map_err(VarError::NotUnicode)
}

/// Return the effective environment for a child command. Callers must still apply
/// their own allowlist before passing entries to `Command::env`.
pub fn vars_os() -> std::vec::IntoIter<(OsString, OsString)> {
    let mut values: HashMap<OsString, OsString> = std::env::vars_os().collect();
    let runtime = match runtime_environment().read() {
        Ok(environment) => environment,
        Err(poisoned) => poisoned.into_inner(),
    };
    for (key, value) in &runtime.dotenv {
        values.entry(key.clone()).or_insert_with(|| value.clone());
    }
    if let Some(value) = &runtime.config_dir_override {
        values.insert(OsString::from("GLOAMY_CONFIG_DIR"), value.clone());
    }
    for (key, value) in &runtime.proxy_overlay {
        match value {
            Some(value) => {
                values.insert(key.clone(), value.clone());
            }
            None => {
                values.remove(key);
            }
        }
    }
    #[cfg(test)]
    TEST_OVERRIDES.with(|overrides| {
        for (key, value) in overrides.borrow().iter() {
            match value {
                Some(value) => {
                    values.insert(key.clone(), value.clone());
                }
                None => {
                    values.remove(key);
                }
            }
        }
    });
    values.into_iter().collect::<Vec<_>>().into_iter()
}

/// Apply a Gloamy-owned proxy overlay. Missing values mask inherited proxy vars,
/// matching the old `apply_env` behavior without changing the parent process.
pub fn apply_proxy_overlay(values: &[(String, Option<String>)]) {
    let mut overlay = HashMap::new();
    for (key, value) in values {
        overlay.insert(OsString::from(key), value.clone().map(OsString::from));
        overlay.insert(
            OsString::from(key.to_ascii_lowercase()),
            value.clone().map(OsString::from),
        );
    }
    match runtime_environment().write() {
        Ok(mut environment) => environment.proxy_overlay = overlay,
        Err(poisoned) => poisoned.into_inner().proxy_overlay = overlay,
    }
}

pub fn clear_proxy_overlay() {
    match runtime_environment().write() {
        Ok(mut environment) => environment.proxy_overlay.clear(),
        Err(poisoned) => poisoned.into_inner().proxy_overlay.clear(),
    }
}

/// Values currently owned by Gloamy's proxy overlay, for status reporting.
pub fn proxy_overlay_snapshot() -> HashMap<String, Option<String>> {
    let runtime = match runtime_environment().read() {
        Ok(environment) => environment,
        Err(poisoned) => poisoned.into_inner(),
    };
    runtime
        .proxy_overlay
        .iter()
        .filter_map(|(key, value)| {
            let key = key.to_str()?;
            if key != key.to_ascii_uppercase() {
                return None;
            }
            Some((
                key.to_string(),
                value
                    .as_ref()
                    .map(|value| value.to_string_lossy().into_owned()),
            ))
        })
        .collect()
}

#[cfg(test)]
thread_local! {
    static TEST_OVERRIDES: std::cell::RefCell<HashMap<OsString, Option<OsString>>> =
        std::cell::RefCell::new(HashMap::new());
}

#[cfg(test)]
pub mod test {
    use super::{OsStr, OsString, TEST_OVERRIDES};
    use std::sync::{Mutex, MutexGuard, OnceLock};

    static ENVIRONMENT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    pub struct ScopedVar {
        key: OsString,
        previous: Option<Option<OsString>>,
    }

    impl Drop for ScopedVar {
        fn drop(&mut self) {
            TEST_OVERRIDES.with(|overrides| {
                let mut overrides = overrides.borrow_mut();
                if let Some(previous) = self.previous.take() {
                    overrides.insert(self.key.clone(), previous);
                } else {
                    overrides.remove(&self.key);
                }
            });
        }
    }

    pub fn lock() -> MutexGuard<'static, ()> {
        ENVIRONMENT_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("environment test lock should not be poisoned")
    }

    pub fn set_var(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) {
        TEST_OVERRIDES.with(|overrides| {
            overrides.borrow_mut().insert(
                key.as_ref().to_os_string(),
                Some(value.as_ref().to_os_string()),
            );
        });
    }

    pub fn remove_var(key: impl AsRef<OsStr>) {
        TEST_OVERRIDES.with(|overrides| {
            overrides
                .borrow_mut()
                .insert(key.as_ref().to_os_string(), None);
        });
    }

    pub fn scoped_set_var(key: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> ScopedVar {
        let key = key.as_ref().to_os_string();
        let previous = TEST_OVERRIDES.with(|overrides| {
            overrides
                .borrow_mut()
                .insert(key.clone(), Some(value.as_ref().to_os_string()))
        });
        ScopedVar { key, previous }
    }

    pub fn scoped_remove_var(key: impl AsRef<OsStr>) -> ScopedVar {
        let key = key.as_ref().to_os_string();
        let previous =
            TEST_OVERRIDES.with(|overrides| overrides.borrow_mut().insert(key.clone(), None));
        ScopedVar { key, previous }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_environment_precedes_dotenv_values() {
        let source = EnvironmentSource::from_maps(
            HashMap::from([(OsString::from("TOKEN"), OsString::from("process"))]),
            HashMap::from([
                (OsString::from("TOKEN"), OsString::from("dotenv")),
                (OsString::from("OTHER"), OsString::from("dotenv-only")),
            ]),
        );
        assert_eq!(source.var("TOKEN").unwrap(), "process");
        assert_eq!(source.var("OTHER").unwrap(), "dotenv-only");
    }

    #[test]
    fn dotenv_parser_supports_export_quotes_and_comments() {
        let values = parse_dotenv(
            "# comment\nexport TOKEN=\"quoted value\" # comment\nPLAIN=value # comment\nHASH=\"#literal value\"\n",
        );
        assert_eq!(values[OsStr::new("TOKEN")], OsStr::new("quoted value"));
        assert_eq!(values[OsStr::new("PLAIN")], OsStr::new("value"));
        assert_eq!(values[OsStr::new("HASH")], OsStr::new("#literal value"));
    }
}
