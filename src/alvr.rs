//! Locating an ALVR installation.
//!
//! Two layouts exist in the wild:
//!   * launcher-managed:  `<ALVR>\installations\<version>\ALVR Dashboard.exe`
//!   * portable/zip:      `<ALVR>\ALVR Dashboard.exe`
//!
//! Both are supported, and the version is read from the executable's version
//! resource so a folder name is not required.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct AlvrInstall {
    pub root: PathBuf,
    pub version: String,
    pub dashboard: PathBuf,
    pub launcher: Option<PathBuf>,
}

fn is_dashboard(name: &OsStr) -> bool {
    let name = name.to_string_lossy().to_lowercase();
    name == "alvr dashboard.exe" || name == "alvr_dashboard.exe"
}

fn is_launcher(name: &OsStr) -> bool {
    let name = name.to_string_lossy().to_lowercase();
    name == "alvr launcher.exe" || name == "alvr_launcher.exe" || name == "alvr-launcher.exe"
}

/// Find a file by exact name, with an optional "contains" fallback that only
/// accepts the given keyword (so a dashboard is never mistaken for a launcher).
fn find_named(dir: &Path, predicate: fn(&OsStr) -> bool, keyword: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(dir).ok()?;
    let mut fallback: Option<PathBuf> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name();
        if predicate(&name) {
            return Some(path);
        }
        if fallback.is_none() {
            let lower = name.to_string_lossy().to_lowercase();
            if lower.ends_with(".exe") && lower.contains(keyword) {
                fallback = Some(path);
            }
        }
    }
    fallback
}

// ---------------------------------------------------------------- file version

#[cfg(windows)]
fn file_version(path: &Path) -> Option<String> {
    use std::os::windows::ffi::OsStrExt;

    #[repr(C)]
    struct FixedFileInfo {
        signature: u32,
        struct_version: u32,
        file_version_ms: u32,
        file_version_ls: u32,
        product_version_ms: u32,
        product_version_ls: u32,
        file_flags_mask: u32,
        file_flags: u32,
        file_os: u32,
        file_type: u32,
        file_subtype: u32,
        file_date_ms: u32,
        file_date_ls: u32,
    }

    #[link(name = "version")]
    extern "system" {
        fn GetFileVersionInfoSizeW(filename: *const u16, handle: *mut u32) -> u32;
        fn GetFileVersionInfoW(filename: *const u16, handle: u32, len: u32, data: *mut u8) -> i32;
        fn VerQueryValueW(
            block: *const u8,
            sub: *const u16,
            out: *mut *mut u8,
            len: *mut u32,
        ) -> i32;
    }

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let size = unsafe { GetFileVersionInfoSizeW(wide.as_ptr(), std::ptr::null_mut()) };
    if size == 0 {
        return None;
    }
    let mut buffer = vec![0u8; size as usize];
    let ok = unsafe { GetFileVersionInfoW(wide.as_ptr(), 0, size, buffer.as_mut_ptr()) };
    if ok == 0 {
        return None;
    }

    let root: Vec<u16> = "\\".encode_utf16().chain(std::iter::once(0)).collect();
    let mut value: *mut u8 = std::ptr::null_mut();
    let mut length: u32 = 0;
    let ok = unsafe {
        VerQueryValueW(
            buffer.as_ptr(),
            root.as_ptr(),
            &mut value as *mut *mut u8,
            &mut length as *mut u32,
        )
    };
    if ok == 0 || value.is_null() || (length as usize) < std::mem::size_of::<FixedFileInfo>() {
        return None;
    }
    let info = unsafe { &*(value as *const FixedFileInfo) };
    if info.signature != 0xfeef04bd {
        return None;
    }
    let major = info.file_version_ms >> 16;
    let minor = info.file_version_ms & 0xffff;
    let patch = info.file_version_ls >> 16;
    Some(format!("{major}.{minor}.{patch}"))
}

#[cfg(not(windows))]
fn file_version(_path: &Path) -> Option<String> {
    None
}

fn version_from_dir(dir: &Path) -> Option<String> {
    let name = dir.file_name()?.to_str()?.trim();
    let trimmed = name.trim_start_matches(['v', 'V']);
    let mut parts = trimmed.split('.');
    let first = parts.next()?;
    let second = parts.next()?;
    if !first.is_empty()
        && first.chars().all(|c| c.is_ascii_digit())
        && !second.is_empty()
        && second.chars().all(|c| c.is_ascii_digit())
    {
        Some(trimmed.to_owned())
    } else {
        None
    }
}

// ------------------------------------------------------------------- detection

