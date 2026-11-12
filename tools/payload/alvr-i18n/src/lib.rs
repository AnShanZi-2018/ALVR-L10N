//! Lightweight runtime localization layer for ALVR.
//!
//! ALVR's UI strings are hard-coded English literals. Instead of touching every call
//! site, translations are applied when text is laid out (see `epaint`'s
//! `set_text_translator`). That means the whole UI - labels, buttons, tooltips, help
//! texts, notifications - is translated without any string-length limitations.
//!
//! Everything language related is data, not code:
//!
//! * `<ALVR dir>/language.ini`     - the selected language
//! * `<ALVR dir>/languages/*.ini`  - one translation pack per language
//!
//! Both files are plain INI so users can add languages or fix wording without
//! recompiling ALVR. Unknown strings simply stay English, so a pack written for an
//! older ALVR keeps working after an update.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, Once, RwLock};

/// Name of the configuration file that selects the language.
pub const LANGUAGE_INI: &str = "language.ini";
/// Name of the directory that contains the translation packs.
pub const LANGUAGES_DIR: &str = "languages";
/// Identifier used for the built-in, untranslated English UI.
pub const ENGLISH: &str = "en";

/// Metadata of a translation pack found on disk.
#[derive(Clone, Debug)]
pub struct LanguageInfo {
    pub id: String,
    /// Name of the language written in that language, e.g. "绠€浣撲腑鏂?.
    pub native_name: String,
    /// Name of the language in English, e.g. "Simplified Chinese".
    pub english_name: String,
    pub author: String,
    pub version: String,
    pub path: PathBuf,
}

impl LanguageInfo {
    pub fn english() -> Self {
        Self {
            id: ENGLISH.to_owned(),
            native_name: "English".to_owned(),
            english_name: "English".to_owned(),
            author: String::new(),
            version: String::new(),
            path: PathBuf::new(),
        }
    }
}

struct Inner {
    lang: String,
    table: HashMap<String, String>,
    /// Keys that end with `*` in the language pack, matched as prefixes. They cover
    /// strings ALVR builds at runtime, e.g. `Tip: <random tip>` or `ALVR streamer v...`.
    prefixes: Vec<(String, String)>,
    dirs: Vec<PathBuf>,
    missing_log: Option<PathBuf>,
    missing: Mutex<HashSet<String>>,
}

static STATE: RwLock<Option<Inner>> = RwLock::new(None);
static INIT: Once = Once::new();

// ---------------------------------------------------------------------------
// path discovery
// ---------------------------------------------------------------------------

/// Directories that are searched for `language.ini` and the `languages` folder.
///
/// The executable directory comes first, then its parents, so both
/// `installations/<version>/language.ini` and `<ALVR dir>/language.ini` work.
pub fn default_search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = Vec::new();

    if let Ok(extra) = std::env::var("ALVR_LANGUAGE_DIR") {
        if !extra.trim().is_empty() {
            dirs.push(PathBuf::from(extra));
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let mut current = exe.parent().map(Path::to_path_buf);
        for _ in 0..4 {
            let Some(dir) = current else { break };
            if dirs.last() != Some(&dir) {
                dirs.push(dir.clone());
            }
            current = dir.parent().map(Path::to_path_buf);
        }
    }

    dirs
}

fn find_existing(dirs: &[PathBuf], file: &str) -> Option<PathBuf> {
    dirs.iter().map(|d| d.join(file)).find(|p| p.is_file())
}

fn collect_pack_paths(dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    for dir in dirs {
        let languages = dir.join(LANGUAGES_DIR);
        let Ok(entries) = fs::read_dir(&languages) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("ini") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if seen.insert(stem.to_lowercase()) {
                paths.push(path);
            }
        }
    }
    paths
}

// ---------------------------------------------------------------------------
// ini parsing
// ---------------------------------------------------------------------------

fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Parse an INI file into `(section, key) -> value`. Keys outside of any section get
/// the empty section name.
fn parse_ini(text: &str) -> HashMap<(String, String), String> {
    let mut map = HashMap::new();
    let mut section = String::new();

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') || line.starts_with("//")
        {
            continue;
        }
        if let Some(rest) = line.strip_prefix('[') {
            if let Some(end) = rest.find(']') {
                section = rest[..end].trim().to_lowercase();
                continue;
            }
        }
        let Some(split) = line.find('=') else {
            continue;
        };
        let key = unescape(line[..split].trim());
        let value = unescape(line[split + 1..].trim());
        if key.is_empty() {
            continue;
        }
        map.insert((section.clone(), key), value);
    }

    map
}

fn lookup<'a>(
    map: &'a HashMap<(String, String), String>,
    section: &str,
    key: &str,
) -> Option<&'a String> {
    map.get(&(section.to_owned(), key.to_owned()))
}

// ---------------------------------------------------------------------------
// language packs
// ---------------------------------------------------------------------------

