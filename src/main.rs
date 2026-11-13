#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! ALVR-L10N — a standalone localization tool for ALVR.
//!
//! * `builds/<version>/` — fully localized Dashboard/Launcher built from that ALVR
//!   release (100% coverage);
//! * patch mode — rewrites strings and swaps the embedded font in place, works on
//!   any ALVR version without rebuilding (partial coverage).
//!
//! Only the language packs the user checks are copied into ALVR, so ALVR's own
//! language selector lists exactly those.
//!
//! Command line: `--apply`, `--restore`, `--backup-settings`, `--lang <id>`,
//! `--no-build`, `--path <alvr dir>`. Results go to `alvr-l10n.log`.

mod alvr;
mod lang;
mod pack;
mod patch;
mod sfnt;
mod theme;

use alvr::AlvrInstall;
use eframe::egui;
use lang::{UiLang, UI_LANGS};
use pack::LanguagePack;
use std::path::{Path, PathBuf};

const UI_FONT_SC: &[u8] = include_bytes!("../assets/ui-sc.ttf");
const UI_FONT_KR: &[u8] = include_bytes!("../assets/ui-kr.ttf");

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| PathBuf::from("."))
}

struct Paths {
    packs: PathBuf,
    builds: PathBuf,
    settings: PathBuf,
}

fn load_deny(paths: &Paths) -> std::collections::HashSet<String> {
    std::fs::read_to_string(paths.packs.join("deny.txt"))
        .map(|text| {
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

// --------------------------------------------------------------------- prefs

pub const WINDOW_TITLE: &str = "ALVR Localization Tool";
pub const PROJECT_URL: &str = "https://github.com/AnShanZi-2018/ALVR-L10N";
pub const PACKS_INDEX: &str =
    "https://api.github.com/repos/AnShanZi-2018/ALVR-L10N/contents/languages";

#[derive(Clone)]
struct UiPrefs {
    lang: UiLang,
    dark: bool,
    alvr_exe: Option<String>,
}

fn read_ini(path: &Path) -> Vec<(String, String)> {
    std::fs::read_to_string(path)
        .map(|text| {
            text.lines()
                .filter_map(|line| {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with(';') || line.starts_with('[') {
                        return None;
                    }
                    let (key, value) = line.split_once('=')?;
                    Some((key.trim().to_ascii_lowercase(), value.trim().to_owned()))
                })
                .collect()
        })
        .unwrap_or_default()
}

fn load_prefs(path: &Path) -> UiPrefs {
    let entries = read_ini(path);
    let lookup = |key: &str| {
        entries
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    };
    UiPrefs {
        lang: lookup("language")
            .map(|value| UiLang::from_id(&value))
            .unwrap_or(UiLang::En),
        dark: lookup("theme")
            .map(|value| !value.eq_ignore_ascii_case("light"))
            .unwrap_or(true),
        alvr_exe: lookup("alvr_exe").filter(|value| !value.is_empty()),
    }
}

fn save_prefs(path: &Path, prefs: UiPrefs) {
    let text = format!(
        "; ALVR-L10N 工具设置 / tool settings\r\n\
         ; language: en / zh-CN / zh-TW / ja-JP / ko-KR\r\n\
         ; theme: dark / light\r\n\
         \r\n[ui]\r\nlanguage = {}\r\ntheme = {}\r\nalvr_exe = {}\r\n",
        prefs.lang.id(),
        if prefs.dark { "dark" } else { "light" },
        prefs.alvr_exe.clone().unwrap_or_default()
    );
    let _ = std::fs::write(path, text);
}

// ------------------------------------------------------------------- settings

fn session_file(install: &AlvrInstall) -> PathBuf {
    install
        .dashboard
        .parent()
        .map(|dir| dir.join("session.json"))
        .unwrap_or_else(|| install.root.join("session.json"))
}

pub const SETTINGS_BACKUP_DIR: &str = "配置备份";

fn backup_folder(install: &AlvrInstall) -> PathBuf {
    install.root.join(SETTINGS_BACKUP_DIR)
}

#[repr(C)]
struct SystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[cfg(windows)]
fn timestamp() -> String {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetLocalTime(out: *mut SystemTime);
    }
    let mut now = SystemTime {
        year: 1970,
        month: 1,
        day_of_week: 0,
        day: 1,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe { GetLocalTime(&mut now) };
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        now.year, now.month, now.day, now.hour, now.minute, now.second
    )
}

#[cfg(not(windows))]
fn timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix-{secs}")
}

/// Copy `session.json` into `<ALVR>/配置备份/` and return the full path.
fn backup_settings(install: &AlvrInstall) -> Result<PathBuf, String> {
    let source = session_file(install);
    let bytes = std::fs::read(&source).map_err(|e| format!("read session.json: {e}"))?;
    let folder = backup_folder(install);
    std::fs::create_dir_all(&folder).map_err(|e| format!("create backup folder: {e}"))?;
    let target = folder.join(format!("session-{}.json", timestamp()));
    std::fs::write(&target, bytes).map_err(|e| format!("write backup: {e}"))?;
    Ok(target)
}

struct SessionSnapshot {
    path: PathBuf,
    bytes: Option<Vec<u8>>,
}

fn snapshot_session(install: &AlvrInstall) -> SessionSnapshot {
    let path = session_file(install);
    let bytes = std::fs::read(&path).ok();
    let _ = backup_settings(install);
    SessionSnapshot { path, bytes }
}

fn ensure_session_intact(snapshot: &SessionSnapshot) -> Option<String> {
    let current = std::fs::read(&snapshot.path).ok();
    if current == snapshot.bytes {
        return None;
    }
    match &snapshot.bytes {
        Some(bytes) if std::fs::write(&snapshot.path, bytes).is_ok() => Some(
            "ALVR rewrote session.json (usually a settings reset) — restored your file.".to_owned(),
        ),
        _ => Some("session.json changed; please check your ALVR settings.".to_owned()),
    }
}

// ------------------------------------------------------------ pack management

/// Copy the checked packs into ALVR and remove the ones installed earlier but no
/// longer checked, so ALVR's language menu matches the selection.
fn sync_chosen_packs(
    install: &AlvrInstall,
    packs: &[LanguagePack],
    chosen: &[usize],
) -> Result<Vec<String>, String> {
    let target = install.root.join("languages");
    std::fs::create_dir_all(&target).map_err(|e| format!("create languages folder: {e}"))?;

    let manifest_dir = install.root.join(patch::BACKUP_DIR);
    let manifest = manifest_dir.join("packs.txt");
    let previous: Vec<String> = std::fs::read_to_string(&manifest)
        .map(|text| {
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    let mut installed = Vec::new();
    for index in chosen {
        let pack = &packs[*index];
        std::fs::copy(&pack.ini_path, target.join(format!("{}.ini", pack.id)))
            .map_err(|e| format!("copy {}: {e}", pack.id))?;
        installed.push(format!("{}.ini", pack.id));
    }

    // Remove packs of ours that are no longer checked. This also cleans up
    // installations made by older versions of this tool (no manifest yet).
    let ours: Vec<String> = packs
        .iter()
        .map(|pack| format!("{}.ini", pack.id))
        .collect();
    let mut to_delete = previous;
    if let Ok(entries) = std::fs::read_dir(&target) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if ours.iter().any(|own| *own == name) {
                to_delete.push(name);
            }
        }
    }
    to_delete.sort();
    to_delete.dedup();
    for old in to_delete {
        if !installed.iter().any(|name| name == &old) {
            let _ = std::fs::remove_file(target.join(old));
        }
    }

    let _ = std::fs::create_dir_all(&manifest_dir);
    let _ = std::fs::write(&manifest, installed.join("\n") + "\n");

    Ok(packs
        .iter()
        .enumerate()
        .filter(|(index, _)| chosen.contains(index))
        .map(|(_, pack)| pack.id.clone())
        .collect())
}

fn write_language_ini(alvr_root: &Path, language: &str) -> Result<(), String> {
    let text = format!(
        "; ============================================================\r\n\
         ;  ALVR 界面语言配置 / ALVR interface language\r\n\
         ;  可选值：en / zh-CN / zh-TW / ja-JP / ko-KR\r\n\
         ;  也可以在 ALVR「设置」页顶部标签栏右侧的「语言」下拉框里切换。\r\n\
         ; ============================================================\r\n\
         \r\n[language]\r\nlanguage = {language}\r\n"
    );
    std::fs::write(alvr_root.join("language.ini"), text)
        .map_err(|e| format!("write language.ini: {e}"))
}

fn install_build(install: &AlvrInstall, build_dir: &Path) -> Result<String, String> {
    let dashboard_src = build_dir.join("ALVR Dashboard.exe");
    let launcher_src = build_dir.join("ALVR Launcher.exe");

    patch::save_backup(&install.dashboard)?;
    std::fs::copy(&dashboard_src, &install.dashboard)
        .map_err(|e| format!("copy Dashboard (is ALVR still running?): {e}"))?;

    let mut message = format!("full build installed for {}", install.version);
    if let (Some(launcher), true) = (install.launcher.clone(), launcher_src.is_file()) {
        patch::save_backup(&launcher)?;
        std::fs::copy(&launcher_src, &launcher).map_err(|e| format!("copy Launcher: {e}"))?;
        message.push_str(" (incl. launcher)");
    }
    Ok(message)
}

/// `builds/v20.14.1` and `builds/20.14.1` are both accepted.
fn build_dir_for(paths: &Paths, version: &str) -> Option<PathBuf> {
    let trimmed = version.trim_start_matches(['v', 'V']);
    for candidate in [
        format!("v{trimmed}"),
        trimmed.to_owned(),
        version.to_owned(),
    ] {
        let dir = paths.builds.join(&candidate);
        if dir.join("ALVR Dashboard.exe").is_file() {
            return Some(dir);
        }
    }
    // last resort: scan the folder and compare normalised names
    for entry in std::fs::read_dir(&paths.builds).ok()?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let name = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        if name
            .trim_start_matches(['v', 'V'])
            .eq_ignore_ascii_case(trimmed)
        {
            return Some(dir);
        }
    }
    None
}

