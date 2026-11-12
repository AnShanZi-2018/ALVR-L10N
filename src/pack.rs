//! Language pack (.ini) loading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LanguagePack {
    pub id: String,
    pub native_name: String,
    pub english_name: String,
    pub version: String,
    pub author: String,
    pub ini_path: PathBuf,
    /// English source string -> translation. Keys ending with `*` are prefix rules.
    pub strings: HashMap<String, String>,
    pub prefixes: Vec<(String, String)>,
    /// Optional per-language font used by patch mode.
    pub font_path: Option<PathBuf>,
}

pub fn unescape(value: &str) -> String {
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
            Some('r') => out.push('\r'),
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

fn parse_ini(text: &str) -> HashMap<(String, String), String> {
    let mut map = HashMap::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
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

pub fn load_pack(ini_path: &Path) -> Option<LanguagePack> {
    let text = std::fs::read_to_string(ini_path).ok()?;
    let map = parse_ini(&text);
    let id = ini_path.file_stem()?.to_str()?.to_owned();

    let mut strings = HashMap::new();
    let mut prefixes = Vec::new();
    for ((section, key), value) in &map {
        if section != "strings" || key.is_empty() || value.is_empty() {
            continue;
        }
        match key.strip_suffix('*') {
            Some(prefix) => prefixes.push((prefix.to_owned(), value.clone())),
            None => {
                strings.insert(key.clone(), value.clone());
            }
        }
    }
    prefixes.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

    let font_path = ini_path.with_extension("ttf");
    Some(LanguagePack {
        id,
        native_name: map
            .get(&("meta".to_owned(), "name".to_owned()))
            .cloned()
            .unwrap_or_default(),
        english_name: map
            .get(&("meta".to_owned(), "english_name".to_owned()))
            .cloned()
            .unwrap_or_default(),
        version: map
            .get(&("meta".to_owned(), "version".to_owned()))
            .cloned()
            .unwrap_or_default(),
        author: map
            .get(&("meta".to_owned(), "author".to_owned()))
            .cloned()
            .unwrap_or_default(),
        font_path: font_path.is_file().then_some(font_path),
        ini_path: ini_path.to_path_buf(),
        strings,
        prefixes,
    })
}

/// All packs found in `dir` (`*.ini`), sorted by id.
pub fn load_dir(dir: &Path) -> Vec<LanguagePack> {
    let mut packs = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return packs;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ini") {
            if let Some(pack) = load_pack(&path) {
                packs.push(pack);
            }
        }
    }
    packs.sort_by(|a, b| a.id.cmp(&b.id));
    packs
}

impl LanguagePack {
    /// Native name only, e.g. "简体中文" (falls back to the id).
    pub fn label(&self) -> String {
        if self.native_name.is_empty() {
            self.id.clone()
        } else {
            self.native_name.clone()
        }
    }

    pub fn display_name(&self) -> String {
        if self.native_name.is_empty() {
            self.id.clone()
        } else {
            format!("{} ({})", self.native_name, self.id)
        }
    }

    /// Translation for an exact English string (used by prefix rules too).
    pub fn lookup(&self, text: &str) -> Option<String> {
        if let Some(value) = self.strings.get(text) {
            return Some(value.clone());
        }
        for (prefix, translation) in &self.prefixes {
            if let Some(rest) = text.strip_prefix(prefix.as_str()) {
                let rest = rest.trim_start();
                let rest = self
                    .strings
                    .get(rest)
                    .cloned()
                    .unwrap_or_else(|| rest.to_owned());
                return Some(format!("{translation}{rest}"));
            }
        }
        None
    }
}
