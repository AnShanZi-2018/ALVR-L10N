//! Optional AI translation: the user brings their own OpenAI-compatible endpoint.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct AiConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub translated: usize,
    pub message: String,
    pub finished: bool,
    /// Set on the final message when the run completed.
    pub result: Option<Vec<(String, String)>>,
}

impl Progress {
    fn step(done: usize, total: usize, translated: usize, message: String) -> Self {
        Self {
            done,
            total,
            translated,
            message,
            finished: false,
            result: None,
        }
    }
}

/// Ask the endpoint to translate one batch. Returns (english -> translation).
fn translate_once(
    config: &AiConfig,
    language: &str,
    batch: &[(String, String)],
) -> Result<Vec<(String, String)>, String> {
    let mut payload = String::from("[");
    for (index, (english, _)) in batch.iter().enumerate() {
        if index > 0 {
            payload.push(',');
        }
        payload.push_str(&serde_json::to_string(english).unwrap_or_default());
    }
    payload.push(']');

    let prompt = format!(
        "You are translating the user interface of ALVR, an open-source PC VR streaming app, \
         from English into {language}.\n\
         Input is a JSON array of English strings. Reply with ONLY a JSON object whose keys are \
         exactly those strings and whose values are the translations. No prose, no code fences.\n\
         Rules:\n\
         - keep product names unchanged: ALVR, SteamVR, OpenVR, Quest, Oculus, Virtual Audio Cable, \
         VoiceMeeter, VBCable, NVENC, AMF, HEVC, H.264, AV1, ADB, OpenXR, Discord, GitHub, Open Collective\n\
         - keep placeholders, numbers, units, file names and URLs unchanged\n\
         - keep leading emoji as they are, translate only the words\n\
         - if a string is already in the target language or is a codec/format name, keep it as is\n\
         - use concise UI wording, no extra punctuation at the end\n\n\
         Input: {payload}"
    );

    let body = serde_json::json!({
        "model": config.model,
        "temperature": 0,
        "messages": [{ "role": "user", "content": prompt }],
    });

    let temp: PathBuf =
        std::env::temp_dir().join(format!("alvr-l10n-ai-{}.json", std::process::id()));
    std::fs::write(&temp, body.to_string()).map_err(|e| format!("write payload: {e}"))?;

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let mut command = std::process::Command::new("curl");
    command.args([
        "-sS",
        "--max-time",
        "180",
        "-X",
        "POST",
        &url,
        "-H",
        "Content-Type: application/json",
        "-d",
        &format!("@{}", temp.display()),
    ]);
    if !config.api_key.trim().is_empty() {
        command.args([
            "-H",
            &format!("Authorization: Bearer {}", config.api_key.trim()),
        ]);
    }
    let output = command.output().map_err(|e| format!("curl: {e}"))?;
    let _ = std::fs::remove_file(&temp);
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr)
            .chars()
            .take(300)
            .collect());
    }

    let response: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|_| {
        String::from_utf8_lossy(&output.stdout)
            .chars()
            .take(300)
            .collect::<String>()
    })?;
    let content = response
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "unexpected response".to_owned())?;

    let start = content
        .find('{')
        .ok_or_else(|| "no JSON in reply".to_owned())?;
    let end = content
        .rfind('}')
        .ok_or_else(|| "no JSON in reply".to_owned())?;
    let map: serde_json::Value =
        serde_json::from_str(&content[start..=end]).map_err(|e| format!("bad JSON: {e}"))?;

    let mut translated = Vec::new();
    for (english, _) in batch {
        if let Some(value) = map.get(english).and_then(|v| v.as_str()) {
            let value = value.trim();
            if !value.is_empty() {
                translated.push((english.clone(), value.to_owned()));
            }
        }
    }
    if translated.is_empty() {
        return Err("reply did not match the requested strings".to_owned());
    }
    Ok(translated)
}

