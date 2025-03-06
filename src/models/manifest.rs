use std::path::PathBuf;

use serde::Deserialize;

use crate::errors::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub members: Vec<String>,
    pub resolver: Option<String>,
}

/// Workspace manifest file
#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub workspace: Workspace,
}

impl Manifest {
    pub fn from_toml(toml_path: &PathBuf) -> Result<Self> {
        let toml = std::fs::read_to_string(toml_path)?;
        let manifest: Self =
            toml::from_str(&toml).map_err(|_| Error::Str("Not a workspace manifest"))?;

        Ok(manifest)
    }
}
