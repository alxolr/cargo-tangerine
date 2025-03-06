use std::path::PathBuf;

use clap::Parser;

use crate::{errors::Result, models::manifest::Manifest};

#[derive(Debug, Parser)]
pub struct List {
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl List {
    pub async fn run(&self) -> Result<()> {
        let manifest_path = self.path.join("Cargo.toml");

        let manifest = Manifest::from_toml(&manifest_path)?;

        for member in manifest.workspace.members {
            println!("{}", member);
        }

        Ok(())
    }
}
