use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::Result;

#[derive(Debug, Deserialize, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
pub struct Dependency {
    pub workspace: Option<bool>,
    pub version: Option<String>,
    pub path: Option<String>,
}

/// Handles both `dep = "version"` and `dep = { version = "...", ... }` forms.
fn deserialize_dependencies<'de, D>(
    deserializer: D,
) -> std::result::Result<HashMap<String, Dependency>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum DepValue {
        Simple(String),
        Table(Dependency),
    }

    let map: HashMap<String, DepValue> = HashMap::deserialize(deserializer)?;
    Ok(map
        .into_iter()
        .map(|(k, v)| match v {
            DepValue::Simple(version) => (
                k,
                Dependency {
                    version: Some(version),
                    workspace: None,
                    path: None,
                },
            ),
            DepValue::Table(dep) => (k, dep),
        })
        .collect())
}

fn default_empty_deps() -> HashMap<String, Dependency> {
    HashMap::new()
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub package: Package,
    #[serde(default = "default_empty_deps", deserialize_with = "deserialize_dependencies")]
    pub dependencies: HashMap<String, Dependency>,
    #[serde(
        default = "default_empty_deps",
        rename = "dev-dependencies",
        deserialize_with = "deserialize_dependencies"
    )]
    #[allow(dead_code)]
    pub dev_dependencies: HashMap<String, Dependency>,
    #[serde(
        default = "default_empty_deps",
        rename = "build-dependencies",
        deserialize_with = "deserialize_dependencies"
    )]
    pub build_dependencies: HashMap<String, Dependency>,
}

impl Manifest {
    pub fn from_toml(toml_path: &PathBuf) -> Result<Self> {
        let toml = std::fs::read_to_string(toml_path)?;
        let manifest: Self = toml::from_str(&toml)?;

        Ok(manifest)
    }

    pub fn with_version(&self) -> String {
        format!("{}@{}", self.package.name, self.package.version)
    }

    /// Returns the names of all dependencies (regular + build, excluding dev).
    pub fn dependency_names(&self) -> Vec<&str> {
        self.dependencies
            .keys()
            .chain(self.build_dependencies.keys())
            .map(|s| s.as_str())
            .collect()
    }
}