/// Apply the chosen packs. Returns (status key, log lines).
fn apply(
    install: &AlvrInstall,
    packs: &[LanguagePack],
    chosen: &[usize],
    default_pack: Option<&str>,
    paths: &Paths,
    options: &patch::PatchOptions,
) -> (&'static str, Vec<String>) {
    let snapshot = snapshot_session(install);
    let mut log = Vec::new();

    let installed = match sync_chosen_packs(install, packs, chosen) {
        Ok(ids) if !ids.is_empty() => ids,
        Ok(_) => return ("status_pick_pack", vec!["no pack selected".to_owned()]),
        Err(error) => return ("status_no_alvr", vec![error]),
    };
    log.push(format!(
        "packs -> ALVR\\languages: {}",
        installed.join(", ")
    ));

    let default_pack = default_pack
        .map(str::to_owned)
        .or_else(|| {
            chosen
                .iter()
                .map(|index| packs[*index].id.clone())
                .find(|id| id.eq_ignore_ascii_case("zh-CN"))
        })
        .unwrap_or_else(|| packs[chosen[0]].id.clone());
    if let Err(error) = write_language_ini(&install.root, &default_pack) {
        return ("status_no_alvr", vec![error]);
    }

    let build_dir = build_dir_for(paths, &install.version);
    let mut status = "status_applied_patch";
    if let Some(build_dir) = build_dir {
        match install_build(install, &build_dir) {
            Ok(message) => {
                log.push(message);
                status = "status_applied_full";
            }
            Err(error) => log.push(format!("full build failed, falling back: {error}")),
        }
    } else {
        log.push(format!(
            "no full build for {} — compatibility mode",
            install.version
        ));
    }

    if status == "status_applied_patch" {
        let mut targets = vec![install.dashboard.clone()];
        if let Some(launcher) = install.launcher.clone() {
            targets.push(launcher);
        }
        for target in targets {
            let name = target
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            match patch::patch_file(&target, &packs[chosen[0]], options) {
                Ok(None) => log.push(format!("{name}: already localized")),
                Ok(Some(report)) => {
                    log.push(format!(
                        "{name}: {} strings, font {}",
                        report.applied,
                        report.font.unwrap_or_else(|| "unchanged".into())
                    ));
                    if !report.skipped_too_long.is_empty() {
                        log.push(format!(
                            "  {} entries kept in English (too long for the byte budget)",
                            report.skipped_too_long.len()
                        ));
                    }
                    if !report.skipped_unsafe.is_empty() {
                        log.push(format!(
                            "  {} entries kept in English (protected values)",
                            report.skipped_unsafe.len()
                        ));
                    }
                }
                Err(error) => log.push(format!("{name}: patch failed: {error}")),
            }
        }
        if let Some(message) = smoke_test(&install.dashboard) {
            log.push(message);
        }
    }

    if let Some(message) = ensure_session_intact(&snapshot) {
        log.push(message);
    }
    (status, log)
}

