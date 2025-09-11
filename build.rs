use std::process::Command;
use vergen::{BuildBuilder, Emitter};

fn main() {
    // Keep existing embuild step for esp-idf
    embuild::espidf::sysenv::output();

    // Try to get the short git hash (fallback to "unknown" on failure)
    let git_short = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string());

    // Check if there are uncommitted changes (dirty working tree)
    let git_dirty = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .ok()
        .map(|o| {
            if o.status.success() {
                !o.stdout.is_empty()
            } else {
                false
            }
        })
        .unwrap_or(false);

    // Export as compile-time env vars accessible via `env!("GIT_SHORT")` or `option_env!("GIT_SHORT")`
    println!("cargo:rustc-env=GIT_SHORT={git_short}");
    println!("cargo:rustc-env=GIT_DIRTY={git_dirty}");

    // Keep using vergen to emit build timestamp (and any other vergen instructions)
    let instructions = BuildBuilder::default()
        .build_timestamp(true)
        .build()
        .unwrap();

    Emitter::default()
        .add_instructions(&instructions)
        .unwrap()
        .emit()
        .unwrap();
}
