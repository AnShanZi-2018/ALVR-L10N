//! The tool's own UI language (independent from the ALVR language packs).

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum UiLang {
    En,
    ZhCn,
    ZhTw,
    JaJp,
    KoKr,
}

pub const UI_LANGS: [UiLang; 5] = [
    UiLang::En,
    UiLang::ZhCn,
    UiLang::ZhTw,
    UiLang::JaJp,
    UiLang::KoKr,
];

impl UiLang {
    pub fn index(self) -> usize {
        UI_LANGS.iter().position(|l| *l == self).unwrap_or(0)
    }

    pub fn id(self) -> &'static str {
        match self {
            UiLang::En => "en",
            UiLang::ZhCn => "zh-CN",
            UiLang::ZhTw => "zh-TW",
            UiLang::JaJp => "ja-JP",
            UiLang::KoKr => "ko-KR",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            UiLang::En => "English",
            UiLang::ZhCn => "简体中文",
            UiLang::ZhTw => "繁體中文",
            UiLang::JaJp => "日本語",
            UiLang::KoKr => "한국어",
        }
    }

    pub fn from_id(id: &str) -> UiLang {
        UI_LANGS
            .iter()
            .copied()
            .find(|l| l.id().eq_ignore_ascii_case(id))
            .unwrap_or(UiLang::En)
    }

    /// Translate a UI string. Order of the array is the order of `UI_LANGS`.
    pub fn t(self, key: &str) -> &'static str {
        let row = ROWS.iter().find(|(k, _)| *k == key);
        match row {
            Some((_, values)) => values[self.index()],
            None => "",
        }
    }
}

