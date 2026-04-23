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

    let commit = run_git(&["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let branch =
        run_git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into());

    let dirty = run_git(&["status", "--porcelain"])
        .map(|s| if s.is_empty() { "false" } else { "true" })
        .unwrap_or("unknown");

    let build_time = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        secs.to_string()
    };
}