fn smoke_test(dashboard: &Path) -> Option<String> {
    let mut child = std::process::Command::new(dashboard).spawn().ok()?;
    std::thread::sleep(std::time::Duration::from_secs(5));
    let failed = matches!(child.try_wait(), Ok(Some(status)) if !status.success());
    let _ = child.kill();
    let _ = child.wait();
    std::thread::sleep(std::time::Duration::from_millis(500));
    if failed {
        let restored = patch::restore_file(dashboard).unwrap_or(false);
        Some(if restored {
            "self-check failed: restored the official build automatically".to_owned()
        } else {
            "self-check failed and restore failed — please restore manually".to_owned()
        })
    } else {
        Some("self-check passed: the localized app starts fine".to_owned())
    }
}

fn restore(install: &AlvrInstall) -> (&'static str, Vec<String>) {
    let mut log = Vec::new();
    let mut targets = vec![install.dashboard.clone()];
    if let Some(launcher) = install.launcher.clone() {
        targets.push(launcher);
    }
    let mut restored = 0;
    for target in targets {
        let name = target
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        match patch::restore_file(&target) {
            Ok(true) => {
                restored += 1;
                log.push(format!("restored: {name}"));
            }
            Ok(false) => log.push(format!("{name}: no backup")),
            Err(error) => log.push(format!("{name}: {error}")),
        }
    }
    let _ = write_language_ini(&install.root, "en");
    (
        if restored > 0 {
            "status_restored"
        } else {
            "status_ready"
        },
        log,
    )
}

/// Turn a .csv/.json export into an .ini pack inside `paths.packs`.
fn import_translation_file(paths: &Paths, file: &Path) -> Result<String, String> {
    let text = std::fs::read_to_string(file).map_err(|e| format!("read: {e}"))?;
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let id = file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("custom")
        .to_owned();

    let pairs = match extension.as_str() {
        "json" => pack::pairs_from_json(&text),
        _ => pack::pairs_from_csv(&text),
    };
    if pairs.is_empty() {
        return Err("import_empty".to_owned());
    }
    let content = pack::pack_from_pairs(&id, &id, &pairs);
    let needs_font = pairs.iter().any(|(_, translation)| {
        translation.chars().any(|c| {
            let code = c as u32;
            (0x0590..=0x05FF).contains(&code)   // Hebrew
                || (0x0600..=0x06FF).contains(&code) // Arabic
                || (0x0900..=0x0DFF).contains(&code) // Indic
                || (0x0E00..=0x0E7F).contains(&code) // Thai
                || (0x1000..=0x109F).contains(&code) // Myanmar
        })
    });
    let font_missing = needs_font && !paths.packs.join(format!("{id}.ttf")).is_file();
    let target = paths.packs.join(format!("{id}.ini"));
    std::fs::create_dir_all(&paths.packs).map_err(|e| format!("mkdir: {e}"))?;
    std::fs::write(&target, content).map_err(|e| format!("write: {e}"))?;
    if font_missing {
        return Ok(format!("{id}.ini|font"));
    }
    Ok(format!("{id}.ini"))
}

/// Write an English -> (empty) CSV that translators can fill in.
fn export_template(paths: &Paths, target: &Path) -> Result<usize, String> {
    let reference = paths.packs.join("zh-CN.ini");
    let keys = pack::english_keys(&reference);
    if keys.is_empty() {
        return Err("reference pack missing".to_owned());
    }
    let mut csv = String::from("English,Translation\n");
    for key in &keys {
        let escaped = if key.contains(',') || key.contains('"') || key.contains('\n') {
            format!("\"{}\"", key.replace('"', "\"\""))
        } else {
            key.clone()
        };
        csv.push_str(&escaped);
        csv.push_str(",\n");
    }
    std::fs::write(target, csv).map_err(|e| format!("write: {e}"))?;
    Ok(keys.len())
}