fn load_pack(
    path: &Path,
) -> Option<(LanguageInfo, HashMap<String, String>, Vec<(String, String)>)> {
    let text = fs::read_to_string(path).ok()?;
    let map = parse_ini(&text);

    let default_id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_owned();
    // The file name is authoritative so that ids stay stable when a pack is edited.
    let id = default_id;
    if id.is_empty() {
        return None;
    }

    let strings_section = if map.keys().any(|(s, _)| s == "strings") {
        "strings"
    } else {
        ""
    };

    let mut table = HashMap::new();
    let mut wildcards = Vec::new();
    for ((section, key), value) in &map {
        // `[meta]` and other reserved sections hold metadata, everything else is a
        // translation of the English source string.
        let is_meta = matches!(
            section.as_str(),
            "meta" | "language" | "info" | "alvr-language-pack"
        );
        if is_meta || *section != strings_section {
            continue;
        }
        if key.is_empty() || value.is_empty() {
            continue;
        }
        if let Some(prefix) = key.strip_suffix('*') {
            // `Tip: * = ...` style rules, used for runtime-built strings.
            wildcards.push((prefix.to_owned(), value.clone()));
            continue;
        }
        table.insert(key.clone(), value.clone());
    }
    wildcards.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let info = LanguageInfo {
        id,
        native_name: lookup(&map, "meta", "name")
            .or_else(|| lookup(&map, "meta", "native_name"))
            .cloned()
            .unwrap_or_default(),
        english_name: lookup(&map, "meta", "english_name")
            .cloned()
            .unwrap_or_default(),
        author: lookup(&map, "meta", "author").cloned().unwrap_or_default(),
        version: lookup(&map, "meta", "version").cloned().unwrap_or_default(),
        path: path.to_path_buf(),
    };

    Some((info, table, wildcards))
}
/// All languages the user can pick: English plus every pack found on disk.
pub fn available_languages() -> Vec<LanguageInfo> {
    let mut result = vec![LanguageInfo::english()];
    let dirs = match STATE.read() {
        Ok(lock) => match lock.as_ref() {
            Some(inner) => inner.dirs.clone(),
            None => default_search_dirs(),
        },
        Err(_) => return result,
    };

    let mut packs: Vec<LanguageInfo> = collect_pack_paths(&dirs)
        .iter()
        .filter_map(|path| load_pack(path).map(|(info, _, _)| info))
        .collect();
    packs.sort_by(|a, b| a.id.cmp(&b.id));
    result.extend(packs);
    result
}

/// Native display name of a language id, falling back to the id itself.
pub fn language_display_name(id: &str) -> String {
    if id.eq_ignore_ascii_case(ENGLISH) {
        return "English".to_owned();
    }
    available_languages()
        .into_iter()
        .find(|info| info.id.eq_ignore_ascii_case(id))
        .map(|info| {
            if info.native_name.is_empty() {
                info.id
            } else {
                info.native_name
            }
        })
        .unwrap_or_else(|| id.to_owned())
}

// ---------------------------------------------------------------------------
// language.ini
// ---------------------------------------------------------------------------

fn read_selected_language(path: &Path) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let map = parse_ini(&text);
    lookup(&map, "language", "language")
        .or_else(|| lookup(&map, "settings", "language"))
        .or_else(|| lookup(&map, "", "language"))
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn write_selected_language(path: &Path, id: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = format!(
        "; ALVR language configuration / ALVR 语言配置\n\
         ;\n\
         ; Available values / 可选值: en, zh-CN, zh-TW, ja-JP, ko-KR\n\
         ; The in-app setting (Settings tab) writes to this file as well.\n\
         ; 界面里的“语言”选项也会写入这个文件。\n\
         \n\
         [language]\n\
         language = {id}\n"
    );
    fs::write(path, content)
}

fn write_path(dirs: &[PathBuf]) -> PathBuf {
    if let Some(existing) = find_existing(dirs, LANGUAGE_INI) {
        return existing;
    }
    // Prefer a directory that looks like the ALVR installation root so the file does
    // not disappear when the installation folder is replaced by an update.
    for dir in dirs {
        if dir.join("installations").is_dir() {
            return dir.join(LANGUAGE_INI);
        }
    }
    dirs.first()
        .cloned()
        .unwrap_or_default()
        .join(LANGUAGE_INI)
}

// ---------------------------------------------------------------------------
// public api
// ---------------------------------------------------------------------------

fn log_missing_enabled(ini_map: Option<&HashMap<(String, String), String>>) -> bool {
    if let Ok(value) = std::env::var("ALVR_I18N_LOG_MISSING") {
        let value = value.trim().to_lowercase();
        if value == "1" || value == "true" {
            return true;
        }
    }
    if let Some(map) = ini_map {
        if let Some(value) = lookup(map, "debug", "log_missing") {
            let value = value.trim().to_lowercase();
            return value == "1" || value == "true";
        }
    }
    false
}

/// Load the selected language. Safe to call more than once; the first call wins.
pub fn init() {
    init_with_dirs(default_search_dirs());
}

