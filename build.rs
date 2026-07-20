use std::process::Command;
use std::time::SystemTime;

fn main() {
    let now = SystemTime::now();
    let date = chrono::DateTime::<chrono::Utc>::from(now)
        .format("%Y-%m-%d %H:%M:%S UTC")
        .to_string();

    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    let git_hash = match output {
        Ok(h) if h.status.success() => String::from_utf8(h.stdout).unwrap_or_default(),
        _ => "unknown".to_string(),
    };
    let git_hash = git_hash.trim();

    println!("cargo:rustc-env=BUILD_TIME={}", date);
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-link-search=native=/usr/local/lib/");

    println!("cargo:rerun-if-changed=build.rs");
}