/// Minimal HTTPS GET via the curl.exe that ships with Windows 10+.
fn http_get(url: &str) -> Option<String> {
    let output = std::process::Command::new("curl")
        .args(["-L", "--silent", "--max-time", "40", url])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Download every language pack of the project that is not here yet.
fn fetch_language_packs(paths: &Paths) -> Result<Vec<String>, String> {
    let listing = http_get(PACKS_INDEX).ok_or_else(|| "network".to_owned())?;
    let items: serde_json::Value =
        serde_json::from_str(&listing).map_err(|_| "network".to_owned())?;
    let array = items.as_array().ok_or_else(|| "network".to_owned())?;

    std::fs::create_dir_all(&paths.packs).map_err(|e| format!("mkdir: {e}"))?;
    let mut downloaded = Vec::new();
    for item in array {
        let Some(name) = item.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let lower = name.to_ascii_lowercase();
        let is_pack = lower.ends_with(".ini") || lower.ends_with(".ttf");
        if !is_pack || lower == "deny.txt" {
            continue;
        }
        let target = paths.packs.join(name);
        if target.is_file() {
            continue;
        }
        let Some(url) = item.get("download_url").and_then(|v| v.as_str()) else {
            continue;
        };
        if let Some(body) = http_get(url) {
            let _ = std::fs::write(&target, body);
            downloaded.push(name.to_owned());
        }
    }
    Ok(downloaded)
}

fn add_pack_dialog(paths: &Paths, title: &str) -> Option<PathBuf> {
    let file = rfd::FileDialog::new()
        .set_title(title)
        .add_filter(
            "Language pack or translation",
            &["ini", "csv", "json", "txt"],
        )
        .pick_file()?;
    let extension = file
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "ini" {
        return import_translation_file(paths, &file)
            .ok()
            .map(|name| paths.packs.join(name));
    }
    let name = file.file_name()?.to_owned();
    std::fs::create_dir_all(&paths.packs).ok()?;
    let target = paths.packs.join(&name);
    std::fs::copy(&file, &target).ok()?;
    let font = file.with_extension("ttf");
    if font.is_file() {
        if let Some(font_name) = font.file_name() {
            let _ = std::fs::copy(&font, paths.packs.join(font_name));
        }
    }
    Some(target)
}

/// Validate and install a `session.json` from another machine.
fn import_config(install: &AlvrInstall, source: &Path) -> Result<(String, Option<String>), String> {
    let text = std::fs::read_to_string(source).map_err(|e| format!("read file: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|_| "not an ALVR session.json".to_owned())?;
    if value.get("session_settings").is_none() {
        return Err("not an ALVR session.json".to_owned());
    }
    let version = value
        .get("server_version")
        .and_then(|v| v.as_str())
        .map(str::to_owned);

    let _ = backup_settings(install);
    let target = session_file(install);
    std::fs::write(&target, text.as_bytes()).map_err(|e| format!("write session.json: {e}"))?;
    Ok((target.display().to_string(), version))
}

// ---------------------------------------------------------------- command line

fn run_cli(paths: &Paths) -> Option<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |name: &str| args.iter().any(|a| a == name);
    let value = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };

    let do_apply = flag("--apply");
    let do_restore = flag("--restore");
    let do_backup = flag("--backup-settings");
    let do_fetch = flag("--fetch-packs");
    let do_template = value("--export-template");
    let do_import = value("--import");
    if !do_apply
        && !do_restore
        && !do_backup
        && !do_fetch
        && do_template.is_none()
        && do_import.is_none()
    {
        return None;
    }

    if let Some(target) = do_template {
        let log_path = exe_dir().join("alvr-l10n.log");
        let result = export_template(paths, Path::new(&target));
        let text = match result {
            Ok(count) => format!("OK: wrote {count} strings to {target}\n"),
            Err(error) => format!("ERROR: {error}\n"),
        };
        let _ = std::fs::write(&log_path, text);
        return Some(());
    }
    if let Some(source) = do_import {
        let log_path = exe_dir().join("alvr-l10n.log");
        let result = import_translation_file(paths, Path::new(&source));
        let text = match result {
            Ok(name) => format!("OK: {name}\n"),
            Err(error) => format!("ERROR: {error}\n"),
        };
        let _ = std::fs::write(&log_path, text);
        return Some(());
    }
    if do_fetch {
        let log_path = exe_dir().join("alvr-l10n.log");
        let text = match fetch_language_packs(paths) {
            Ok(list) if list.is_empty() => "OK: nothing new\n".to_owned(),
            Ok(list) => format!("OK: {}", list.join(", ")),
            Err(error) => format!("ERROR: {error}\n"),
        };
        let _ = std::fs::write(&log_path, text);
        return Some(());
    }

    let log_path = exe_dir().join("alvr-l10n.log");
    let mut out = String::new();
    let install = match value("--path") {
        Some(path) => alvr::detect_in(Path::new(&path)),
        None => alvr::detect(),
    };
    let Some(install) = install else {
        out.push_str("ERROR: ALVR not found\n");
        let _ = std::fs::write(&log_path, out);
        return Some(());
    };
    out.push_str(&format!(
        "ALVR: {}\nversion: {}\n",
        install.root.display(),
        install.version
    ));

    let packs = pack::load_dir(&paths.packs);
    let (status, log) = if do_restore {
        restore(&install)
    } else if do_backup {
        match backup_settings(&install) {
            Ok(path) => (
                "status_backup",
                vec![format!("saved to {}", path.display())],
            ),
            Err(error) => ("status_no_alvr", vec![error]),
        }
    } else {
        let wanted: Vec<String> = value("--lang")
            .map(|v| v.split(',').map(|s| s.trim().to_owned()).collect())
            .unwrap_or_else(|| vec!["zh-CN".to_owned()]);
        let chosen: Vec<usize> = packs
            .iter()
            .enumerate()
            .filter(|(_, pack)| wanted.iter().any(|w| pack.id.eq_ignore_ascii_case(w)))
            .map(|(index, _)| index)
            .collect();
        if chosen.is_empty() {
            (
                "status_pick_pack",
                vec![format!("no pack of {wanted:?} found")],
            )
        } else {
            let options = patch::PatchOptions {
                font: !flag("--no-font"),
                strings: !flag("--no-strings"),
                deny: load_deny(paths),
            };
            apply(
                &install,
                &packs,
                &chosen,
                wanted.first().map(String::as_str),
                paths,
                &options,
            )
        }
    };
    out.push_str(&format!("STATUS: {}\n", lang::UiLang::En.t(status)));
    for line in log {
        out.push_str(&format!("  {line}\n"));
    }
    let _ = std::fs::write(&log_path, out);
    Some(())
}

// ------------------------------------------------------------------------ gui

const NAV_WIDTH: f32 = 180.0;

#[derive(PartialEq, Clone, Copy)]
enum Page {
    Localize,
    Help,
    About,
}

#[derive(PartialEq)]
enum Action {
    None,
    Redetect,
    BrowseExe,
    FetchPacks,
    ExportTemplate,
    Apply,
    Restore,
    Backup,
    AddPack,
    ImportConfig,
    OpenAlvr,
    OpenPackFolder,
    ToggleTheme,
    Share,
    Feedback,
    OpenUrl(&'static str),
}

struct App {
    install: Option<AlvrInstall>,
    packs: Vec<LanguagePack>,
    checked: Vec<bool>,
    ui_lang: UiLang,
    page: Page,
    dark: bool,
    log: Vec<String>,
    status_key: &'static str,
    status_ok: bool,
    status_extra: String,
    toast: Option<(String, f64)>,
    window_styled: bool,
    alvr_exe: Option<String>,
    paths: Paths,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_ui_fonts(&cc.egui_ctx);
        let base = exe_dir();
        let paths = Paths {
            packs: base.join("languages"),
            builds: base.join("builds"),
            settings: base.join("tool.ini"),
        };
        let prefs = load_prefs(&paths.settings);
        theme::apply(&cc.egui_ctx, prefs.dark);
        let install = prefs
            .alvr_exe
            .as_ref()
            .map(Path::new)
            .and_then(alvr::detect_exe)
            .or_else(alvr::detect);
        let packs = pack::load_dir(&paths.packs);
        let mut app = Self {
            install,
            packs,
            checked: Vec::new(),
            ui_lang: prefs.lang,
            page: match std::env::var("ALVR_L10N_PAGE").as_deref() {
                Ok("about") => Page::About,
                Ok("help") => Page::Help,
                _ => Page::Localize,
            },
            dark: prefs.dark,
            log: Vec::new(),
            status_key: "status_ready",
            status_ok: true,
            status_extra: String::new(),
            toast: None,
            window_styled: false,
            alvr_exe: prefs.alvr_exe,
            paths,
        };
        app.sync_checked_with_alvr();
        app.summarize();
        app
    }

    fn t(&self, key: &str) -> &'static str {
        self.ui_lang.t(key)
    }

    fn sync_checked_with_alvr(&mut self) {
        let installed: Vec<String> = self
            .install
            .as_ref()
            .map(|install| {
                std::fs::read_dir(install.root.join("languages"))
                    .map(|entries| {
                        entries
                            .flatten()
                            .filter_map(|entry| {
                                let path = entry.path();
                                if path.extension().and_then(|e| e.to_str()) != Some("ini") {
                                    return None;
                                }
                                path.file_stem().and_then(|s| s.to_str()).map(str::to_owned)
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        self.checked = self
            .packs
            .iter()
            .map(|pack| {
                if installed.is_empty() {
                    pack.id.eq_ignore_ascii_case("zh-CN")
                } else {
                    installed.iter().any(|id| id.eq_ignore_ascii_case(&pack.id))
                }
            })
            .collect();
    }

    fn summarize(&mut self) {
        self.log.clear();
        match &self.install {
            Some(found) => {
                let build = build_dir_for(&self.paths, &found.version);
                self.log.push(if build.is_some() {
                    self.t("full_build_ok").to_owned()
                } else {
                    self.t("compat_only").to_owned()
                });
            }
            None => self.log.push(self.t("not_detected").to_owned()),
        }
        self.log.push(self.t("about_packs").to_owned());
        self.status_key = "status_ready";
        self.status_ok = true;
        self.status_extra.clear();
    }

    fn chosen(&self) -> Vec<usize> {
        self.checked
            .iter()
            .enumerate()
            .filter(|(_, on)| **on)
            .map(|(index, _)| index)
            .collect()
    }

    fn toast(&mut self, text: impl Into<String>) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        self.toast = Some((text.into(), now + 2.5));
    }

    fn set_theme(&mut self, ctx: &egui::Context, dark: bool) {
        self.dark = dark;
        theme::apply(ctx, dark);
        theme::apply_window_decoration(WINDOW_TITLE, dark);
        save_prefs(
            &self.paths.settings,
            UiPrefs {
                lang: self.ui_lang,
                dark,
                alvr_exe: self.alvr_exe.clone(),
            },
        );
    }

    fn perform(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::None => {}
            Action::Feedback => {
                ctx.copy_text(self.t("about_email").to_owned());
                let _ = std::process::Command::new("cmd")
                    .args([
                        "/c",
                        "start",
                        "",
                        "mailto:anshanzi2018@gmail.com?subject=ALVR-L10N%20feedback",
                    ])
                    .spawn();
                self.toast(format!(
                    "{} {}",
                    self.t("feedback_copied"),
                    self.t("about_email")
                ));
            }
            Action::OpenUrl(url) => {
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", url])
                    .spawn();
            }
            Action::Share => {
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "clip"])
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                    .and_then(|mut child| {
                        use std::io::Write;
                        if let Some(stdin) = child.stdin.as_mut() {
                            stdin.write_all(PROJECT_URL.as_bytes())?;
                        }
                        child.wait()
                    });
                self.toast(self.t("share_copied"));
            }
            Action::ToggleTheme => {}
            Action::FetchPacks => {
                self.status_key = "fetch_busy";
                self.status_ok = true;
                self.status_extra.clear();
                match fetch_language_packs(&self.paths) {
                    Ok(downloaded) if downloaded.is_empty() => {
                        self.status_key = "fetch_none";
                        self.status_ok = true;
                        self.log = vec![self.t("fetch_none").to_owned()];
                    }
                    Ok(downloaded) => {
                        self.packs = pack::load_dir(&self.paths.packs);
                        self.sync_checked_with_alvr();
                        self.status_key = "fetch_ok";
                        self.status_ok = true;
                        self.status_extra = downloaded.join(", ");
                        self.log =
                            vec![format!("{} {}", self.t("fetch_ok"), downloaded.join(", "))];
                    }
                    Err(_) => {
                        self.status_key = "fetch_fail";
                        self.status_ok = false;
                        self.log = vec![self.t("fetch_fail").to_owned()];
                    }
                }
            }
            Action::ExportTemplate => {
                if let Some(target) = rfd::FileDialog::new()
                    .set_title(self.t("export_template"))
                    .set_file_name("alvr-translation-template.csv")
                    .add_filter("CSV", &["csv"])
                    .save_file()
                {
                    match export_template(&self.paths, &target) {
                        Ok(count) => {
                            self.status_key = "template_saved";
                            self.status_ok = true;
                            self.status_extra = target.display().to_string();
                            self.log = vec![format!(
                                "{} {} ({} strings)",
                                self.t("template_saved"),
                                target.display(),
                                count
                            )];
                        }
                        Err(error) => {
                            self.status_key = "template_fail";
                            self.status_ok = false;
                            self.log = vec![error];
                        }
                    }
                }
            }
            Action::BrowseExe => {
                let picked = rfd::FileDialog::new()
                    .set_title("ALVR Dashboard.exe")
                    .add_filter("ALVR Dashboard", &["exe"])
                    .pick_file();
                if let Some(file) = picked {
                    match alvr::detect_exe(&file) {
                        Some(found) => {
                            self.install = Some(found);
                            self.alvr_exe = Some(file.display().to_string());
                            self.packs = pack::load_dir(&self.paths.packs);
                            self.sync_checked_with_alvr();
                            self.summarize();
                            self.status_key = "exe_using";
                            self.status_ok = true;
                            self.status_extra = file.display().to_string();
                            save_prefs(
                                &self.paths.settings,
                                UiPrefs {
                                    lang: self.ui_lang,
                                    dark: self.dark,
                                    alvr_exe: self.alvr_exe.clone(),
                                },
                            );
                        }
                        None => {
                            self.status_key = "exe_invalid";
                            self.status_ok = false;
                            self.status_extra.clear();
                        }
                    }
                }
            }
            Action::Redetect => {
                self.install = self
                    .alvr_exe
                    .as_ref()
                    .map(Path::new)
                    .and_then(alvr::detect_exe)
                    .or_else(alvr::detect);
                self.packs = pack::load_dir(&self.paths.packs);
                self.sync_checked_with_alvr();
                self.summarize();
            }
            Action::AddPack => {
                if let Some(path) = add_pack_dialog(&self.paths, self.t("import_any_title")) {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.packs = pack::load_dir(&self.paths.packs);
                    self.sync_checked_with_alvr();
                    let needs_font = name.ends_with("|font");
                    let name = name.trim_end_matches("|font").to_owned();
                    self.log = vec![format!("{} {}", self.t("pack_added"), name)];
                    if needs_font {
                        self.log.push(self.t("font_needed").to_owned());
                    }
                    self.status_key = "status_ready";
                    self.status_ok = true;
                } else {
                    self.log = vec![self.t("pack_add_failed").to_owned()];
                }
            }
            Action::OpenPackFolder => {
                let _ = std::fs::create_dir_all(&self.paths.packs);
                let _ = std::process::Command::new("explorer")
                    .arg(&self.paths.packs)
                    .spawn();
            }
            Action::OpenAlvr => {
                if let Some(install) = &self.install {
                    let _ = std::process::Command::new("explorer")
                        .arg(&install.root)
                        .spawn();
                }
            }
            Action::Backup => {
                if let Some(install) = self.install.clone() {
                    match backup_settings(&install) {
                        Ok(path) => {
                            self.status_key = "status_backup";
                            self.status_ok = true;
                            self.status_extra = path.display().to_string();
                            self.log = vec![format!("{} {}", self.t("saved_to"), path.display())];
                        }
                        Err(error) => {
                            self.status_key = "status_no_alvr";
                            self.status_ok = false;
                            self.log = vec![error];
                        }
                    }
                } else {
                    self.status_key = "status_no_alvr";
                    self.status_ok = false;
                }
            }
            Action::ImportConfig => {
                if let Some(install) = self.install.clone() {
                    if let Some(file) = rfd::FileDialog::new()
                        .set_title(self.t("import_title"))
                        .add_filter("ALVR session", &["json", "txt"])
                        .pick_file()
                    {
                        match import_config(&install, &file) {
                            Ok((path, version)) => {
                                self.status_key = "import_ok";
                                self.status_ok = true;
                                self.status_extra.clear();
                                let mut log = vec![format!("{} {}", self.t("saved_to"), path)];
                                if let Some(version) = version {
                                    log.push(format!("{} {}", self.t("import_version"), version));
                                }
                                self.log = log;
                            }
                            Err(error) => {
                                self.status_key = "import_bad";
                                self.status_ok = false;
                                self.log = vec![error];
                            }
                        }
                    }
                } else {
                    self.status_key = "status_no_alvr";
                    self.status_ok = false;
                }
            }
            Action::Restore => {
                if let Some(install) = self.install.clone() {
                    let (status, log) = restore(&install);
                    self.status_key = status;
                    self.status_ok = true;
                    self.log = log;
                } else {
                    self.status_key = "status_no_alvr";
                    self.status_ok = false;
                }
            }
            Action::Apply => {
                let chosen = self.chosen();
                if chosen.is_empty() {
                    self.status_key = "status_pick_pack";
                    self.status_ok = false;
                    return;
                }
                if let Some(install) = self.install.clone() {
                    let options = patch::PatchOptions {
                        deny: load_deny(&self.paths),
                        ..patch::PatchOptions::all()
                    };
                    let (status, log) =
                        apply(&install, &self.packs, &chosen, None, &self.paths, &options);
                    self.status_key = status;
                    self.status_ok = status != "status_no_alvr";
                    self.log = log;
                } else {
                    self.status_key = "status_no_alvr";
                    self.status_ok = false;
                }
            }
        }
    }

    // ------------------------------------------------------------- page: home

    fn home_ui(&mut self, ui: &mut egui::Ui, action: &mut Action) {
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(self.t("page_localize"))
                .size(20.0)
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.add_space(2.0);
        match &self.install {
            Some(found) => {
                ui.label(
                    egui::RichText::new(format!(
                        "{}: {}   ({} {})",
                        self.t("alvr_path"),
                        found.root.display(),
                        self.t("version"),
                        found.version
                    ))
                    .weak(),
                );
            }
            None => {
                ui.colored_label(egui::Color32::from_rgb(220, 90, 90), self.t("not_detected"));
            }
        }

        ui.add_space(6.0);
        if ui
            .button(self.t("browse_exe"))
            .on_hover_text(self.t("tip_browse_exe"))
            .clicked()
        {
            *action = Action::BrowseExe;
        }

        ui.add_space(12.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(self.t("packs"));
            ui.add(
                egui::Label::new(egui::RichText::new(self.t("packs_hint")).small().weak()).wrap(),
            );
            ui.add_space(6.0);
            if self.packs.is_empty() {
                ui.label(self.t("no_packs"));
            }
            let mut toggled: Option<usize> = None;
            for (index, pack) in self.packs.iter().enumerate() {
                let selected = self.checked.get(index).copied().unwrap_or(false);
                let label = format!("{}   ({})", pack.label(), pack.id);
                if ui.selectable_label(selected, label).clicked() {
                    toggled = Some(index);
                }
            }
            if let Some(index) = toggled {
                if self.checked.len() <= index {
                    self.checked.resize(index + 1, false);
                }
                if let Some(slot) = self.checked.get_mut(index) {
                    *slot = !*slot;
                }
            }
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui
                    .button(self.t("add_pack"))
                    .on_hover_text(self.t("tip_add_pack"))
                    .clicked()
                {
                    *action = Action::AddPack;
                }
                if ui
                    .button(self.t("open_pack_folder"))
                    .on_hover_text(self.t("tip_open_pack_folder"))
                    .clicked()
                {
                    *action = Action::OpenPackFolder;
                }
                if ui.button(self.t("fetch_packs")).clicked() {
                    *action = Action::FetchPacks;
                }
                if ui.button(self.t("export_template")).clicked() {
                    *action = Action::ExportTemplate;
                }
            });
            ui.add_space(4.0);
        });

        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [132.0, 34.0],
                    egui::Button::new(egui::RichText::new(self.t("apply")).size(15.0)),
                )
                .clicked()
            {
                *action = Action::Apply;
            }
            if ui
                .add_sized(
                    [150.0, 34.0],
                    egui::Button::new(egui::RichText::new(self.t("restore")).size(15.0)),
                )
                .clicked()
            {
                *action = Action::Restore;
            }
            if ui
                .add_sized(
                    [160.0, 34.0],
                    egui::Button::new(egui::RichText::new(self.t("backup")).size(15.0)),
                )
                .clicked()
            {
                *action = Action::Backup;
            }
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui.button(self.t("import_config")).clicked() {
                *action = Action::ImportConfig;
            }
            if ui.button(self.t("open_alvr")).clicked() {
                *action = Action::OpenAlvr;
            }
        });

        ui.add_space(10.0);
        let color = if self.status_ok {
            egui::Color32::from_rgb(110, 200, 130)
        } else {
            egui::Color32::from_rgb(225, 95, 95)
        };
        let mut status = self.t(self.status_key).to_owned();
        if !self.status_extra.is_empty() {
            status.push_str(": ");
            status.push_str(&self.status_extra);
        }
        ui.add(egui::Label::new(egui::RichText::new(status).size(15.0).color(color)).wrap());
        ui.add(
            egui::Label::new(egui::RichText::new(self.t("settings_note")).small().weak()).wrap(),
        );

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(self.t("log_title"))
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.add_space(4.0);
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(ui.available_height() - 8.0)
            .show(ui, |ui| {
                for line in &self.log {
                    ui.add(egui::Label::new(line).wrap());
                }
            });
    }

    // ------------------------------------------------------------- page: help

    fn help_ui(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(self.t("nav_help"))
                .size(20.0)
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.add_space(8.0);
        ui.add(egui::Label::new(self.t("help_intro")).wrap());
        ui.add_space(12.0);

        for (title, body) in [
            ("help_lang_title", "help_lang_body"),
            ("help_custom_title", "help_custom_body"),
            ("help_pack_title", "help_pack_body"),
            ("help_font_title", "help_font_body"),
            ("help_where_title", "help_where_body"),
            ("help_safety_title", "help_safety_body"),
        ] {
            ui.label(
                egui::RichText::new(self.t(title))
                    .strong()
                    .color(ui.visuals().text_color()),
            );
            ui.add(egui::Label::new(self.t(body)).wrap());
            ui.add_space(10.0);
        }
    }

    // ------------------------------------------------------------ page: about

    fn about_ui(&mut self, ui: &mut egui::Ui, action: &mut Action) {
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(self.t("nav_about"))
                .size(20.0)
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(format!(
                "{}  v{}",
                self.t("title"),
                env!("CARGO_PKG_VERSION")
            ))
            .size(26.0)
            .strong()
            .color(ui.visuals().text_color()),
        );
        ui.add_space(2.0);
        ui.label(egui::RichText::new(self.t("about_tagline")).weak());
        ui.add_space(16.0);

        link_row(ui, "🌐", self.t("about_github"), PROJECT_URL, action);
        let clicked = ui
            .button(egui::RichText::new(format!(
                "✉   {}",
                self.t("about_feedback")
            )))
            .on_hover_text(self.t("about_email"))
            .clicked();
        if clicked {
            *action = Action::Feedback;
        }
        let clicked = ui
            .button(egui::RichText::new(format!(
                "📋  {}",
                self.t("about_share")
            )))
            .clicked();
        if clicked {
            *action = Action::Share;
        }

        ui.add_space(16.0);
        ui.label(
            egui::RichText::new(self.t("about_license"))
                .strong()
                .color(ui.visuals().text_color()),
        );
        ui.add_space(4.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(220.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(egui::RichText::new(include_str!("../LICENSE")).small())
                            .wrap(),
                    );
                });
        });
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(self.t("about_third_party"))
                .small()
                .weak(),
        );
        let clicked = ui.button(self.t("about_changelog")).clicked();
        if clicked {
            *action = Action::OpenUrl(PROJECT_URL);
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.window_styled {
            theme::apply_window_decoration(WINDOW_TITLE, self.dark);
            self.window_styled = true;
        }
        {
            let path = std::path::Path::new(r"G:\ALVR-L10N\ui-debug.txt");
            let _ = std::fs::write(
                path,
                format!(
                    "dark={} panel={:?} expected={:?} override={:?} pref={:?} page={}\n",
                    self.dark,
                    ctx.style().visuals.panel_fill,
                    theme::expected_panel(self.dark),
                    ctx.style().visuals.override_text_color,
                    ctx.options(|o| o.theme_preference),
                    if self.page == Page::About {
                        "about"
                    } else {
                        "localize"
                    }
                ),
            );
        }

        // eframe follows the system theme; keep our own choice pinned.
        let current = &ctx.style().visuals;
        if current.panel_fill != theme::expected_panel(self.dark)
            || current.override_text_color != Some(theme::expected_text(self.dark))
        {
            theme::apply(ctx, self.dark);
        }

        let mut action = Action::None;

        egui::SidePanel::left("nav")
            .exact_width(NAV_WIDTH)
            .resizable(false)
            .show(ctx, |ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("ALVR")
                        .size(26.0)
                        .strong()
                        .color(ui.visuals().text_color()),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "{}  v{}",
                        self.t("nav_tool"),
                        env!("CARGO_PKG_VERSION")
                    ))
                    .small()
                    .color(ui.visuals().weak_text_color()),
                );
                ui.add_space(20.0);

                if nav_row(
                    ui,
                    "🔌",
                    self.t("nav_localize"),
                    self.page == Page::Localize,
                ) {
                    self.page = Page::Localize;
                }
                if nav_row(ui, "?", self.t("nav_help"), self.page == Page::Help) {
                    self.page = Page::Help;
                }
                if nav_row(ui, "ℹ", self.t("nav_about"), self.page == Page::About) {
                    self.page = Page::About;
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.add_space(12.0);
                    if nav_row(
                        ui,
                        if self.dark { "☀" } else { "🌙" },
                        if self.dark {
                            self.t("theme_light")
                        } else {
                            self.t("theme_dark")
                        },
                        false,
                    ) {
                        action = Action::ToggleTheme;
                    }
                    ui.add_space(6.0);
                    let mut chosen = self.ui_lang;
                    egui::ComboBox::from_id_salt("ui_language")
                        .width(160.0)
                        .selected_text(self.ui_lang.label())
                        .show_ui(ui, |ui| {
                            for candidate in UI_LANGS {
                                ui.selectable_value(&mut chosen, candidate, candidate.label());
                            }
                        });
                    if chosen != self.ui_lang {
                        self.ui_lang = chosen;
                        save_prefs(
                            &self.paths.settings,
                            UiPrefs {
                                lang: chosen,
                                dark: self.dark,
                                alvr_exe: self.alvr_exe.clone(),
                            },
                        );
                        self.summarize();
                    }
                    ui.add_space(4.0);
                    if ui.button(self.t("redetect")).clicked() {
                        action = Action::Redetect;
                    }
                });
            });

        let panel_frame =
            egui::Frame::central_panel(&ctx.style()).inner_margin(egui::Margin::same(16));
        egui::CentralPanel::default()
            .frame(panel_frame)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| match self.page {
                        Page::Localize => self.home_ui(ui, &mut action),
                        Page::Help => self.help_ui(ui),
                        Page::About => self.about_ui(ui, &mut action),
                    });
            });

        if action == Action::ToggleTheme {
            let dark = !self.dark;
            self.set_theme(ctx, dark);
        } else {
            self.perform(ctx, action);
        }

        // toast
        if let Some((text, until)) = self.toast.clone() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0);
            if now < until {
                egui::Area::new(egui::Id::new("toast"))
                    .anchor(egui::Align2::CENTER_BOTTOM, egui::vec2(0.0, -18.0))
                    .show(ctx, |ui| {
                        egui::Frame::window(ui.style()).show(ui, |ui| {
                            ui.label(egui::RichText::new(text).strong());
                        });
                    });
                ctx.request_repaint_after(std::time::Duration::from_millis(120));
            } else {
                self.toast = None;
            }
        }
    }
}

