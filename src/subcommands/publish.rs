use std::{env, path::PathBuf};

use clap::Parser;

use crate::{
    errors::Result,
    models::manifest::{package, workspace},
    utils::{needs_publishing, run_cargo_publish, topological_sort},
};

#[derive(Debug, Parser)]
pub struct Publish {
    #[clap(default_value_os_t = env::current_dir().unwrap())]
    path: PathBuf,

    /// Perform a dry run without actually publishing packages
    #[clap(long, short = 'n')]
    dry_run: bool,
}

impl Publish {
    pub async fn run(&self) -> Result<()> {
        if self.dry_run {
            println!("Dry run mode — no packages will be published.\n");
        }

        println!("Checking packages...\n");

        let manifest_path = self.path.join("Cargo.toml");
        let manifest = workspace::Manifest::from_toml(&manifest_path)?;

        // Sort members topologically so dependencies are published first
        let sorted_members = topological_sort(&manifest.workspace.members, &self.path)?;

        for member in sorted_members.iter() {
            let package_manifest_path = self.path.join(member).join("Cargo.toml");
            let package_manifest = package::Manifest::from_toml(&package_manifest_path)?;

            let name = &package_manifest.package.name;
            let local_version = &package_manifest.package.version;

            if !needs_publishing(name, local_version, &self.path).await? {
                println!("{} ✔ (up to date)", package_manifest.with_version());
                continue;
            }

            if self.dry_run {
                println!("{} - would publish", package_manifest.with_version());
            } else {
                println!("{} - to publish", package_manifest.with_version());
                run_cargo_publish(member, &self.path).await?;
            }
        }

        Ok(())
    }
}
