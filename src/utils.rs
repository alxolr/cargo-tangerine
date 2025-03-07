use std::path::PathBuf;

use crate::{errors::Result, models::manifest::package::Package};
use regex::Regex;
use tokio::process::Command;

pub async fn run_cargo_info(member: &str, path: &PathBuf) -> Result<Package> {
    let output = Command::new("cargo")
        .current_dir(path)
        .args(["info", member])
        .output()
        .await?;

    if !output.status.success() {
        return Err("Failed to run cargo info".into());
    }

    let output = String::from_utf8(output.stdout)?;
    let output = output.trim();

    let re = Regex::new(r"version:\s*(\S+)").unwrap();
    let version = re
        .captures(output)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))
        .unwrap_or("");

    Ok(Package {
        name: member.to_string(),
        version: version.to_string(),
    })
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
