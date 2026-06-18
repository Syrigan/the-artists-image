use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tauri::{AppHandle, Manager};

pub fn resolve_scraper_path(app: &AppHandle) -> Result<PathBuf, String> {
    let candidates = scraper_candidates(app);
    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(
        "Could not find lyrics scraper (sidecar/scraper.py). \
         Rebuild the app or install Python dependencies: \
         pip install -r src-tauri/sidecar/requirements.txt"
            .to_string(),
    )
}

fn scraper_candidates(app: &AppHandle) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            paths.push(parent.join("sidecar").join("scraper.py"));
        }
    }

    if let Ok(resource) = app
        .path()
        .resolve("sidecar/scraper.py", tauri::path::BaseDirectory::Resource)
    {
        paths.push(resource);
    }

    paths.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("sidecar")
            .join("scraper.py"),
    );

    paths
}

pub fn run_scraper(scraper_path: &Path, artist: &str, album: &str) -> Result<Output, String> {
    let attempts: &[(&str, &[&str])] = if cfg!(windows) {
        &[("py", &["-3"] as &[&str]), ("python", &[]), ("python3", &[])]
    } else {
        &[("python3", &[]), ("python", &[])]
    };

    let mut last_error = String::from("Python is not installed or not on PATH.");

    for (program, prefix_args) in attempts {
        let mut command = Command::new(program);
        for arg in *prefix_args {
            command.arg(arg);
        }
        command.arg(scraper_path).arg(artist).arg(album);

        match command.output() {
            Ok(output) => return Ok(output),
            Err(err) => {
                last_error = format!("Failed to run {program}: {err}");
            }
        }
    }

    Err(format!(
        "{last_error} Install Python 3 and run: pip install -r sidecar/requirements.txt"
    ))
}