fn version_of(path: &Path, dir: &Path) -> String {
    file_version(path)
        .or_else(|| version_from_dir(dir))
        .unwrap_or_else(|| "unknown".to_owned())
}

/// Locate a dashboard executable inside `root`, directly or one level down.
fn dashboard_in(root: &Path) -> Option<(PathBuf, String)> {
    // portable layout: dashboard right in the ALVR root
    if let Some(path) = find_named(root, is_dashboard, "dashboard") {
        let version = version_of(&path, root);
        return Some((path, version));
    }

    // launcher-managed layout: installations/<version>/...
    let installations = root.join("installations");
    if installations.is_dir() {
        let mut best: Option<(String, PathBuf)> = None;
        for entry in std::fs::read_dir(&installations).ok()?.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            if let Some(path) = find_named(&dir, is_dashboard, "dashboard") {
                let key = dir
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if best.as_ref().map(|(b, _)| key > *b).unwrap_or(true) {
                    best = Some((key, path));
                }
            }
        }
        if let Some((key, path)) = best {
            let version = version_of(&path, Path::new(&key));
            return Some((path, version));
        }
    }

    // one level of sub folders (a portable build unpacked into a sub folder)
    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        if let Some(path) = find_named(&dir, is_dashboard, "dashboard") {
            let version = version_of(&path, &dir);
            return Some((path, version));
        }
    }

    None
}

pub fn detect_in(root: &Path) -> Option<AlvrInstall> {
    let (dashboard, version) = dashboard_in(root)?;
    let launcher = find_named(root, is_launcher, "launcher").or_else(|| {
        dashboard
            .parent()
            .and_then(|dir| find_named(dir, is_launcher, "launcher"))
    });
    Some(AlvrInstall {
        root: root.to_path_buf(),
        version,
        dashboard,
        launcher,
    })
}

/// Point the tool at a specific `ALVR Dashboard.exe` the user picked.
pub fn detect_exe(dashboard: &Path) -> Option<AlvrInstall> {
    if !dashboard.is_file() || !is_dashboard(dashboard.file_name()?) {
        return None;
    }
    let root = dashboard.parent()?.to_path_buf();
    let version = file_version(dashboard)
        .or_else(|| version_from_dir(&root))
        .unwrap_or_else(|| "unknown".to_owned());
    let launcher = find_named(&root, is_launcher, "launcher");
    Some(AlvrInstall {
        root,
        version,
        dashboard: dashboard.to_path_buf(),
        launcher,
    })
}

fn candidates() -> Vec<PathBuf> {
    let mut list = Vec::new();

    // 1. where the tool lives (it may also be dropped inside the ALVR folder)
    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.parent().map(Path::to_path_buf);
        for _ in 0..5 {
            let Some(dir) = current else { break };
            list.push(dir.clone());
            list.push(dir.join("ALVR"));
            current = dir.parent().map(Path::to_path_buf);
        }
    }

    // 2. common install locations
    for var in [
        "ProgramFiles",
        "ProgramFiles(x86)",
        "LOCALAPPDATA",
        "APPDATA",
    ] {
        if let Ok(value) = std::env::var(var) {
            let base = PathBuf::from(value);
            list.push(base.join("ALVR"));
            list.push(base.join("Programs").join("ALVR"));
        }
    }

    // 3. per-drive guesses
    for letter in b'C'..=b'Z' {
        let drive = PathBuf::from(format!("{}:\\", letter as char));
        if !drive.is_dir() {
            continue;
        }
        for sub in [
            "ALVR",
            "Software\\ALVR",
            "Games\\ALVR",
            "Tools\\ALVR",
            "Program Files\\ALVR",
            "Program Files (x86)\\ALVR",
        ] {
            list.push(drive.join(sub));
        }
    }

    list
}

/// A real installation has at least one of these next to the dashboard.
fn looks_complete(install: &AlvrInstall) -> bool {
    let root = &install.root;
    root.join("bin").is_dir()
        || root.join("installations").is_dir()
        || root.join("driver.vrdrivermanifest").is_file()
}

pub fn detect() -> Option<AlvrInstall> {
    let mut seen: Vec<PathBuf> = Vec::new();
    let mut weak: Option<AlvrInstall> = None;
    for candidate in candidates() {
        if seen.contains(&candidate) {
            continue;
        }
        seen.push(candidate.clone());
        if let Some(found) = detect_in(&candidate) {
            if looks_complete(&found) {
                return Some(found);
            }
            if weak.is_none() {
                weak = Some(found);
            }
        }
    }
    weak
}
