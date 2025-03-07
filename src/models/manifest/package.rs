use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::Result;

#[derive(Debug, Deserialize, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub package: Package,
}

impl Manifest {
    pub fn from_toml(toml: &PathBuf) -> Result<Self> {
        let toml = std::fs::read_to_string(toml)?;
        let manifest: Self = toml::from_str(&toml)?;

        Ok(manifest)
    }
}
