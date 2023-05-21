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
    IoError(std::io::Error),
    TomlError(toml::de::Error),
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
pub struct ECConfig {
    pub(crate) ntp_servers: Vec<String>,
    pub(crate) target_precision: f32
}

impl ECConfig {
    const CFG_FILENAME: &str = "eng-clock.toml";
    const DEFAULT_TGT_PRECISION: f32 = 0.03;

    pub fn default() -> ECConfig {
        ECConfig {
            ntp_servers:
                DEFAULT_NTP_SERVERS.into_iter()
                                   .map(|h| String::from(h)).collect(),
            target_precision: ECConfig::DEFAULT_TGT_PRECISION
        }
    }

    pub fn from_toml(s: &str) -> Result<ECConfig, ConfigReadError> {
        toml::from_str::<ECConfig>(s)
            .map_err(|e| ConfigReadError::TomlError(e))
    }

    pub fn from_path(path: &Path) -> Result<ECConfig, ConfigReadError> {
        let raw = std::fs::read(path)?;
        let doc = String::from_utf8_lossy(&raw);

        ECConfig::from_toml(&doc)
    }

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
