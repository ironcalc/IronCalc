// At build time, we grab the release version from the git tag.
// This is used by INFO("release")
use std::process::Command;

fn run_git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn main() {
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");

    // Prefer exact tag
    let version = run_git(&["describe", "--tags", "--exact-match"])
        .or_else(|| run_git(&["describe", "--tags", "--dirty", "--always"]))
        .unwrap_or_else(|| "unknown".into());

    println!("cargo:rustc-env=GIT_VERSION={}", version);
}
