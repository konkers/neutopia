use std::env;
use std::fs;
use std::path::PathBuf;

use failure::{format_err, Error};

#[cfg(target_os = "macos")]
fn os_type() -> &'static str {
    "macos"
}

#[cfg(target_os = "windows")]
fn os_type() -> &'static str {
    "windows"
}

#[cfg(target_os = "linux")]
fn os_type() -> &'static str {
    "linux"
}

fn bass_path() -> PathBuf {
    PathBuf::from("../build/bin")
        .join(os_type())
        .join("bass.exe")
}

fn handle_asm(path: &PathBuf) -> Result<(), Error> {
    let bass = bass_path();
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let tmp_dir = out_path.join("asm_build");

    let ips = path.with_extension("ips");
    let ips = ips.file_name().unwrap();
    let ips = out_path.join("asm").join(PathBuf::from(&ips));

    fs::create_dir_all(ips.parent().unwrap()).map_err(|e| {
        format_err!(
            "unable to create dir {}: {}",
            ips.parent().unwrap().to_string_lossy(),
            e
        )
    })?;
    fs::create_dir_all(&tmp_dir)
        .map_err(|e| format_err!("unable to create dir {}: {}", tmp_dir.to_string_lossy(), e))?;
    asm_build::build(&bass, 0x60000, &tmp_dir, &[path.clone()], &ips)
        .map_err(|e| format_err!("asm_build failed: {}", e))?;
    Ok(())
}

fn main() -> Result<(), Error> {
    let asm_src_dir = PathBuf::from("src").join("asm");
    println!("cargo:rerun-if-changed={}", asm_src_dir.to_string_lossy());
    for entry in fs::read_dir(&asm_src_dir)? {
        let path = entry?.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "asm" {
                    println!("cargo:rerun-if-changed={}", &path.to_string_lossy());
                    handle_asm(&path)?;
                }
            }
        }
    }

    Ok(())
}
