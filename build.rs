use std::env;
use std::fs;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=SNAKE_GIT_SHA");
    println!("cargo:rerun-if-changed=.git/HEAD");
    if let Ok(head) = fs::read_to_string(".git/HEAD") {
        if let Some(reference) = head.strip_prefix("ref: ").map(str::trim) {
            println!("cargo:rerun-if-changed=.git/{reference}");
        }
    }

    let commit = env::var("SNAKE_GIT_SHA")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(repository_commit)
        .unwrap_or_else(|| "unknown".to_owned());

    println!("cargo:rustc-env=SNAKE_GIT_SHA={commit}");
}

fn repository_commit() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}
