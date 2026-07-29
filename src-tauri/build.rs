//! build.rs — Runs BEFORE the Rust compilation.
//! Copies the frontend dist assets (JS, CSS) into src-tauri/assets/ so that
//! the advice-chat window (which references /assets/*.js) can find the correct
//! hashed files.

use std::fs;
use std::path::Path;

fn main() {
    let workspace_root = env!("CARGO_MANIFEST_DIR"); // = src-tauri/
    let frontend_dist = Path::new(workspace_root).join("../src/frontend/dist");
    let target_assets = Path::new(workspace_root).join("assets");

    if frontend_dist.exists() {
        println!("cargo:warning=Building with frontend dist...");

        let frontend_assets = frontend_dist.join("assets");
        if frontend_assets.exists() {
            for entry in fs::read_dir(&frontend_assets).expect("Failed to read frontend assets dir") {
                let entry = entry.expect("Failed to read entry");
                let file_name = entry.file_name();
                let src_path = entry.path();
                let dst_path = target_assets.join(&file_name);
                if let Err(e) = fs::copy(&src_path, &dst_path) {
                    eprintln!("Warning: failed to copy {}: {}", file_name.to_string_lossy(), e);
                } else {
                    println!("cargo:warning=Copied {} to assets/", file_name.to_string_lossy());
                }
            }
            println!("cargo:warning=Frontend assets copied to assets/");
        }

        let logo_src = frontend_dist.join("logo.png");
        if logo_src.exists() {
            if let Err(e) = fs::copy(&logo_src, target_assets.join("logo.png")) {
                eprintln!("Warning: failed to copy logo.png: {}", e);
            }
        }

        let index_src = frontend_dist.join("index.html");
        if index_src.exists() {
            if let Err(e) = fs::copy(&index_src, target_assets.join("index.html")) {
                eprintln!("Warning: failed to copy index.html: {}", e);
            } else {
                println!("cargo:warning=Copied index.html to assets/");
            }
        }

        // Generate advice-chat.html as a copy of the clean index.html
        if let Err(e) = fs::copy(
            &index_src,
            target_assets.join("advice-chat.html"),
        ) {
            eprintln!("Warning: failed to copy advice-chat.html: {}", e);
        } else {
            println!("cargo:warning=Copied advice-chat.html (clean, no debug checker)");
        }
    } else {
        println!(
            "cargo:warning=Frontend dist not found at {:?}, skipping copy",
            frontend_dist
        );
    }

    println!("cargo:rerun-if-changed=../src/frontend/dist");
    println!("cargo:rerun-if-changed=../src/frontend/dist/assets");
    println!("cargo:rerun-if-changed=../src/frontend/dist/index.html");

    tauri_build::build();
}