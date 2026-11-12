//! In-place patching: swap the embedded font for a CJK subset and translate
//! string literals, keeping every replacement inside the original byte budget.

use crate::pack::LanguagePack;
use crate::sfnt;
use std::path::{Path, PathBuf};

/// Folder (inside the ALVR installation folder) that holds our backups.
pub const BACKUP_DIR: &str = ".alvr-l10n";
/// Suffix of the older backups created before the backup folder existed.
pub const BACKUP_SUFFIX: &str = ".alvr-l10n-orig";
/// Backup suffix used by the very first one-click installer, honoured for restore.
pub const LEGACY_BACKUP_SUFFIX: &str = ".orig";

#[derive(Default, Debug)]
pub struct PatchReport {
    pub applied: usize,
    pub skipped_too_long: Vec<String>,
    pub skipped_unsafe: Vec<String>,
    pub skipped_missing: Vec<String>,
    pub font: Option<String>,
    pub notes: Vec<String>,
}

pub fn backup_path(target: &Path, suffix: &str) -> PathBuf {
    PathBuf::from(format!("{}{suffix}", target.display()))
}

/// Folder that keeps the pristine executables, next to the ALVR binaries.
pub fn backup_dir(target: &Path) -> PathBuf {
    target
        .parent()
        .map(|parent| parent.join(BACKUP_DIR))
        .unwrap_or_else(|| PathBuf::from(BACKUP_DIR))
}

/// Current backup location: `<install dir>/.alvr-l10n/<exe name>`.
pub fn backup_file(target: &Path) -> PathBuf {
    backup_dir(target).join(target.file_name().unwrap_or_default())
}

/// Copy `target` to the backup folder (no-op when a backup already exists).
pub fn save_backup(target: &Path) -> Result<(), String> {
    if original_of(target).is_some() || !target.is_file() {
        return Ok(());
    }
    let dir = backup_dir(target);
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建备份目录失败：{e}"))?;
    std::fs::copy(target, backup_file(target))
        .map_err(|e| format!("备份失败（ALVR 是否还在运行？）：{e}"))?;
    Ok(())
}

