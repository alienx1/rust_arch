//! Build script for master-app (Leptos WebAssembly SPA).
//!
//! Injects environment variables from the root `.env` file at compile-time so that
//! `option_env!("VAR")` / `env!("VAR")` reflects values configured in `.env`.
//! Also triggers an automatic Cargo / Trunk re-compilation whenever `.env` changes.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Search for .env starting from workspace root (4 levels up from this crate)
    let candidates = ["../../../../.env", ".env"];

    for path_str in candidates {
        let env_path = Path::new(path_str);
        if env_path.exists() {
            println!("cargo:rerun-if-changed={}", env_path.display());
            if let Ok(iter) = dotenvy::from_path_iter(env_path) {
                for item in iter.flatten() {
                    let (key, val) = item;
                    // Expose .env variable to rustc during compilation
                    println!("cargo:rustc-env={key}={val}");
                }
            }
            break;
        }
    }
}
