use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use anyhow::{anyhow, Context, Result};

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
enum SubCommand {
    /// Create something new
    New {
        /// Name of the new item
        name: String,
    },

    /// Add to something
    Add {
        /// Item to add
        name: String,
    },

    /// Initialize something
    Init,

    /// Builds a Rust crate to the WASI target
    /// (currently only supports building Rust)
    Build {
        /// Optional path to the project
        path: Option<String>,
    },

    /// Attempt to load into the current daemon;
    /// otherwise start the daemon and load the plugin
    Load {
        /// Path or name of the plugin to load
        plugin: String,
    },
}

#[derive(Debug, StructOpt)]
pub struct Args {
    /// One of: new, add, init, build, load
    #[structopt(subcommand)] 
    subcommand: SubCommand,
}


fn build_crate(crate_path: &Path) -> Result<()> {
    let toml_path = crate_path.join("Cargo.toml");
    if !crate_path.exists() || !toml_path.exists() {
        return Err(anyhow!("Provided path is not a valid Rust crate"));
    }

    // Ensure target is added
    println!("Checking if target `wasm32-wasi` is installed...");
    let _ = Command::new("rustup")
        .args(["target", "add", "wasm32-wasip1"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to run rustup")?;

    // Run cargo build
    println!("Building crate at {} for wasm32-wasi...", crate_path.display());
    let status = Command::new("cargo")
        .current_dir(crate_path)
        .args(["build", "--release", "--target", "wasm32-wasi"])
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        return Err(anyhow!("Build failed"));
    }

    // Find output .wasm file
    let crate_name = get_crate_name(&toml_path)?;
    let wasm_path = crate_path
        .join("target/wasm32-wasi/release")
        .join(format!("{crate_name}.wasm"));

    if !wasm_path.exists() {
        return Err(anyhow!("Compiled wasm not found at {:?}", wasm_path));
    }

    println!("✅ WASM build succeeded: {}", wasm_path.display());

    if !status.success() {
        return Err(anyhow!("Execution in wasmtime failed"));
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::from_args();

    match args.subcommand {
        SubCommand::Build { path: _ } => {

        },
        _ => {},
    }

    Ok(())
}

fn get_crate_name(crate_path: &Path) -> Result<String> {
    let cargo_toml = std::fs::read_to_string(crate_path.join("Cargo.toml"))
        .context("Failed to read Cargo.toml")?;
    for line in cargo_toml.lines() {
        if let Some(rest) = line.strip_prefix("name") {
            if let Some(name) = rest.split('=').nth(1) {
                return Ok(name.trim().trim_matches('"').to_string());
            }
        }
    }
    Err(anyhow!("Could not determine crate name from Cargo.toml"))
}