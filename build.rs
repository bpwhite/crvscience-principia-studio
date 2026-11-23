// Copyright (C) 2026 CRV Science Network (https://crvscience.com)
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://gnu.org/licenses/>.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=assets/principia_studio.ico");
    println!("cargo:rerun-if-changed=build.rs");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => return,
    };
    let out_dir = match env::var("OUT_DIR") {
        Ok(val) => PathBuf::from(val),
        Err(_) => return,
    };

    let ico_path = manifest_dir.join("assets").join("principia_studio.ico");
    if !ico_path.exists() {
        println!(
            "cargo:warning=Icon file missing at {}; skipping Windows resource embedding.",
            ico_path.display()
        );
        return;
    }

    let ico_rc_path = ico_path.to_string_lossy().replace('\\', "/");
    let rc_content = format!(
        r#"1 ICON "{ico_rc_path}"

1 VERSIONINFO
FILEVERSION 1,1,0,0
PRODUCTVERSION 1,1,0,0
FILEOS 0x40004
FILETYPE 0x1
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904B0"
        BEGIN
            VALUE "CompanyName", "CRV Science Network"
            VALUE "FileDescription", "Principia Studio - Multithreaded Safe-Rust Computational Mathematics Explorer"
            VALUE "FileVersion", "1.1.0.0"
            VALUE "InternalName", "principia-studio"
            VALUE "LegalCopyright", "Copyright CRV Science Network (GPL-3.0-or-later)"
            VALUE "OriginalFilename", "principia-studio.exe"
            VALUE "ProductName", "Principia Studio"
            VALUE "ProductVersion", "1.1.0"
        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x0409, 1200
    END
END
"#
    );

    let rc_path = out_dir.join("app.rc");
    if let Err(err) = fs::write(&rc_path, rc_content) {
        println!("cargo:warning=Failed to write {}: {err}", rc_path.display());
        return;
    }

    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_env == "gnu" {
        let obj_path = out_dir.join("app_res.o");
        for tool in ["windres", "x86_64-w64-mingw32-windres"] {
            if let Ok(status) = Command::new(tool)
                .arg(&rc_path)
                .arg("-O")
                .arg("coff")
                .arg("-o")
                .arg(&obj_path)
                .status()
            {
                if status.success() {
                    println!("cargo:rustc-link-arg-bins={}", obj_path.display());
                    return;
                }
            }
        }
        println!(
            "cargo:warning=windres not available; skipping Windows PE icon/versioninfo embedding."
        );
    } else if target_env == "msvc" {
        let res_path = out_dir.join("app_res.res");
        if let Some(rc_exe) = find_msvc_rc() {
            if let Ok(status) = Command::new(&rc_exe)
                .arg("/nologo")
                .arg("/fo")
                .arg(&res_path)
                .arg(&rc_path)
                .status()
            {
                if status.success() {
                    println!("cargo:rustc-link-arg-bins={}", res_path.display());
                    return;
                }
            }
        }
        println!(
            "cargo:warning=rc.exe not available; skipping Windows PE icon/versioninfo embedding."
        );
    }
}

fn find_msvc_rc() -> Option<PathBuf> {
    if Command::new("rc.exe").arg("/?").output().is_ok() {
        return Some(PathBuf::from("rc.exe"));
    }
    let kits_bin = Path::new(r"C:\Program Files (x86)\Windows Kits\10\bin");
    if let Ok(entries) = fs::read_dir(kits_bin) {
        let mut versions: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|v| v.path()))
            .filter(|p| p.is_dir())
            .collect();
        versions.sort();
        for dir in versions.into_iter().rev() {
            let candidate = dir.join("x64").join("rc.exe");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}
