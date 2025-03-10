use std::{env, path::PathBuf};

use clap::Parser;

use crate::{
    errors::Result,
    models::manifest::{package, workspace},
    utils::{is_package_published, run_cargo_publish},
};

#[derive(Debug, Parser)]
pub struct Publish {
    #[clap(default_value_os_t = env::current_dir().unwrap())]
    path: PathBuf,
}

impl Publish {
    pub async fn run(&self) -> Result<()> {
        println!("Checking packages...\n");

        let manifest_path = self.path.join("Cargo.toml");
        let manifest = workspace::Manifest::from_toml(&manifest_path)?;

        for member in manifest.workspace.members.iter() {
            let package_manifest_path = self.path.join(member).join("Cargo.toml");
            let package_manifest = package::Manifest::from_toml(&package_manifest_path)?;

            if is_package_published(&package_manifest.with_version(), &self.path).await? {
                println!("{} ✔", package_manifest.with_version());
                continue;
            } else {
                println!("{} - to publish", package_manifest.with_version());
                // Publish package
                run_cargo_publish(member, &self.path).await?;
            }
        }

        Ok(())
    }
}
