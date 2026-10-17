use std::fs;
use std::path::Path;

fn main() {
    // Hash del commit para mostrarlo en Ajustes (misma fuente que
    // grafito-app/build.rs, que no propaga su `rustc-env` a este crate).
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    {
        let git_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.git");
        let head_path = git_dir.join("HEAD");
        if let Ok(head) = fs::read_to_string(&head_path) {
            let head = head.trim();
            if let Some(ref_path) = head.strip_prefix("ref: ") {
                let ref_file = git_dir.join(ref_path);
                println!("cargo:rerun-if-changed=../../{ref_path}");
                let _ = ref_file;
            }
        }
    }
    let hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format!("dev-{}", env!("CARGO_PKG_VERSION")));
    println!("cargo:rustc-env=GRAFITO_GIT_HASH={hash}");
}
