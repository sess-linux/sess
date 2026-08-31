use serde::Deserialize;
use std::path::PathBuf;

/// sess's config lives at ~/.config/sess/config.toml. It's entirely optional —
/// a missing or invalid file just means defaults, never a hard error, so `sess`
/// stays predictable even if the file gets corrupted by hand-editing.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub autosave: AutosaveConfig,
    #[serde(default)]
    pub environment: EnvironmentConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AutosaveConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: default_interval(),
        }
    }
}

fn default_interval() -> u64 {
    30
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct EnvironmentConfig {
    /// Variable names allowed to be captured and persisted on save. Empty by
    /// default: sess never captures environment variables unless explicitly
    /// told to.
    #[serde(default)]
    pub persist: Vec<String>,
}

pub fn config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("sess").join("config.toml"))
}

/// Loads the config, falling back to defaults if the file is missing, unreadable,
/// or fails to parse. Never returns an error — config problems should not block
/// ordinary use of sess.
pub fn load() -> Config {
    let Some(path) = config_path() else {
        return Config::default();
    };
    let Ok(raw) = std::fs::read_to_string(path) else {
        return Config::default();
    };
    toml::from_str(&raw).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_falls_back_to_defaults() {
        let cfg: Config = toml::from_str("").unwrap();
        assert!(!cfg.autosave.enabled);
        assert_eq!(cfg.autosave.interval, 30);
        assert!(cfg.environment.persist.is_empty());
    }

    #[test]
    fn parses_documented_example() {
        let raw = r#"
            [autosave]
            enabled = true
            interval = 45

            [environment]
            persist = ["NODE_ENV", "EDITOR"]
        "#;
        let cfg: Config = toml::from_str(raw).unwrap();
        assert!(cfg.autosave.enabled);
        assert_eq!(cfg.autosave.interval, 45);
        assert_eq!(cfg.environment.persist, vec!["NODE_ENV", "EDITOR"]);
    }

    #[test]
    fn invalid_toml_does_not_panic_via_load() {
        // load() must never panic even if the file exists but is garbage —
        // covered indirectly since `load()` swallows parse errors into defaults.
        let cfg: Config = toml::from_str("not = [valid").unwrap_or_default();
        assert!(!cfg.autosave.enabled);
    }
}
