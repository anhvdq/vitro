//! Repository tasks run with `cargo x <command>`.

use std::{path::Path, process::ExitCode};

/// Packages whose `.env` is created from `.env.example`.
const ENV_PACKAGES: [&str; 2] = ["packages/api", "packages/frontend"];

fn main() -> ExitCode {
    let command = std::env::args().nth(1);
    let result = match command.as_deref() {
        Some("setup") => setup(),
        _ => Err("usage: cargo x setup".to_owned()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

/// Creates each package's `.env` from its template. Never overwrites an existing `.env`.
fn setup() -> Result<(), String> {
    for package in ENV_PACKAGES {
        let example = Path::new(package).join(".env.example");
        let env = Path::new(package).join(".env");
        if !example.exists() {
            return Err(format!(
                "missing environment template: {}",
                example.display()
            ));
        }
        if env.exists() {
            println!("Skipped existing {}", env.display());
        } else {
            std::fs::copy(&example, &env)
                .map_err(|e| format!("failed to create {}: {e}", env.display()))?;
            println!("Created {}", env.display());
        }
    }
    Ok(())
}