/// Path of the pristine original for `target`, if we have one.
pub fn original_of(target: &Path) -> Option<PathBuf> {
    let file = backup_file(target);
    if file.is_file() {
        return Some(file);
    }
    for suffix in [BACKUP_SUFFIX, LEGACY_BACKUP_SUFFIX] {
        let path = backup_path(target, suffix);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

fn is_alnum(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

/// ALVR stores enum values (e.g. `"High"` in `session.json`) as plain strings, so
/// replacing a bare word can break deserialization and panic on startup. Only
/// translate text that is clearly prose: it contains a space, ends with
/// punctuation, or is a long sentence.
fn looks_like_ui_text(text: &str) -> bool {
    let trimmed = text.trim_end();
    if trimmed.is_empty() {
        return false;
    }
    let first = trimmed.chars().next().unwrap_or(' ');
    if first.is_ascii_lowercase() {
        return false;
    }
    let has_space = trimmed.contains(' ');
    let ends = matches!(
        trimmed.chars().last(),
        Some(':') | Some('!') | Some('?') | Some('.') | Some('…') | Some(')')
    );
    let has_arrow = trimmed.contains("->");
    has_space || ends || has_arrow
}

#[derive(Default, Debug)]
pub struct PatchOptions {
    pub font: bool,
    pub strings: bool,
    /// Strings that must never be replaced: ALVR stores enum/option names in
    /// session.json, so changing them breaks loading of existing settings.
    pub deny: std::collections::HashSet<String>,
}

impl PatchOptions {
    pub fn all() -> Self {
        Self {
            font: true,
            strings: true,
            deny: std::collections::HashSet::new(),
        }
    }
}

/// Apply `pack` to one executable. Returns `Ok(None)` when the file is already
/// one of our localized builds (nothing to do).
pub fn patch_file(
    target: &Path,
    pack: &LanguagePack,
    options: &PatchOptions,
) -> Result<Option<PatchReport>, String> {
    let original = original_of(target);
    let source_path = original.clone().unwrap_or_else(|| target.to_path_buf());
    let mut data = std::fs::read(&source_path)
        .map_err(|e| format!("读取 {} 失败：{e}", source_path.display()))?;

    // Already a localized build (e.g. installed from `builds/`)? Then just leave it.
    if original.is_none() && contains(&data, b"alvr_interface_language") {
        return Ok(None);
    }

    // Make sure we can restore later before touching anything.
    save_backup(target)?;

    let mut report = PatchReport::default();
    let mut occupied: Vec<(usize, usize)> = Vec::new();

    // 1. font
    if !options.font {
        // explicitly skipped for diagnostics
    } else if let Some(font_path) = &pack.font_path {
        match std::fs::read(font_path) {
            Ok(font) => match sfnt::find_blob(&data, "Ubuntu") {
                Some(blob) if font.len() <= blob.length => {
                    data[blob.offset..blob.offset + font.len()].copy_from_slice(&font);
                    for byte in &mut data[blob.offset + font.len()..blob.offset + blob.length] {
                        *byte = 0;
                    }
                    occupied.push((blob.offset, blob.offset + blob.length));
                    report.font = Some(format!(
                        "{} ({} KB, 槽位 {} KB)",
                        blob.family,
                        font.len() / 1024,
                        blob.length / 1024
                    ));
                }
                Some(blob) => report.notes.push(format!(
                    "字体过大：{} KB > 槽位 {} KB，已跳过字体替换",
                    font.len() / 1024,
                    blob.length / 1024
                )),
                None => report.notes.push("未在内嵌字库中找到 Ubuntu 字体槽位".into()),
            },
            Err(e) => report.notes.push(format!("读取字体失败：{e}")),
        }
    } else {
        report.notes.push("该语言没有附带字体，中文可能显示为方块".into());
    }

    // 2. strings, longest first so shorter keys never eat into a longer match
    let mut entries: Vec<(&String, &String)> = pack.strings.iter().collect();
    entries.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
    if !options.strings {
        entries.clear();
    }

    for (english, chinese) in entries {
        let needle = english.as_bytes();
        let replacement = chinese.as_bytes();
        if needle.is_empty() {
            continue;
        }
        if options.deny.contains(english) || !looks_like_ui_text(english) {
            report.skipped_unsafe.push(english.clone());
            continue;
        }
        if replacement.len() > needle.len() {
            report.skipped_too_long.push(english.clone());
            continue;
        }
        let mut padded = replacement.to_vec();
        padded.resize(needle.len(), b' ');

        let mut applied_here = 0usize;
        let mut index = 0usize;
        while index + needle.len() <= data.len() {
            let Some(relative) = find(&data[index..], needle) else {
                break;
            };
            let start = index + relative;
            index = start + 1;

            if occupied
                .iter()
                .any(|(from, to)| start < *to && start + needle.len() > *from)
            {
                continue;
            }
            // Never cut into a longer word / sentence fragment.
            if start > 0 && is_alnum(data[start - 1]) {
                continue;
            }
            let end = start + needle.len();
            if end < data.len() && is_alnum(data[end]) {
                continue;
            }
            if end + 1 < data.len() && data[end] == b' ' && data[end + 1].is_ascii_lowercase() {
                continue;
            }

            data[start..end].copy_from_slice(&padded);
            occupied.push((start, end));
            applied_here += 1;
        }

        if applied_here > 0 {
            report.applied += applied_here;
        } else {
            report.skipped_missing.push(english.clone());
        }
    }

    std::fs::write(target, &data).map_err(|e| format!("写入失败（ALVR 是否还在运行？）：{e}"))?;
    Ok(Some(report))
}

/// Restore the pristine executable from our backup.
pub fn restore_file(target: &Path) -> Result<bool, String> {
    let Some(original) = original_of(target) else {
        return Ok(false);
    };
    std::fs::copy(&original, target).map_err(|e| format!("还原失败（ALVR 是否还在运行？）：{e}"))?;
    Ok(true)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    find(haystack, needle).is_some()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    let first = needle[0];
    let mut index = 0usize;
    while index + needle.len() <= haystack.len() {
        let Some(offset) = haystack[index..]
            .iter()
            .position(|&b| b == first)
            .map(|p| index + p)
        else {
            return None;
        };
        if offset + needle.len() > haystack.len() {
            return None;
        }
        if &haystack[offset..offset + needle.len()] == needle {
            return Some(offset);
        }
        index = offset + 1;
    }
    None
}
