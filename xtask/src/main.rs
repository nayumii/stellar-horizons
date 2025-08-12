use anyhow::{Context, Result};
use clap::Parser;
use dirs_next::home_dir;
use serde::Deserialize;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::FileOptions;

#[derive(Parser, Debug)]
#[command(about = "Build Factorio-ready ZIPs for all submods")]
struct Args {
    /// Copy built zips into your Factorio mods folder
    #[arg(short, long)]
    install: bool,

    /// Output directory (default: build)
    #[arg(long, default_value = "build")]
    out_dir: String,
}

#[derive(Deserialize)]
struct Info {
    name: String,
    version: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let repo_root = std::env::current_dir()?;
    let submods = find_all_mods(&repo_root)?;

    let out_dir = PathBuf::from(&args.out_dir);
    fs::create_dir_all(&out_dir)?;

    for mod_root in submods {
        println!("📦 Building {}", mod_root.display());
        let info: Info = {
            let s = fs::read_to_string(mod_root.join("info.json"))?;
            serde_json::from_str(&s).context("Failed to parse info.json")?
        };
        let top = format!("{}_{}", info.name, info.version);
        let zip_path = out_dir.join(format!("{top}.zip"));

        if zip_path.exists() {
            fs::remove_file(&zip_path)?;
        }

        build_zip(&mod_root, &zip_path, &top)?;
        println!("  → Built: {}", zip_path.display());

        if args.install {
            let dest = factorio_mods_dir()?.join(zip_path.file_name().unwrap());
            fs::create_dir_all(dest.parent().unwrap())?;
            fs::copy(&zip_path, &dest)?;
            println!("  → Installed to: {}", dest.display());
        }
    }

    Ok(())
}

/// Finds all directories (including root) containing info.json
fn find_all_mods(root: &Path) -> Result<Vec<PathBuf>> {
    let mut mods = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() && path.join("info.json").exists() {
            mods.push(path);
        }
    }
    Ok(mods)
}

/// Builds a ZIP with correct top folder & forward slashes
fn build_zip(mod_root: &Path, out_zip: &Path, top: &str) -> Result<()> {
    let file = File::create(out_zip)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let excludes = [
        "build", ".git", ".github", ".idea", ".vscode", "node_modules"
    ];

    for entry in WalkDir::new(mod_root).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path == mod_root {
            continue;
        }

        // Skip excluded top-level dirs
        if let Some(name) = path.strip_prefix(mod_root)
            .ok()
            .and_then(|p| p.components().next())
            .and_then(|c| c.as_os_str().to_str())
        {
            if excludes.contains(&name) {
                continue;
            }
        }

        let rel = path.strip_prefix(mod_root).unwrap();
        let mut inzip = PathBuf::from(top);
        inzip.push(rel);
        let inzip = path_to_forward_slashes(&inzip);

        if path.is_dir() {
            zip.add_directory(inzip, opts)?;
        } else {
            zip.start_file(inzip, opts)?;
            let mut f = File::open(path)?;
            io::copy(&mut f, &mut zip)?;
        }
    }

    zip.finish()?;
    Ok(())
}

fn path_to_forward_slashes(p: &Path) -> String {
    p.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn factorio_mods_dir() -> Result<PathBuf> {
    let home = home_dir().context("No home directory found")?;
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| home.join("AppData\\Roaming").to_string_lossy().to_string());
        Ok(PathBuf::from(appdata).join("Factorio\\mods"))
    }
    #[cfg(target_os = "macos")]
    {
        Ok(home.join("Library/Application Support/factorio/mods"))
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        Ok(home.join(".factorio/mods"))
    }
}