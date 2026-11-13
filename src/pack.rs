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

/// All English source strings of the reference pack, in file order.
pub fn english_keys(reference: &std::path::Path) -> Vec<String> {
    let mut keys = Vec::new();
    if let Ok(text) = std::fs::read_to_string(reference) {
        let mut in_strings = false;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line
                .strip_prefix('[')
                .and_then(|rest| rest.split(']').next())
            {
                in_strings = name.trim().eq_ignore_ascii_case("strings");
                continue;
            }
            if in_strings {
                if let Some((key, _)) = line.split_once('=') {
                    keys.push(unescape(key.trim()));
                }
            }
        }
    }
    keys
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Render a language pack from (english, translation) pairs.
pub fn pack_from_pairs(id: &str, name: &str, pairs: &[(String, String)]) -> String {
    let mut text = String::new();
    text.push_str(&format!("; generated by ALVR-L10N for {id}\n\n[meta]\n"));
    text.push_str(&format!(
        "name = {name}\nenglish_name = {name}\nauthor = \nversion = 1.0.0\n\n[strings]\n"
    ));
    for (english, translation) in pairs {
        if translation.trim().is_empty() {
            continue;
        }
        text.push_str(&format!("{} = {}\n", escape(english), escape(translation)));
    }
    text
}

/// Parse a two-column CSV (or JSON object) into translation pairs.
pub fn pairs_from_csv(text: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for (index, raw) in text.lines().enumerate() {
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }
        let fields = split_csv_line(line);
        if fields.len() < 2 {
            continue;
        }
        let (english, translation) = (fields[0].clone(), fields[1].clone());
        if index == 0 && english.to_ascii_lowercase().contains("english") {
            continue; // header row
        }
        pairs.push((english, translation));
    }
    pairs
}

fn split_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => fields.push(std::mem::take(&mut current)),
            _ => current.push(c),
        }
    }
    fields.push(current);
    fields.into_iter().map(|f| f.trim().to_owned()).collect()
}

pub fn pairs_from_json(text: &str) -> Vec<(String, String)> {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(serde_json::Value::Object(map)) => map
            .into_iter()
            .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_owned())))
            .collect(),
        _ => Vec::new(),
    }
}

/// Read `[strings]` of an existing pack as (english, translation) pairs.
pub fn read_strings(path: &std::path::Path) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    if let Ok(text) = std::fs::read_to_string(path) {
        let mut in_strings = false;
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
                continue;
            }
            if let Some(name) = line
                .strip_prefix('[')
                .and_then(|rest| rest.split(']').next())
            {
                in_strings = name.trim().eq_ignore_ascii_case("strings");
                continue;
            }
            if in_strings {
                if let Some((key, value)) = line.split_once('=') {
                    pairs.push((unescape(key.trim()), unescape(value.trim())));
                }
            }
        }
    }
    pairs
}