/// A sidebar navigation entry.
fn nav_row(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> bool {
    let height = 36.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    let visuals = ui.visuals();
    let fill = if selected {
        theme::ACCENT
    } else if response.hovered() {
        visuals.widgets.hovered.bg_fill
    } else {
        egui::Color32::TRANSPARENT
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(6), fill);
    let color = if selected {
        egui::Color32::WHITE
    } else {
        visuals.text_color()
    };
    // icon gets a fixed slot so every label starts at the same x
    let icon_rect = egui::Rect::from_min_size(
        rect.left_top() + egui::vec2(8.0, 0.0),
        egui::vec2(28.0, height),
    );
    ui.painter().text(
        icon_rect.center(),
        egui::Align2::CENTER_CENTER,
        icon,
        egui::FontId::proportional(15.0),
        color,
    );
    ui.painter().text(
        egui::pos2(rect.left() + 44.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(15.0),
        color,
    );
    response.clicked()
}

/// A link-like row used on the About page.
fn link_row(ui: &mut egui::Ui, icon: &str, label: &str, url: &'static str, action: &mut Action) {
    let height = 30.0;
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), height),
        egui::Sense::click(),
    );
    if response.hovered() {
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(6),
            ui.visuals().widgets.hovered.bg_fill,
        );
    }
    let color = if response.hovered() {
        theme::ACCENT
    } else {
        ui.visuals().text_color()
    };
    ui.painter().text(
        rect.left_center() + egui::vec2(4.0, 0.0),
        egui::Align2::LEFT_CENTER,
        format!("{icon}   {label}"),
        egui::FontId::proportional(15.0),
        color,
    );
    if response.clicked() {
        *action = Action::OpenUrl(url);
    }
}

