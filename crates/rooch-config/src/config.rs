// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub enum MapConfigValueSource {
    ConfigKey,   // Value came from the presence of a key in the configuration
    Environment, // Value came from the environment
    Default,     // Value came from a defined default value
}

pub trait Config
where
    Self: DeserializeOwned + Serialize,
{
    fn persisted(self, path: &Path) -> PersistedConfig<Self> {
        PersistedConfig {
            inner: self,
            path: path.to_path_buf(),
        }
    }

    fn load<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let path = path.as_ref();
        let reader = fs::File::open(path)
            .with_context(|| format!("Unable to load config from {}", path.display()))?;
        Ok(serde_yaml::from_reader(reader)?)
    }

    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), anyhow::Error> {
        let path = path.as_ref();
        let config = serde_yaml::to_string(&self)?;
        fs::write(path, config)
            .with_context(|| format!("Unable to save config to {}", path.display()))?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct PersistedConfig<C> {
    inner: C,
    path: PathBuf,
}

impl<C> PersistedConfig<C>
where
    C: Config,
{
    pub fn read(path: &Path) -> Result<C, anyhow::Error> {
        Config::load(path)
    }

    pub fn save(&self) -> Result<(), anyhow::Error> {
        self.inner.save(&self.path)
    }

    pub fn into_inner(self) -> C {
        self.inner
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<C> std::ops::Deref for PersistedConfig<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<C> std::ops::DerefMut for PersistedConfig<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn parse_hashmap(
    s: &str,
) -> Result<HashMap<String, String>, Box<dyn Error + Send + Sync + 'static>> {
    s.split(',')
        .filter(|kv| !kv.is_empty())
        .map(|kv| {
            let mut parts = kv.splitn(2, '=');
            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) if !key.trim().is_empty() => {
                    Ok((key.to_string(), value.to_string()))
                }
                (Some(""), Some(_)) => Err("key is missing before '='".into()),
                _ => {
                    Err("each key=value pair must be separated by a comma and contain a key".into())
                }
            }
        })
        .collect()
}

// value order:
// 1. key value
// 2. env value
// 3. default value
pub fn retrieve_map_config_value(
    config: &mut HashMap<String, String>,
    key: &str,
    env_var: Option<&str>,
    default_var: &str,
) -> MapConfigValueSource {
    if config.contains_key(key) {
        return MapConfigValueSource::ConfigKey;
    }

    if let Some(env_var) = env_var {
        if let Ok(env_var_value) = std::env::var(env_var) {
            // env_var exists
            config.insert(key.to_string(), env_var_value.clone());
            return MapConfigValueSource::Environment;
        }
    }

    // Use the default
    config.insert(key.to_string(), default_var.to_string());
    MapConfigValueSource::Default
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hashmap_ok() {
        let input = "key1=VALUE1,key2=value2";
        let output = parse_hashmap(input).unwrap();

        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), "VALUE1".to_string());
        expected.insert("key2".to_string(), "value2".to_string());

        assert_eq!(output, expected);
    }

    #[test]
    fn test_parse_hashmap_empty_value() {
        let input = "key1=,key2=value2";
        let output = parse_hashmap(input).unwrap();

        let mut expected = HashMap::new();
        expected.insert("key1".to_string(), "".to_string());
        expected.insert("key2".to_string(), "value2".to_string());

        assert_eq!(output, expected);
    }

    #[test]
    fn test_parse_hashmap_empty_string() {
        let input = "";
        let output = parse_hashmap(input).unwrap();

        let expected = HashMap::new();

        assert_eq!(output, expected);
    }

    #[test]
    fn test_parse_hashmap_missing_value() {
        let input = "key1,key2=value2";
        let output = parse_hashmap(input);

        assert!(output.is_err());
    }

    #[test]
    fn test_parse_hashmap_missing_key() {
        let input = "=value1,key2=value2";
        let output = parse_hashmap(input);

        assert!(output.is_err());
    }

    #[test]
    fn test_parse_hashmap_no_equals_sign() {
        let input = "key1value1,key2=value2";
        let output = parse_hashmap(input);

        assert!(output.is_err());
    }
}