/// key, [English, 简体中文, 繁體中文, 日本語, 한국어]
const ROWS: &[(&str, [&str; 5])] = &[
    ("title", ["ALVR Localization Tool", "ALVR 汉化工具", "ALVR 漢化工具", "ALVR 日本語化ツール", "ALVR 한글화 도구"]),
    ("redetect", ["Re-scan", "重新检测", "重新偵測", "再スキャン", "다시 검색"]),
    ("ui_language", ["Interface language", "界面语言", "介面語言", "表示言語", "인터페이스 언어"]),
    ("alvr_path", ["ALVR installation", "ALVR 位置", "ALVR 位置", "ALVR の場所", "ALVR 위치"]),
    ("version", ["version", "版本", "版本", "バージョン", "버전"]),
    ("not_detected", ["ALVR not found. Install ALVR first, or put this tool inside the ALVR folder.", "未检测到 ALVR：请先安装 ALVR，或把本工具放进 ALVR 目录。", "未偵測到 ALVR：請先安裝 ALVR，或把本工具放進 ALVR 資料夾。", "ALVR が見つかりません。ALVR をインストールするか、このツールを ALVR フォルダー内に置いてください。", "ALVR를 찾을 수 없습니다. ALVR을 설치하거나 이 도구를 ALVR 폴더에 넣어 주세요."]),
    ("packs", ["Language packs", "语言包", "語言包", "言語パック", "언어 팩"]),
    ("packs_hint", ["Check the packs to install into ALVR. Unchecked ones stay out of ALVR's language menu.", "勾选要装进 ALVR 的语言包；不勾的不会出现在 ALVR 的语言菜单里。", "勾選要裝進 ALVR 的語言包；未勾選的不會出現在 ALVR 的語言選單裡。", "ALVR に入れる言語パックにチェックを入れてください。チェックしないものは ALVR の言語メニューに表示されません。", "ALVR에 설치할 언어 팩을 선택하세요. 선택하지 않은 팩은 ALVR 언어 메뉴에 나타나지 않습니다."]),
    ("no_packs", ["No language pack found in the languages folder.", "languages 文件夹里没有语言包。", "languages 資料夾裡沒有語言包。", "languages フォルダーに言語パックがありません。", "languages 폴더에 언어 팩이 없습니다."]),
    ("add_pack", ["Add language pack…", "扩展语言包…", "擴充語言包…", "言語パックを追加…", "언어 팩 추가…"]),
    ("open_pack_folder", ["Open pack folder", "打开语言包文件夹", "開啟語言包資料夾", "パックフォルダーを開く", "팩 폴더 열기"]),
    ("apply", ["Apply", "应用", "套用", "適用", "적용"]),
    ("restore", ["Restore English", "还原成英文", "還原成英文", "英語に戻す", "영어로 복원"]),
    ("backup", ["Back up settings", "备份我的设置", "備份我的設定", "設定をバックアップ", "설정 백업"]),
    ("open_alvr", ["Open ALVR folder", "打开 ALVR 目录", "開啟 ALVR 資料夾", "ALVR フォルダーを開く", "ALVR 폴더 열기"]),
    ("status_ready", ["Ready. Select packs and press Apply.", "就绪：勾选语言包后点「应用」。", "就緒：勾選語言包後按「套用」。", "準備完了。パックを選んで「適用」を押してください。", "준비 완료. 팩을 선택하고 적용을 누르세요."]),
    ("status_applied_full", ["Done — full 100% localization installed.", "完成：已安装完整汉化（100%）。", "完成：已安裝完整漢化（100%）。", "完了：完全な日本語化（100%）をインストールしました。", "완료: 완전한 한글화(100%)를 설치했습니다."]),
    ("status_applied_patch", ["Done — compatibility mode applied (partial coverage).", "完成：兼容模式已应用（部分覆盖）。", "完成：相容模式已套用（部分覆蓋）。", "完了：互換モードを適用しました（一部のみ）。", "완료: 호환 모드 적용(부분 번역)."]),
    ("status_restored", ["Restored to the official English build.", "已还原为官方英文版。", "已還原為官方英文版。", "公式の英語版に戻しました。", "공식 영어 버전으로 복원했습니다."]),
    ("status_backup", ["Settings backed up.", "设置已备份。", "設定已備份。", "設定をバックアップしました。", "설정을 백업했습니다."]),
    ("status_no_alvr", ["Cannot continue: ALVR not found.", "无法继续：没有找到 ALVR。", "無法繼續：找不到 ALVR。", "続行できません：ALVR が見つかりません。", "계속할 수 없습니다: ALVR을 찾을 수 없습니다."]),
    ("status_pick_pack", ["Please check at least one language pack.", "请至少勾选一个语言包。", "請至少勾選一個語言包。", "言語パックを 1 つ以上選んでください。", "언어 팩을 하나 이상 선택하세요."]),
    ("settings_note", ["Your session.json is backed up before any change.", "改动前会自动备份 session.json。", "改動前會自動備份 session.json。", "変更前に session.json を自動バックアップします。", "변경 전에 session.json을 자동 백업합니다."]),
    ("saved_to", ["Saved to", "已保存到", "已儲存到", "保存先", "저장 위치"]),
    ("full_build_ok", ["A 100% build for this ALVR version is bundled.", "已内置该版本的完整汉化包（100%）。", "已內建此版本的完整漢化包（100%）。", "このバージョン用の完全版が同梱されています。", "이 버전의 완전판이 포함되어 있습니다."]),
    ("compat_only", ["No full build for this version — compatibility mode will be used.", "该版本没有完整版，将使用兼容模式。", "此版本沒有完整版，將使用相容模式。", "このバージョンの完全版がありません。互換モードを使用します。", "이 버전의 완전판이 없습니다. 호환 모드를 사용합니다."]),
    ("log_title", ["Details", "详细日志", "詳細日誌", "詳細", "상세 로그"]),
    ("about_packs", ["Only the checked packs are copied into ALVR, so ALVR's own language menu lists exactly those.", "只有勾选的语言包会被复制进 ALVR，ALVR 的语言菜单里就只会出现这些。", "只有勾選的語言包會被複製進 ALVR，ALVR 的語言選單就只會出現這些。", "チェックした言語パックだけが ALVR にコピーされ、ALVR の言語メニューにはそれだけが表示されます。", "체크한 언어 팩만 ALVR에 복사되어 ALVR 언어 메뉴에 그것만 표시됩니다."]),
    ("pick_file_title", ["Select a language pack (.ini)", "选择语言包（.ini）", "選擇語言包（.ini）", "言語パック（.ini）を選択", "언어 팩(.ini) 선택"]),
    ("pack_added", ["Language pack added:", "已添加语言包：", "已新增語言包：", "言語パックを追加しました：", "언어 팩을 추가했습니다:"]),
    ("import_config", ["Import config file…", "导入配置文件…", "匯入設定檔…", "設定ファイルを読み込む…", "설정 파일 가져오기…"]),
    ("import_title", ["Select an ALVR session.json", "选择 ALVR 的 session.json", "選擇 ALVR 的 session.json", "ALVR の session.json を選択", "ALVR session.json 선택"]),
    ("import_ok", ["Configuration imported.", "配置已导入。", "設定已匯入。", "設定を読み込みました。", "설정을 가져왔습니다."]),
    ("import_bad", ["That is not an ALVR session.json.", "这不是 ALVR 的 session.json。", "這不是 ALVR 的 session.json。", "ALVR の session.json ではありません。", "ALVR session.json이 아닙니다."]),
    ("import_version", ["Config was made for ALVR", "这份配置对应的 ALVR 版本是", "這份設定的 ALVR 版本是", "この設定の ALVR バージョン:", "이 설정의 ALVR 버전:"]),
    ("pack_add_failed", ["Could not add the language pack.", "添加语言包失败。", "新增語言包失敗。", "言語パックを追加できませんでした。", "언어 팩을 추가하지 못했습니다."]),
];
