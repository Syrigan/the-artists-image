fn main() {
    tauri_build::build();
    copy_sidecar_to_target();
}

fn copy_sidecar_to_target() {
    let manifest_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => dir,
        Err(_) => return,
    };
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".into());

    let src = std::path::Path::new(&manifest_dir).join("sidecar");
    let dst = std::path::Path::new(&manifest_dir)
        .join("target")
        .join(profile)
        .join("sidecar");

    if !src.join("scraper.py").exists() {
        return;
    }

    let _ = std::fs::create_dir_all(&dst);
    for name in ["scraper.py", "requirements.txt"] {
        let from = src.join(name);
        if from.exists() {
            let _ = std::fs::copy(from, dst.join(name));
        }
    }
}
