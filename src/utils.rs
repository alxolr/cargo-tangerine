use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};

use crate::errors::Result;
use crate::models::manifest::package;
use regex::Regex;
use tokio::process::Command;

/// Topologically sorts workspace members so that dependencies are published before dependents.
///
/// Reads each member's Cargo.toml, builds a dependency graph of inter-workspace deps,
/// and returns the members in an order where each package comes after its workspace dependencies.
pub fn topological_sort(members: &[String], workspace_path: &Path) -> Result<Vec<String>> {
    // Parse all member manifests and collect their package names
    let mut manifests: Vec<(String, package::Manifest)> = Vec::new();
    let mut name_to_member: HashMap<String, String> = HashMap::new();

    for member in members {
        let manifest_path = workspace_path.join(member).join("Cargo.toml");
        let manifest = package::Manifest::from_toml(&manifest_path)?;
        name_to_member.insert(manifest.package.name.clone(), member.clone());
        manifests.push((member.clone(), manifest));
    }

    let workspace_packages: HashSet<&str> = name_to_member.keys().map(|s| s.as_str()).collect();

    // Build adjacency list: member -> set of members it depends on
    let mut deps: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut in_degree: HashMap<&str, usize> = HashMap::new();

    for member in members {
        deps.entry(member.as_str()).or_default();
        in_degree.entry(member.as_str()).or_insert(0);
    }

    for (member, manifest) in &manifests {
        let dep_names = manifest.dependency_names();
        for dep_name in dep_names {
            if workspace_packages.contains(dep_name) {
                let dep_member = name_to_member[dep_name].as_str();
                deps.entry(member.as_str()).or_default().push(dep_member);
                *in_degree.entry(member.as_str()).or_insert(0) += 1;
            }
        }
    }

    // Kahn's algorithm for topological sort
    let mut queue: VecDeque<&str> = VecDeque::new();
    for (member, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(member);
        }
    }

    let mut sorted: Vec<String> = Vec::new();
    while let Some(current) = queue.pop_front() {
        sorted.push(current.to_string());

        // For each member that depends on `current`, decrease in-degree
        for (member, member_deps) in &deps {
            if member_deps.contains(&current) {
                let degree = in_degree.get_mut(member).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(member);
                }
            }
        }
    }

    if sorted.len() != members.len() {
        return Err("Circular dependency detected among workspace members".into());
    }

    Ok(sorted)
}

/// Will run `cargo info` for the given member in the given path and return the package information.
/// If the command fails this means the package with version is not published.
///
/// Note: We run `cargo info` from a temp directory with `--registry crates-io` to avoid
/// resolving the package locally from the workspace (which would always report the current version).
pub async fn is_package_published(member: &str, _path: &PathBuf) -> Result<bool> {
    let output = Command::new("cargo")
        .current_dir(std::env::temp_dir())
        .args(["info", member, "--registry", "crates-io"])
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
