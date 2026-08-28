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

/// Runs `cargo info {name}` (by name only) and returns the latest version published to the
/// registry, or `None` if the package has never been published.
///
/// Querying by name (rather than `name@version`) makes cargo report the latest version from the
/// registry index. The query runs from a neutral directory (see below) with the workspace's
/// `.cargo/config.toml` supplied via `--config`, targeting the configured default registry.
pub async fn fetch_remote_version(name: &str, path: &Path) -> Result<Option<String>> {
    // Run `cargo info` from a directory OUTSIDE the workspace. Inside the workspace, cargo
    // resolves members as local path dependencies and reports the local version
    // (`version: X (from ./...)`) instead of the registry version. From a neutral directory
    // there is no local package to resolve, so cargo queries the registry.
    let query_dir = std::env::temp_dir();

    let mut args: Vec<String> = vec!["info".into(), name.into()];

    // Since we run outside the workspace, cargo no longer picks up the workspace's
    // `.cargo/config.toml`. Locate it and pass it via `--config` so any private/alternate
    // registry definition and default-registry setting are honored. If a default registry is
    // configured, target it explicitly with `--registry`.
    if let Some(config_path) = find_cargo_config(path) {
        args.push("--config".into());
        args.push(config_path.to_string_lossy().into_owned());

        if let Some(default_registry) = read_default_registry(&config_path) {
            args.push("--registry".into());
            args.push(default_registry);
        }
    }

    let output = Command::new("cargo")
        .current_dir(&query_dir)
        .args(&args)
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr)?;
        let re = Regex::new(r"could not find `(.*)` in registry")?;
        if re.is_match(&stderr) {
            // Package not published at all
            return Ok(None);
        }
        // Some other failure (network, auth, etc.) - surface it
        return Err("Failed to run `cargo info`".into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    Ok(parse_version(&stdout))
}

/// Walks up from `start` looking for a `.cargo/config.toml` (or legacy `.cargo/config`).
fn find_cargo_config(start: &Path) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(current) = dir {
        for name in [".cargo/config.toml", ".cargo/config"] {
            let candidate = current.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        dir = current.parent();
    }
    None
}

/// Reads the `[registry] default = "..."` value from a cargo config file, if present.
fn read_default_registry(config_path: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(config_path).ok()?;
    let value: toml::Value = toml::from_str(&contents).ok()?;
    value
        .get("registry")?
        .get("default")?
        .as_str()
        .map(|s| s.to_string())
}

/// Parses the registry version from `cargo info` output.
///
/// The `version:` line may carry a source marker:
///   - `version: 1.2.3`                                  → registry version (bare)
///   - `version: 1.2.3 (from registry `my-registry`)`    → registry version
///   - `version: 1.2.3 (from ./some/path)`               → LOCAL path resolution
///
/// A local path resolution is not a registry version, so we return `None` for it. For the other
/// cases we return the bare version number (the `(from ...)` suffix is stripped).
fn parse_version(info_output: &str) -> Option<String> {
    for line in info_output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("version:") {
            let value = rest.trim();

            // Split off any `(from ...)` source marker.
            let (version, source) = match value.split_once("(from") {
                Some((v, s)) => (v.trim(), Some(s.trim_end_matches(')').trim())),
                None => (value, None),
            };

            // If the source is a local path (starts with `.` or `/`), this is not a
            // registry version.
            if let Some(source) = source {
                if source.starts_with('.') || source.starts_with('/') {
                    return None;
                }
            }

            if version.is_empty() {
                return None;
            }
            return Some(version.to_string());
        }
    }
    None
}

/// Determines whether a package needs publishing by comparing the local version against
/// the version currently on the remote registry.
///
/// Returns `true` when the package has never been published, or when the local version
/// differs from the published version.
pub async fn needs_publishing(name: &str, local_version: &str, path: &Path) -> Result<bool> {
    match fetch_remote_version(name, path).await? {
        Some(remote_version) => Ok(remote_version != local_version),
        None => Ok(true),
    }
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

#[cfg(test)]
mod tests {
    use super::parse_version;

    #[test]
    fn parses_version_from_cargo_info_output() {
        let output = "\
serde #serde #serialization #no_std
A generic serialization/deserialization framework
version: 1.0.229
license: MIT OR Apache-2.0
";
        assert_eq!(parse_version(output), Some("1.0.229".to_string()));
    }

    #[test]
    fn parses_version_with_leading_whitespace() {
        let output = "  version:   11.4.2  ";
        assert_eq!(parse_version(output), Some("11.4.2".to_string()));
    }

    #[test]
    fn returns_none_when_no_version_line() {
        let output = "some package\ndescription only";
        assert_eq!(parse_version(output), None);
    }

    #[test]
    fn returns_none_for_locally_resolved_version() {
        // When cargo resolves a workspace member locally it appends a `(from ./path)` marker.
        // That is a local path version, not a registry version.
        let output = "\
cargo-tangerine
version: 0.1.5 (from ./)
license: MIT
";
        assert_eq!(parse_version(output), None);
    }

    #[test]
    fn returns_none_for_local_path_subdir() {
        let output = "version: 11.4.1 (from ./conform-cdl)";
        assert_eq!(parse_version(output), None);
    }

    #[test]
    fn parses_registry_sourced_version() {
        // A version sourced from a named registry IS a registry version; strip the marker.
        let output = "\
conform-cdl
version: 11.4.1 (from registry `conform5-rust-common`)
license: unknown
";
        assert_eq!(parse_version(output), Some("11.4.1".to_string()));
    }
}
