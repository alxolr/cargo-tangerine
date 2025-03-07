use std::{env, path::PathBuf};

use clap::Parser;

use crate::{
    errors::Result,
    models::manifest::{package, workspace},
    utils::{run_cargo_info, run_cargo_publish},
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

            let published_version = run_cargo_info(member, &self.path).await?;

            if published_version.version == package_manifest.package.version {
                println!("{}@{} ✔", member, package_manifest.package.version);
                continue;
            } else {
                println!(
                    "{}@{} -> {}@{}",
                    published_version.name,
                    published_version.version,
                    member,
                    package_manifest.package.version
                );

                // Publish package
                run_cargo_publish(member, &self.path).await?;
            }
        }

        Ok(())
    }
}