/// Translate everything in `entries`, in batches, reporting progress.
pub fn translate_all(
    config: AiConfig,
    language: String,
    entries: Vec<(String, String)>,
    cancel: Arc<AtomicBool>,
    sender: Sender<Progress>,
) {
    let total = entries.len();
    let mut out: Vec<(String, String)> = Vec::new();
    let batch_size = 25;

    let mut index = 0;
    while index < entries.len() {
        if cancel.load(Ordering::Relaxed) {
            let _ = sender.send(Progress {
                done: index,
                total,
                translated: out.len(),
                message: "cancelled".to_owned(),
                finished: true,
                result: Some(out),
            });
            return;
        }
        let end = (index + batch_size).min(entries.len());
        let batch = &entries[index..end];

        let mut attempt = 0;
        let mut result = Err("not tried".to_owned());
        while attempt < 3 {
            attempt += 1;
            result = translate_once(&config, &language, batch);
            if result.is_ok() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
        match result {
            Ok(pairs) => out.extend(pairs),
            Err(error) => {
                let _ = sender.send(Progress::step(
                    index,
                    total,
                    out.len(),
                    format!("batch {index}-{end} failed: {error}"),
                ));
            }
        }
        index = end;
        let _ = sender.send(Progress::step(index, total, out.len(), String::new()));
    }

    let _ = sender.send(Progress {
        done: total,
        total,
        translated: out.len(),
        message: String::new(),
        finished: true,
        result: Some(out),
    });
}

/// Well-known OpenAI-compatible endpoints.
pub struct Provider {
    pub name: &'static str,
    pub base: &'static str,
    pub models: &'static [&'static str],
    pub key_url: &'static str,
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        name: "OpenAI",
        base: "https://api.openai.com/v1",
        models: &["gpt-4o-mini", "gpt-4o", "gpt-4.1-mini"],
        key_url: "https://platform.openai.com/api-keys",
    },
    Provider {
        name: "DeepSeek",
        base: "https://api.deepseek.com/v1",
        models: &["deepseek-chat"],
        key_url: "https://platform.deepseek.com/api_keys",
    },
    Provider {
        name: "Moonshot",
        base: "https://api.moonshot.cn/v1",
        models: &["moonshot-v1-8k"],
        key_url: "https://platform.moonshot.cn/console/api-keys",
    },
    Provider {
        name: "智谱 GLM",
        base: "https://open.bigmodel.cn/api/paas/v4",
        models: &["glm-4-flash", "glm-4-air"],
        key_url: "https://open.bigmodel.cn/usercenter/apikeys",
    },
    Provider {
        name: "SiliconFlow",
        base: "https://api.siliconflow.cn/v1",
        models: &["Qwen/Qwen2.5-7B-Instruct", "THUDM/glm-4-9b-chat"],
        key_url: "https://cloud.siliconflow.cn/account/ak",
    },
    Provider {
        name: "Ollama (local)",
        base: "http://127.0.0.1:11434/v1",
        models: &["qwen2.5:7b", "llama3.1:8b"],
        key_url: "",
    },
    Provider {
        name: "Custom",
        base: "",
        models: &[],
        key_url: "",
    },
];

/// Languages offered in the dropdown: (native name, pack id).
pub const LANGUAGES: &[(&str, &str)] = &[
    ("简体中文", "zh-CN"),
    ("繁體中文", "zh-TW"),
    ("English", "en"),
    ("日本語", "ja-JP"),
    ("한국어", "ko-KR"),
    ("Русский", "ru-RU"),
    ("Français", "fr-FR"),
    ("Deutsch", "de-DE"),
    ("Español", "es-ES"),
    ("Português (Brasil)", "pt-BR"),
    ("Italiano", "it-IT"),
    ("Polski", "pl-PL"),
    ("Türkçe", "tr-TR"),
    ("Українська", "uk-UA"),
    ("हिन्दी", "hi-IN"),
    ("বাংলা", "bn-BD"),
    ("العربية", "ar-SA"),
    ("فارسی", "fa-IR"),
    ("ไทย", "th-TH"),
    ("Tiếng Việt", "vi-VN"),
    ("Bahasa Indonesia", "id-ID"),
    ("Nederlands", "nl-NL"),
    ("Svenska", "sv-SE"),
    ("Čeština", "cs-CZ"),
    ("Magyar", "hu-HU"),
    ("Română", "ro-RO"),
    ("Ελληνικά", "el-GR"),
    ("עברית", "he-IL"),
    ("Монгол", "mn-MN"),
];

/// Translate a single string; used by the connection test.
pub fn test_connection(config: &AiConfig) -> Result<String, String> {
    let batch = vec![("Hello".to_owned(), String::new())];
    translate_once(config, "Test", &batch).map(|pairs| {
        pairs
            .first()
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    })
}
