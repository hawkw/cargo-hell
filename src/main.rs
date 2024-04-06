use anyhow::Context;
use clap::Parser;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

/// Punishes you for your Hubris.
#[derive(Debug, Clone, clap::Parser)]
struct CliArgs {
    /// Path to the Cargo.toml file.
    path: PathBuf,

    #[clap(long, env = "CARGO_TARGET_DIR", default_value = "target")]
    target_dir: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let args = CliArgs::parse();
    let manifest_path = args.path.join("Cargo.toml");

    eprintln!("punishing {} you for its Hubris", manifest_path.display());

    let cargo_meta = cargo_metadata::MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .context("failed to get cargo metadata")?;
    let package = cargo_meta
        .root_package()
        .context("root package not found")?;
    let hubris_meta = &package.metadata.get("hubris").with_context(|| {
        format!(
            "{} does not contain a [package.metadata.hubris] table",
            manifest_path.display()
        )
    })?;

    let dist_dir = args.target_dir.join("dist").join(&package.name);
    fs::create_dir_all(&dist_dir)
        .with_context(|| format!("failed to create {}", dist_dir.display()))?;
    let app_toml_path = dist_dir.join("app.toml");
    let mut app_toml = fs::File::create(&app_toml_path)?;
    let hubris_cfg_toml =
        toml::to_string_pretty(hubris_meta).context("hubris meta isn't tomlable")?;
    eprintln!("{hubris_cfg_toml}");
    app_toml.write_all(hubris_cfg_toml.as_bytes())?;

    let triple = hubris_meta
        .get("target")
        .ok_or_else(|| anyhow::anyhow!("missing 'target = ...' in [package.metadata.hubris]"))?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("'target' must be a string"))?;

    // Here is where we would actually "draw the rest of `cargo xtask dist`",
    // basically...
    std::process::Command::new("cargo")
        .args(&["build", "--release"])
        .arg("--package")
        .arg(&package.name)
        .arg("--target")
        .arg(triple)
        .env("CARGO_TARGET_DIR", &args.target_dir)
        .env("HUBRIS_TASK_CONFIG", &app_toml_path)
        // TODO(eliza): set `HUBRIS_TASKS` env var...
        .status()
        .context("cargo build didnt do that")?;

    Ok(())
}
