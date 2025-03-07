# cargo-tangerine
<img src="/assets/images/cargo-tangerine.png" alt="cargo-tangerine logo" width="200"/>

A cargo subcomand to handle workspaces and publish only the changed crates in the right order.

## Installation

```bash
cargo install cargo-tangerine
```

### Usage

```bash
cargo tangerine publish
```

### Algorithm

*v0.1.0*

- Check the list of members in the workspace manifest `Cargo.toml`, `members = ["crate1", "crate2"]`
- For each crate in the list, check if the version was changed using `cargo info {crate}`
- Publish the crate if the version was changed `cargo publish -p {crate}`

