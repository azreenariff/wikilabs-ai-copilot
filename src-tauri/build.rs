//! build.rs — Runs BEFORE the Rust compilation.
//! Copies the frontend dist assets (JS, CSS) into src-tauri/assets/ so that
//! the advice-chat window (which references /assets/*.js) can find the correct
//! hashed files. Also generates a clean advice-chat.html from the frontend's
//! index.html, removing the debug "HTML LOADED OK" overlay.

use std::fs;
use std::path::Path;

/// Generate a clean index.html for the advice-chat window.
/// Removes the debug "HTML LOADED OK" overlay and its MutationObserver script.
fn generate_advice_chat_html(frontend_dist: &Path, target_path: &Path) -> std::io::Result<()> {
    let index_path = frontend_dist.join("index.html");
    if !index_path.exists() {
        return Ok(());
    }
    let content = fs::read_to_string(&index_path)?;

    // Remove the debug HTML checker div and its MutationObserver
    let cleaned = content
        .replace(
            "    <div id=\"html-checker\" style=\"position:fixed;top:0;left:0;width:100vw;height:100vh;background:#ff0000;display:flex;align-items:center;justify-content:center;font-size:36px;color:white;z-index:999999;\">\n      HTML LOADED OK\n    </div>\n",
            "",
        )
        .replace(
            "    <script>\n      // If React mounts, hide the checker\n      const checker = document.getElementById('html-checker');\n      const observer = new MutationObserver(function() {\n        if (document.getElementById('root').children.length > 0) {\n          checker.style.display = 'none';\n        }\n      });\n      observer.observe(document.getElementById('root'), { childList: true, subtree: true });\n    </script>\n",
            "",
        );

    fs::write(target_path, cleaned)?;
    println!("cargo:warning=Generated clean advice-chat.html");
    Ok(())
}

fn ensure_copy(src: &Path, dst: &Path) -> std::io::Result<()> {
    // Only copy if source is newer or destination doesn't exist
    let dst_exists = dst.exists();
    let src_mtime = fs::metadata(src)?.modified()?;
    if dst_exists {
        let dst_mtime = fs::metadata(dst)?.modified()?;
        if src_mtime <= dst_mtime {
            return Ok(());
        }
    }
    fs::copy(src, dst)?;
    println!("cargo:warning=Copied {} to assets/", src.file_name().unwrap().to_string_lossy());
    Ok(())
}

fn main() {
    // Resolve paths relative to src-tauri/ (where cargo runs build.rs from)
    let workspace_root = env!("CARGO_MANIFEST_DIR"); // = src-tauri/
    let frontend_dist = Path::new(workspace_root).join("../src/frontend/dist");
    let target_assets = Path::new(workspace_root).join("assets");

    if frontend_dist.exists() {
        println!("cargo:warning=Building with frontend dist...");

        // Copy JS/CSS from frontend dist/assets/ to assets/ (flat)
        // These are served by ServeDir::new("../assets") at /assets/* path
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

        // Copy logo.png
        let logo_src = frontend_dist.join("logo.png");
        if logo_src.exists() {
            if let Err(e) = fs::copy(&logo_src, target_assets.join("logo.png")) {
                eprintln!("Warning: failed to copy logo.png: {}", e);
            }
        }

        // Copy index.html (for any direct references)
        let index_src = frontend_dist.join("index.html");
        if index_src.exists() {
            if let Err(e) = fs::copy(&index_src, target_assets.join("index.html")) {
                eprintln!("Warning: failed to copy index.html: {}", e);
            } else {
                println!("cargo:warning=Copied index.html to assets/");
            }
        }

        // Generate a clean advice-chat.html (remove debug overlay)
        if let Err(e) = generate_advice_chat_html(
            &frontend_dist,
            target_assets.join("advice-chat.html").as_path(),
        ) {
            eprintln!("Warning: failed to generate advice-chat.html: {}", e);
        }
    } else {
        println!(
            "cargo:warning=Frontend dist not found at {:?}, skipping copy",
            frontend_dist
        );
    }

    // Tell cargo to re-run this script when these change
    println!("cargo:rerun-if-changed=../src/frontend/dist");
    println!("cargo:rerun-if-changed=../src/frontend/dist/assets");
    println!("cargo:rerun-if-changed=../src/frontend/dist/index.html");

    tauri_build::build();
}