//! Turn the detected hardware into concrete ALVR settings.

use crate::sysinfo::Hardware;

/// Common headsets and their native per-eye resolution.
pub const HEADSETS: &[(&str, u32, u32)] = &[
    ("Quest 3 / 3S", 2064, 2208),
    ("Quest 2 / Pro", 1832, 1920),
    ("Pico 4", 2160, 2160),
    ("Pico Neo 3", 1832, 1920),
    ("Vive Focus 3", 2448, 2448),
    ("Valve Index", 1440, 1600),
    ("Reverb G2", 2160, 2160),
];

#[derive(Clone, Debug)]
pub struct Recommendation {
    pub tier_key: &'static str,
    pub codec: &'static str,
    pub bitrate_mbps: u32,
    pub eye_width: u32,
    pub eye_height: u32,
    pub refresh_hz: u32,
    pub foveated: bool,
    pub notes: Vec<String>,
}

fn score_gpu(name: &str, vram_mb: u64) -> u32 {
    let lower = name.to_ascii_lowercase();
    if lower.contains("intel")
        && (lower.contains("uhd") || lower.contains("iris") || lower.contains("hd graphics"))
    {
        return 10;
    }
    if lower.contains("rtx 50") || lower.contains("rtx 40 90") || lower.contains("rtx 4090") {
        return 100;
    }
    if lower.contains("rtx 4080")
        || lower.contains("rtx 4070")
        || lower.contains("rtx 3090")
        || lower.contains("rtx 3080")
    {
        return 90;
    }
    if lower.contains("rtx 4060")
        || lower.contains("rtx 3070")
        || lower.contains("rtx 3060")
        || lower.contains("rx 7")
    {
        return 70;
    }
    if lower.contains("rtx 20")
        || lower.contains("gtx 16")
        || lower.contains("rx 6")
        || lower.contains("arc a")
    {
        return 55;
    }
    if lower.contains("gtx 10") || lower.contains("rx 5") {
        return 35;
    }
    if vram_mb >= 12 * 1024 {
        return 85;
    }
    if vram_mb >= 8 * 1024 {
        return 60;
    }
    if vram_mb >= 4 * 1024 {
        return 40;
    }
    20
}

pub fn recommend(hardware: &Hardware, wired: bool, native: Option<(u32, u32)>) -> Recommendation {
    let best = hardware
        .gpus
        .iter()
        .filter(|gpu| {
            let lower = gpu.name.to_ascii_lowercase();
            !lower.contains("virtual")
                && !lower.contains("basic display")
                && !lower.contains("remote")
        })
        .max_by_key(|gpu| score_gpu(&gpu.name, gpu.vram_mb))
        .map(|gpu| {
            (
                gpu.name.clone(),
                gpu.vram_mb,
                score_gpu(&gpu.name, gpu.vram_mb),
            )
        })
        .unwrap_or_else(|| ("unknown".to_owned(), 0, 10));

    let score = best.2;
    let vram_gb = best.1 / 1024;
    let mut notes = Vec::new();

    if score <= 10 {
        notes.push("检测到集成显卡：ALVR 串流需要独立显卡，建议先换台式独显再试。".to_owned());
    }
    if hardware.ram_gb > 0 && hardware.ram_gb < 16 {
        notes.push(format!(
            "内存 {} GB，建议 16 GB 以上（VR 游戏 + 串流同时占用较大）。",
            hardware.ram_gb
        ));
    }
    if hardware.disk_free_gb > 0 && hardware.disk_free_gb < 20 {
        notes.push(format!(
            "ALVR 所在磁盘只剩 {} GB，录制/缓存可能失败。",
            hardware.disk_free_gb
        ));
    }
    if !wired && score >= 70 {
        notes
            .push("无线模式建议 5GHz / WiFi 6 且头显与路由器同一房间；有条件优先有线。".to_owned());
    }

    let (tier, codec, bitrate_wired, bitrate_wifi, width, refresh) = if score >= 85 {
        (
            "tier_high",
            "Hevc",
            450,
            300,
            (3072u32, 3216u32),
            if wired { 120 } else { 90 },
        )
    } else if score >= 60 {
        ("tier_mid", "Hevc", 300, 200, (2448, 2560), 90)
    } else if score >= 35 {
        ("tier_entry", "H264", 180, 120, (2064, 2208), 72)
    } else {
        ("tier_low", "H264", 120, 90, (1832, 1920), 72)
    };

    let (mut eye_w, mut eye_h) = (width.0, width.1);
    if let Some((native_w, native_h)) = native {
        let scale = if score >= 85 {
            1.4f32
        } else if score >= 60 {
            1.15
        } else {
            1.0
        };
        let round8 = |value: f32| ((value as u32) + 7) / 8 * 8;
        eye_w = round8(native_w as f32 * scale);
        eye_h = round8(native_h as f32 * scale);
        notes.push(format!(
            "按头显原生 {}x{} 与显卡档位计算（x{scale:.2}）。",
            native_w, native_h
        ));
    }

    if vram_gb > 0 && vram_gb < 6 {
        notes.push(format!(
            "显存仅 {} GB，建议把分辨率再降一档或关闭注视点编码。",
            vram_gb
        ));
    }

    Recommendation {
        tier_key: tier,
        codec,
        bitrate_mbps: if wired { bitrate_wired } else { bitrate_wifi },
        eye_width: eye_w,
        eye_height: eye_h,
        refresh_hz: refresh,
        foveated: score >= 55,
        notes,
    }
}
