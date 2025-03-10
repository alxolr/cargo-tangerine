use std::path::PathBuf;

use crate::errors::Result;
use regex::Regex;
use tokio::process::Command;

/// Will run `cargo info` for the given member in the given path and return the package information.
/// If the command fails this means the package with version is not published.
pub async fn is_package_published(member: &str, path: &PathBuf) -> Result<bool> {
    let output = Command::new("cargo")
        .current_dir(path)
        .args(["info", member])
        .output()
        .await?;

    if !output.status.success() {
        // check if the error is due to the package not being published
        let stderr = String::from_utf8(output.stderr)?;
        let re = Regex::new(r"could not find `(.*)` in registry")?;
        if re.is_match(&stderr) {
            return Ok(false);
        }
    }

    Ok(true)
}

pub async fn run_cargo_publish(member: &str, path: &PathBuf) -> Result<()> {
    let output = Command::new("cargo")
        .current_dir(path)
        .args(["publish", "-p", member])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        eprintln!("Failed to publish package: {}", stderr);

        return Err("Failed to publish package".into());
    }

    println!("Published package: {}", member);

    Ok(())
}
