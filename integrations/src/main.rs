use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{anyhow, Context, Result};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path-to-crate>", args[0]);
        std::process::exit(1);
    }

    let crate_path = Path::new(&args[1]);
    if !crate_path.exists() || !crate_path.join("Cargo.toml").exists() {
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
    let crate_name = get_crate_name(crate_path)?;
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