fn install_ui_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "ui-cjk".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(UI_FONT_SC)),
    );
    fonts.font_data.insert(
        "ui-kr".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(UI_FONT_KR)),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        if let Some(list) = fonts.families.get_mut(&family) {
            list.push("ui-cjk".to_owned());
            list.push("ui-kr".to_owned());
        }
    }
    ctx.set_fonts(fonts);
}

/// Windows scaling factor (1.0 at 96 DPI, 1.5 at 150%).
#[cfg(windows)]
fn system_scale() -> f32 {
    #[link(name = "user32")]
    extern "system" {
        fn GetDpiForSystem() -> u32;
    }
    let dpi = unsafe { GetDpiForSystem() };
    if dpi == 0 {
        1.0
    } else {
        dpi as f32 / 96.0
    }
}

#[cfg(not(windows))]
fn system_scale() -> f32 {
    1.0
}

fn main() -> eframe::Result<()> {
    let base = exe_dir();
    let paths = Paths {
        packs: base.join("languages"),
        builds: base.join("builds"),
        settings: base.join("tool.ini"),
    };
    if run_cli(&paths).is_some() {
        return Ok(());
    }

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 580.0])
            .with_min_inner_size([680.0, 520.0])
            .with_title(WINDOW_TITLE),
        ..Default::default()
    };
    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
