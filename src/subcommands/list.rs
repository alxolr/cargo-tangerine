use std::path::PathBuf;

use clap::Parser;

use crate::{
    errors::Result,
    models::manifest::{package, workspace::Manifest},
};

#[derive(Debug, Parser)]
pub struct List {
    #[clap(short, long, default_value = ".")]
    path: PathBuf,
}

impl List {
    pub async fn run(&self) -> Result<()> {
        let manifest_path = self.path.join("Cargo.toml");
        let manifest = Manifest::from_toml(&manifest_path)?;

        for member in manifest.workspace.members.iter() {
            let package_path = self.path.join(member).join("Cargo.toml");
            let package_manifest = package::Manifest::from_toml(&package_path)?;

            println!("{}@{}", member, package_manifest.package.version);
        }

        Ok(())
    }
}
