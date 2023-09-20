use std::fs;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_refresh")]
    pub refresh: u64,
    #[serde(default = "default_low")]
    pub low: u8,
    #[serde(default = "default_critical")]
    pub critical: u8,
    pub locker: Option<String>,
}

const DEFAULT_LOW: u8 = 15;
const DEFAULT_CRITICAL: u8 = 10;
const DEFAULT_REFRESH: u64 = 10;

const fn default_low() -> u8 {
    DEFAULT_LOW
}

const fn default_critical() -> u8 {
    DEFAULT_CRITICAL
}

const fn default_refresh() -> u64 {
    DEFAULT_REFRESH
}

impl Default for Config {
    fn default() -> Self {
        Self {
            low: DEFAULT_LOW,
            critical: DEFAULT_CRITICAL,
            refresh: DEFAULT_REFRESH,
            locker: None,
        }
    }
}

// Typically, the XDG_CONFIG_HOME environment variable
// should be used for this, but many Linux users with
// a barebones WM will not have this variable set due
// to the lack of a DE. Hardcoding the path is a safer
// and saner option.
const CONFIG_PATH: &str = ".config/batman/config.toml";

impl Config {
    pub fn get() -> Result<Self> {
        toml::from_str(&fs::read_to_string(CONFIG_PATH).unwrap_or_else(|_| {
            eprintln!("config file does not exist, falling back to defaults");
            String::default()
        }))
        .context("invalid config, please check the TOML")
    }

    pub fn validate(&mut self) {
        self.low = self.validate_low().unwrap_or_else(|e| {
            eprintln!("{e}");
            DEFAULT_LOW
        });
        self.critical = self.validate_critical().unwrap_or_else(|e| {
            eprintln!("{e}");
            DEFAULT_CRITICAL
        });
    }

    fn validate_low(&self) -> Result<u8> {
        if self.low < 5 || self.low > 95 {
            bail!("value must be between 5 and 95")
        } else if self.low < self.critical {
            bail!(
                "value {} of `low` cannot be less than value {} of `critical`",
                self.low,
                self.critical
            )
        }
        Ok(self.low)
    }

    fn validate_critical(&self) -> Result<u8> {
        if self.critical < 5 || self.critical > 95 {
            bail!("value must be between 5 and 95")
        } else if self.critical > self.low {
            bail!(
                "value {} of `critical` cannot be greater than value {} of `low`",
                self.critical,
                self.low
            )
        }
        Ok(self.critical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trivial() {
        let config: Config = toml::from_str(
            r#"
        low = 10
        critical = 5
        refresh = 10
        locker = "i3lock"
            "#,
        )
        .unwrap();
        assert_eq!(config.low, 10);
        assert_eq!(config.critical, 5);
        assert_eq!(config.refresh, 10);
        assert_eq!(config.locker, Some("i3lock".to_string()));
    }

    #[test]
    fn default() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.low, DEFAULT_LOW);
        assert_eq!(config.critical, DEFAULT_CRITICAL);
        assert_eq!(config.refresh, DEFAULT_REFRESH);
        assert_eq!(config.locker, None);
    }

    #[test]
    fn no_locker() {
        let config: Config = toml::from_str(
            r#"
        low = 10
        critical = 5
            "#,
        )
        .unwrap();
        assert_eq!(config.low, 10);
        assert_eq!(config.critical, 5);
        assert_eq!(config.locker, None);
    }
}
