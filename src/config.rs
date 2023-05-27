/*
 *  Configuration parameter management for eng-clock
 *  RW Penney, May 2023
 */

use dirs;
use serde::Deserialize;
use std::path::Path;
use toml;


#[derive(Debug)]
pub enum ConfigReadError {
    /// A filesystem read error
    IoError(std::io::Error),

    /// A failure to parse the configuration data
    TomlError(toml::de::Error),

    /// A failure to identify OS-specific config-directory location
    UnknownHome
}

impl From<std::io::Error> for ConfigReadError {
    fn from(e: std::io::Error) -> Self {
        ConfigReadError::IoError(e)
    }
}


const DEFAULT_NTP_SERVERS: [&str; 4] = [
    "0.pool.ntp.org",
    "1.pool.ntp.org",
    "2.pool.ntp.org",
    "3.pool.ntp.org"
];


#[derive(Clone, Debug, Deserialize)]
pub struct SyncConfig {
    /// A collection of NTP hostnames
    pub ntp_servers: Vec<String>,

    /// The desired margin of error in the estimate clock-offset, in seconds
    #[serde(default = "SyncConfig::default_tgt_precision")]
    pub target_precision: f32
}

impl SyncConfig {
    const DEFAULT_TGT_PRECISION: f32 = 0.03;

    fn default_tgt_precision() -> f32 {
        SyncConfig::DEFAULT_TGT_PRECISION
    }

    pub fn default() -> SyncConfig {
        SyncConfig {
            ntp_servers:
                DEFAULT_NTP_SERVERS.into_iter()
                                   .map(|h| String::from(h)).collect(),
            target_precision: SyncConfig::DEFAULT_TGT_PRECISION
        }
    }
}


#[derive(Clone, Debug, Deserialize)]
pub struct ECConfig {
    pub sync: SyncConfig
}

impl ECConfig {
    const CFG_FILENAME: &str = "eng-clock.toml";

    /// Create a configuration parameters from a built-in global list of NTP servers
    pub fn default() -> ECConfig {
        ECConfig {
            sync: SyncConfig::default()
        }
    }

    /// Read configuration settings from a TOML file
    ///
    /// # Example
    /// ```
    /// use eng_clock::config::ECConfig;
    /// let cfg = ECConfig::from_toml(r#"
    ///         [sync]
    ///         ntp_servers = [
    ///         "1.africa.pool.ntp.org", "1.asia.pool.ntp.org",
    ///         "1.europe.pool.ntp.org", "1.north-america.pool.ntp.org",
    ///         "1.oceania.pool.ntp.org", "1.south-america.pool.ntp.org" ]"#).unwrap();
    /// assert_eq!(cfg.sync.ntp_servers.len(), 6);
    /// ```
    pub fn from_toml(s: &str) -> Result<ECConfig, ConfigReadError> {
        toml::from_str::<ECConfig>(s)
            .map_err(|e| ConfigReadError::TomlError(e))
    }

    /// Read TOML configuration settings from a supplied filesystem path
    pub fn from_path(path: &Path) -> Result<ECConfig, ConfigReadError> {
        let raw = std::fs::read(path)?;
        let doc = String::from_utf8_lossy(&raw);

        ECConfig::from_toml(&doc)
    }

    /// Read TOML configuration settings from a well-known location
    pub fn from_user_config() -> Result<ECConfig, ConfigReadError> {
        match dirs::config_dir() {
            Some(mut path) => {
                path.push(ECConfig::CFG_FILENAME);
                ECConfig::from_path(&path)
            },
            None => Err(ConfigReadError::UnknownHome)
        }
    }
}

// (C)Copyright 2023, RW Penney