/// Same as [`init`] but with explicit search directories (used by the ALVR apps).
pub fn init_with_dirs(dirs: Vec<PathBuf>) {
    INIT.call_once(|| {
        let ini = find_existing(&dirs, LANGUAGE_INI);
        let ini_map = ini
            .as_deref()
            .and_then(|path| fs::read_to_string(path).ok())
            .map(|text| parse_ini(&text));

        let selected = std::env::var("ALVR_LANGUAGE")
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .or_else(|| {
                ini.as_deref()
                    .and_then(read_selected_language)
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| ENGLISH.to_owned());

        let missing_log = if log_missing_enabled(ini_map.as_ref()) {
            dirs.first().map(|dir| dir.join("untranslated.log"))
        } else {
            None
        };

        let inner = Inner {
            lang: ENGLISH.to_owned(),
            table: HashMap::new(),
            prefixes: Vec::new(),
            dirs: dirs.clone(),
            missing_log,
            missing: Mutex::new(HashSet::new()),
        };

        if let Ok(mut lock) = STATE.write() {
            *lock = Some(inner);
        }

        set_language(&selected);
    });
}

/// Currently selected language id.
pub fn current_language() -> String {
    match STATE.read() {
        Ok(lock) => lock
            .as_ref()
            .map(|inner| inner.lang.clone())
            .unwrap_or_else(|| ENGLISH.to_owned()),
        Err(_) => ENGLISH.to_owned(),
    }
}

/// Switch to another language. Returns `true` if the language is now active.
///
/// The choice is persisted to `language.ini` so it is remembered across restarts.
pub fn set_language(id: &str) -> bool {
    let id = if id.trim().is_empty() { ENGLISH } else { id.trim() };

    let mut lock = match STATE.write() {
        Ok(lock) => lock,
        Err(_) => return false,
    };
    let Some(inner) = lock.as_mut() else {
        return false;
    };

    let mut loaded = None;
    if !id.eq_ignore_ascii_case(ENGLISH) {
        for path in collect_pack_paths(&inner.dirs) {
            let Some((info, table, wildcards)) = load_pack(&path) else {
                continue;
            };
            if info.id.eq_ignore_ascii_case(id) {
                loaded = Some((info.id, table, wildcards));
                break;
            }
        }
        if loaded.is_none() {
            return false;
        }
    }

    match loaded {
        Some((lang, table, prefixes)) => {
            inner.lang = lang;
            inner.table = table;
            inner.prefixes = prefixes;
        }
        None => {
            inner.lang = ENGLISH.to_owned();
            inner.table.clear();
            inner.prefixes.clear();
        }
    }

    let path = write_path(&inner.dirs);
    let _ = write_selected_language(&path, &inner.lang);

    true
}

/// Translate a UI string. Returns `None` when the string should stay as it is.
///
/// This is called for every laid out text, so it must stay cheap: one read lock and
/// one hash lookup, and an early return for English.
pub fn translate(text: &str) -> Option<String> {
    let lock = STATE.read().ok()?;
    let inner = lock.as_ref()?;

    if inner.table.is_empty() {
        return None;
    }

    if let Some(translated) = inner.table.get(text) {
        if !translated.is_empty() {
            return Some(translated.clone());
        }
        return None;
    }

    // Prefix rules: e.g. "Tip: <text>" -> "提示：<translated text>".
    for (prefix, prefix_translation) in &inner.prefixes {
        let Some(rest) = text.strip_prefix(prefix.as_str()) else {
            continue;
        };
        let rest = rest.trim_start();
        let rest = inner
            .table
            .get(rest)
            .cloned()
            .unwrap_or_else(|| rest.to_owned());
        return Some(format!("{prefix_translation}{rest}"));
    }

    if inner.missing_log.is_some() && !text.trim().is_empty() && text.chars().any(|c| c.is_ascii_alphabetic())
    {
        let is_new = inner
            .missing
            .lock()
            .map(|mut set| set.insert(text.to_owned()))
            .unwrap_or(false);
        if is_new {
            if let Some(path) = &inner.missing_log {
                use std::io::Write;
                if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
                    let _ = writeln!(file, "{}", text.replace('\n', "\\n"));
                }
            }
        }
    }

    None
}

/// Convenience wrapper for places that cannot use the layout-time hook.
pub fn tr(text: &str) -> String {
    translate(text).unwrap_or_else(|| text.to_owned())
}

/// Whether a translation table is currently active.
pub fn is_translating() -> bool {
    match STATE.read() {
        Ok(lock) => lock.as_ref().map(|inner| !inner.table.is_empty()).unwrap_or(false),
        Err(_) => false,
    }
}

/// Force a fresh scan of the pack directory (e.g. after the user copied a new pack).
pub fn reload_language_list() {
    // Nothing is cached besides the loaded table, so just re-applying the current
    // language is enough to pick up edited files.
    let current = current_language();
    set_language(&current);
}
