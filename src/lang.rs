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
    ("nav_localize", ["Localize", "汉化", "漢化", "日本語化", "한글화"]),
    ("nav_about", ["About", "关于", "關於", "このアプリについて", "정보"]),
    ("nav_tool", ["Tool", "工具", "工具", "ツール", "도구"]),
    ("page_localize", ["Localize ALVR", "汉化 ALVR", "漢化 ALVR", "ALVR をローカライズ", "ALVR 한글화"]),
    ("pack_selected", ["Selected", "已选择", "已選擇", "選択済み", "선택됨"]),
    ("pack_unselected", ["Not selected", "未选择", "未選擇", "未選択", "선택 안 됨"]),
    ("theme_light", ["Light theme", "浅色主题", "淺色主題", "ライトテーマ", "라이트 테마"]),
    ("theme_dark", ["Dark theme", "深色主题", "深色主題", "ダークテーマ", "다크 테마"]),
    ("share_copied", ["Link copied to clipboard", "链接已复制到剪贴板", "連結已複製到剪貼簿", "リンクをコピーしました", "링크를 클립보드에 복사했습니다"]),
    ("about_tagline", ["Standalone localization tool for ALVR", "ALVR 的独立多语言工具", "ALVR 的獨立多語言工具", "ALVR の独立ローカライズツール", "ALVR 독립 다국어 도구"]),
    ("about_github", ["View this project on GitHub", "在 GitHub 上查看本项目", "在 GitHub 上檢視本專案", "GitHub でプロジェクトを見る", "GitHub에서 프로젝트 보기"]),
    ("about_feedback", ["Send feedback", "反馈意见", "回饋意見", "フィードバックを送る", "피드백 보내기"]),
    ("about_share", ["Share this tool (copy link)", "分享此软件（复制链接）", "分享此軟體（複製連結）", "このツールを共有（リンクをコピー）", "이 도구 공유(링크 복사)"]),
    ("about_license", ["License", "许可证", "許可證", "ライセンス", "라이선스"]),
    ("about_third_party", ["ALVR is MIT licensed; bundled font subsets are SIL OFL 1.1. See THIRD_PARTY_LICENSES.md in the repository.", "ALVR 为 MIT 许可；内置字体子集为 SIL OFL 1.1。详见仓库里的 THIRD_PARTY_LICENSES.md。", "ALVR 為 MIT 授權；內建字型子集為 SIL OFL 1.1。詳見儲存庫的 THIRD_PARTY_LICENSES.md。", "ALVR は MIT ライセンス、同梱フォントは SIL OFL 1.1 です。詳細はリポジトリの THIRD_PARTY_LICENSES.md を参照してください。", "ALVR은 MIT 라이선스이며 포함된 글꼴은 SIL OFL 1.1입니다. 자세한 내용은 저장소의 THIRD_PARTY_LICENSES.md를 참고하세요."]),
    ("about_changelog", ["Release notes", "更新日志", "更新日誌", "リリースノート", "릴리스 노트"]),
    ("nav_help", ["Help", "帮助", "說明", "ヘルプ", "도움말"]),
    ("about_email", ["anshanzi2018@gmail.com", "anshanzi2018@gmail.com", "anshanzi2018@gmail.com", "anshanzi2018@gmail.com", "anshanzi2018@gmail.com"]),
    ("feedback_copied", ["E-mail address copied:", "邮箱已复制：", "電子郵件已複製：", "メールアドレスをコピーしました：", "이메일 주소를 복사했습니다:"]),
    ("tip_add_pack", ["Pick a language pack (.ini). A font with the same name (.ttf) next to it is taken along.", "选择一个语言包（.ini）。同名 .ttf 字体会一起导入。", "選擇一個語言包（.ini）。同名 .ttf 字型會一併匯入。", "言語パック（.ini）を選択します。同じ名前の .ttf も一緒に追加されます。", "언어 팩(.ini)을 선택하세요. 같은 이름의 .ttf도 함께 추가됩니다."]),
    ("tip_open_pack_folder", ["Put your own .ini (and optional .ttf) here, then press Re-scan.", "把自己的 .ini（可选 .ttf）拖进这里，然后点「重新检测」。", "把自己的 .ini（可選 .ttf）拖進來，然後按「重新偵測」。", "自作の .ini（任意で .ttf）をここに置き、再スキャンを押してください。", "직접 만든 .ini(선택 .ttf)를 여기에 넣고 다시 검색을 누르세요."]),
    ("help_intro", ["How this tool works, in short.", "这个工具怎么用，一句话版本。", "這個工具怎麼用，簡短說明。", "このツールの使い方（要点）。", "이 도구의 사용법 요약."]),
    ("help_lang_title", ["Where does the language come from?", "语言是怎么来的？", "語言是怎麼來的？", "言語はどこから来る？", "언어는 어디서 오나요?"]),
    ("help_lang_body", ["ALVR itself only has an English interface. The tool ships language packs (plain .ini files in the languages folder) and installs the ones you tick into ALVR\\languages. The localized ALVR build then reads them at start-up and lists them in its own language menu, so you can switch there at any time.", "ALVR 本身只有英文界面。本工具自带语言包（languages 文件夹里的纯文本 .ini），把你勾选的那些装进 ALVR\\languages，汉化版 ALVR 启动时读取它们，并在自己的语言菜单里列出来，之后随时可以在 ALVR 里切换。", "ALVR 本身只有英文介面。本工具自帶語言包（languages 資料夾裡的純文字 .ini），把你勾選的裝進 ALVR\\languages，中文化版 ALVR 啟動時讀取它們，並在自己的語言選單裡列出，之後隨時可以在 ALVR 裡切換。", "ALVR 本体は英語のみです。このツールは言語パック（languages フォルダーの .ini）を同梱し、チェックしたものを ALVR\\languages に配置します。ローカライズ版 ALVR が起動時に読み込み、ALVR 側の言語メニューに表示されます。", "ALVR 자체는 영어만 지원합니다. 이 도구는 언어 팩(languages 폴더의 .ini)을 포함하며, 선택한 팩을 ALVR\\languages에 설치합니다. 한글화된 ALVR이 시작할 때 읽어 자체 언어 메뉴에 표시합니다."]),
    ("help_pack_title", ["Add a language pack", "扩展语言包", "擴充語言包", "言語パックを追加", "언어 팩 추가"]),
    ("help_pack_body", ["Press \"Add language pack…\" and pick an .ini file. Alternatively open the pack folder, drop the file there and press Re-scan. The pack appears in the list immediately and only needs to be ticked before Apply.", "点「扩展语言包…」选一个 .ini 即可。也可以点「打开语言包文件夹」把文件放进去，再点「重新检测」。新语言包会立刻出现在列表里，勾上再点应用就行。", "按「擴充語言包…」選一個 .ini 即可。也可以按「開啟語言包資料夾」把檔案放進去，再按「重新偵測」。新語言包會立刻出現在清單裡，勾選後再套用。", "「言語パックを追加…」から .ini を選びます。パックフォルダーに置いて再スキャンしても構いません。一覧に追加されるので、チェックして適用します。", "언어 팩 추가…에서 .ini를 선택하세요. 팩 폴더에 넣고 다시 검색해도 됩니다. 목록에 나타나면 체크하고 적용하세요."]),
    ("help_font_title", ["Fonts (CJK / Thai / Arabic …)", "字体（中日韩 / 泰文 / 阿拉伯文…）", "字型（中日韓 / 泰文 / 阿拉伯文…）", "フォント（CJK / タイ語 / アラビア語…）", "글꼴(중일한 / 태국어 / 아랍어…)"]),
    ("help_font_body", ["Latin, Cyrillic and Greek already work with ALVR's own font. For CJK, Thai, Arabic and similar scripts a font must be supplied: put a .ttf next to the .ini with the same base name (for example th-TH.ini + th-TH.ttf). Compatibility mode injects it into ALVR's font slot, so it has to stay under roughly 350 KB — subset it first.", "拉丁、西里尔、希腊字母用 ALVR 自带字体就够了。中日韩、泰文、阿拉伯文这类需要额外提供字体：把 .ttf 放在 .ini 旁边、文件名相同（例如 th-TH.ini + th-TH.ttf）。兼容模式会把它注入 ALVR 的字体槽，所以体积要控制在约 350 KB 以内——先用工具做子集化。", "拉丁、西里爾、希臘字母用 ALVR 自帶字型就夠了。中日韓、泰文、阿拉伯文這類需要額外提供字型：把 .ttf 放在 .ini 旁邊、檔名相同（例如 th-TH.ini + th-TH.ttf）。相容模式會把它注入 ALVR 的字型槽，所以體積要控制在約 350 KB 以內——先用工具做子集化。", "ラテン語・キリル文字・ギリシャ文字は ALVR 内蔵フォントで表示できます。CJK・タイ語・アラビア語などはフォントが必要です。.ini と同じ名前の .ttf（例: th-TH.ini + th-TH.ttf）を置いてください。互換モードでは ALVR のフォント領域に埋め込むため、約 350 KB 以下にサブセット化してください。", "라틴·키릴·그리스 문자는 ALVR 기본 글꼴로 충분합니다. 중일한·태국어·아랍어 등은 글꼴이 필요합니다. .ini와 같은 이름의 .ttf(예: th-TH.ini + th-TH.ttf)를 함께 두세요. 호환 모드는 ALVR 글꼴 영역에 주입하므로 약 350KB 이하로 서브셋해야 합니다."]),
    ("help_where_title", ["What gets installed where", "装到哪儿去了", "裝到哪兒去了", "どこにインストールされる？", "어디에 설치되나요?"]),
    ("help_where_body", ["Only the ticked packs are copied to <ALVR>\\languages; the default language goes to <ALVR>\\language.ini. Everything else stays inside your ALVR folder — no registry keys, no system changes. Tick fewer packs if you want a shorter language menu inside ALVR.", "只有勾选的语言包会被复制到 <ALVR>\\languages，默认语言写进 <ALVR>\\language.ini。其它东西都留在 ALVR 文件夹里——不写注册表、不改系统。想让 ALVR 的语言菜单更短，就少勾几个。", "只有勾選的語言包會被複製到 <ALVR>\\languages，預設語言寫進 <ALVR>\\language.ini。其他東西都留在 ALVR 資料夾裡——不寫登錄檔、不改系統。想讓 ALVR 的語言選單更短，就少勾幾個。", "チェックしたパックだけが <ALVR>\\languages にコピーされ、既定の言語は <ALVR>\\language.ini に書かれます。それ以外は ALVR フォルダー内で完結し、レジストリやシステムは変更しません。", "체크한 팩만 <ALVR>\\languages로 복사되고 기본 언어는 <ALVR>\\language.ini에 기록됩니다. 그 외에는 ALVR 폴더 안에서만 동작하며 레지스트리나 시스템을 변경하지 않습니다."]),
    ("help_safety_title", ["Backups and rollback", "备份与还原", "備份與還原", "バックアップと復元", "백업 및 복원"]),
    ("help_safety_body", ["Before anything is written, session.json is copied to <ALVR>\\配置备份 and the original executables to .alvr-l10n. If a patched build fails to start, the tool restores the official one automatically. \"Restore English\" brings back the official build at any time.", "写入任何东西之前，session.json 会先复制到 <ALVR>\\配置备份，原版 exe 复制到 .alvr-l10n。如果补丁版启动失败，工具会自动还原官方版。任何时候点「还原成英文」都能拿回官方版本。", "寫入任何東西之前，session.json 會先複製到 <ALVR>\\配置备份，原版 exe 複製到 .alvr-l10n。如果補丁版啟動失敗，工具會自動還原官方版。任何時候按「還原成英文」都能拿回官方版本。", "書き込む前に session.json を <ALVR>\\配置备份 へ、元の exe を .alvr-l10n へコピーします。パッチ版が起動しない場合は公式版へ自動で戻します。「英語に戻す」でいつでも復元できます。", "쓰기 전에 session.json은 <ALVR>\\配置备份로, 원본 exe는 .alvr-l10n으로 복사합니다. 패치 버전이 시작되지 않으면 공식 버전으로 자동 복원합니다. 영어로 복원을 누르면 언제든 되돌릴 수 있습니다."]),
    ("browse_exe", ["Choose ALVR Dashboard.exe…", "选择 ALVR Dashboard.exe…", "選擇 ALVR Dashboard.exe…", "ALVR Dashboard.exe を選択…", "ALVR Dashboard.exe 선택…"]),
    ("tip_browse_exe", ["Pick the ALVR executable directly. The folder is remembered in tool.ini.", "直接指定 ALVR 的可执行文件，路径会记在 tool.ini 里。", "直接指定 ALVR 的執行檔，路徑會記在 tool.ini 裡。", "ALVR の実行ファイルを直接指定します。パスは tool.ini に保存されます。", "ALVR 실행 파일을 직접 지정합니다. 경로는 tool.ini에 저장됩니다."]),
    ("exe_invalid", ["That is not ALVR Dashboard.exe", "这不是 ALVR Dashboard.exe", "這不是 ALVR Dashboard.exe", "ALVR Dashboard.exe ではありません", "ALVR Dashboard.exe가 아닙니다"]),
    ("exe_using", ["Using the executable you picked", "正在使用你指定的可执行文件", "正在使用你指定的執行檔", "指定された実行ファイルを使用中", "지정한 실행 파일 사용 중"]),
    ("fetch_packs", ["Get more language packs", "获取更多语言包", "取得更多語言包", "言語パックを取得", "언어 팩 더 받기"]),
    ("fetch_busy", ["Downloading…", "正在下载…", "正在下載…", "ダウンロード中…", "다운로드 중…"]),
    ("fetch_none", ["All language packs are up to date.", "所有语言包都是最新的。", "所有語言包都是最新的。", "すべての言語パックは最新です。", "모든 언어 팩이 최신입니다."]),
    ("fetch_ok", ["Downloaded packs:", "已下载语言包：", "已下載語言包：", "ダウンロードしたパック：", "다운로드한 팩:"]),
    ("fetch_fail", ["Download failed — check your network.", "下载失败——请检查网络。", "下載失敗——請檢查網路。", "ダウンロード失敗——ネットワークを確認してください。", "다운로드 실패 — 네트워크를 확인하세요."]),
    ("export_template", ["Export translation template…", "导出翻译模板…", "匯出翻譯模板…", "翻訳テンプレートを書き出す…", "번역 템플릿 내보내기…"]),
    ("template_saved", ["Template saved:", "模板已保存：", "模板已儲存：", "テンプレートを保存しました：", "템플릿 저장됨:"]),
    ("template_fail", ["Could not save the template.", "模板保存失败。", "模板儲存失敗。", "テンプレートを保存できませんでした。", "템플릿을 저장하지 못했습니다."]),
    ("import_any_title", ["Language pack or translation file", "语言包或翻译文件", "語言包或翻譯檔", "言語パックまたは翻訳ファイル", "언어 팩 또는 번역 파일"]),
    ("import_converted", ["Converted and added:", "已转换并添加：", "已轉換並新增：", "変換して追加しました：", "변환하여 추가했습니다:"]),
    ("import_empty", ["No translations found in that file.", "该文件里没有找到译文。", "該檔案裡找不到譯文。", "そのファイルに翻訳が見つかりません。", "해당 파일에서 번역을 찾지 못했습니다."]),
    ("font_needed", ["This language needs a font file: put <id>.ttf next to the .ini, otherwise ALVR shows squares.", "该语言需要字体：把 <id>.ttf 放在 ini 旁边，否则 ALVR 里会显示成方框。", "該語言需要字型：把 <id>.ttf 放在 ini 旁邊，否則 ALVR 裡會顯示成方框。", "この言語にはフォントが必要です。<id>.ttf を .ini と同じ場所に置いてください。", "이 언어는 글꼴이 필요합니다. <id>.ttf를 .ini 옆에 두세요."]),
    ("help_custom_title", ["Add a new language yourself", "自己新增一个语种", "自己新增一個語種", "自分で新しい言語を追加", "직접 새 언어 추가"]),
    ("help_custom_body", ["1) Press \"Export translation template…\" and save the CSV. 2) Translate the second column (Excel, Google Translate, DeepL — anything). 3) Press \"Add language pack…\" and pick that CSV; it is converted into a pack immediately. 4) Tick it in the list and press Apply. 5) If the language needs a font (Hindi, Thai, Arabic…), put <id>.ttf next to the .ini first — the tool tells you when that is required.", "1）点「导出翻译模板…」保存 CSV；2）翻译第二列（Excel、机翻、人工都行）；3）点「扩展语言包…」选这个 CSV，会自动转成语言包；4）在列表里勾上，点「应用」；5）如果该语言需要字体（印地语、泰语、阿拉伯语…），先把 <id>.ttf 放到 ini 旁边——需要时工具会提示你。", "1）按「匯出翻譯模板…」儲存 CSV；2）翻譯第二欄（Excel、機翻、人工都行）；3）按「擴充語言包…」選這個 CSV，會自動轉成語言包；4）在清單裡勾選，按「套用」；5）如果該語言需要字型（印地語、泰語、阿拉伯語…），先把 <id>.ttf 放到 ini 旁邊——需要時工具會提示。", "1) 「翻訳テンプレートを書き出す…」で CSV を保存。2) 2列目を翻訳。3) 「言語パックを追加…」でその CSV を選択（自動変換）。4) 一覧でチェックして適用。5) フォントが必要な言語（ヒンディー語・タイ語・アラビア語など）は <id>.ttf を .ini の隣に置いてください。", "1) 번역 템플릿 내보내기로 CSV 저장. 2) 두 번째 열을 번역. 3) 언어 팩 추가…에서 그 CSV 선택(자동 변환). 4) 목록에서 체크 후 적용. 5) 글꼴이 필요한 언어(힌디어·태국어·아랍어 등)는 <id>.ttf를 .ini 옆에 두세요."]),
    ("nav_translate", ["Translate", "翻译", "翻譯", "翻訳", "번역"]),
    ("ai_intro", ["Let an AI translate the interface for you. Bring your own API key — it is used by this tool only.", "让 AI 帮你翻译界面，用你自己的 API Key，只在本机使用。", "讓 AI 幫你翻譯介面，用你自己的 API Key，只在本機使用。", "AI に翻訳させます。ご自身の API キーを使用し、このツール内でのみ使われます。", "AI로 번역합니다. 본인의 API 키를 사용하며 이 도구에서만 쓰입니다."]),
    ("ai_base", ["API base URL", "接口地址", "介面位址", "API ベース URL", "API 기본 URL"]),
    ("ai_model", ["Model", "模型", "模型", "モデル", "모델"]),
    ("ai_key", ["API key", "API Key", "API Key", "API キー", "API 키"]),
    ("ai_language", ["Target language", "目标语言", "目標語言", "対象言語", "대상 언어"]),
    ("ai_pack_id", ["Pack id (file name)", "语言包 ID（文件名）", "語言包 ID（檔名）", "パック ID（ファイル名）", "팩 ID(파일 이름)"]),
    ("ai_start", ["Translate missing strings", "翻译未翻译的条目", "翻譯未翻譯的條目", "未翻訳の項目を翻訳", "번역되지 않은 항목 번역"]),
    ("ai_stop", ["Stop", "停止", "停止", "停止", "중지"]),
    ("ai_remember", ["Remember the key in tool.ini (plain text)", "在 tool.ini 里记住密钥（明文）", "在 tool.ini 裡記住密鑰（明文）", "キーを tool.ini に保存（平文）", "키를 tool.ini에 저장(평문)"]),
    ("ai_progress", ["Progress", "进度", "進度", "進捗", "진행률"]),
    ("ai_ready", ["Fill in the fields and press translate.", "填好上面的信息，然后点翻译。", "填好上面的資訊，然後按翻譯。", "上を入力して翻訳を押してください。", "위 항목을 입력하고 번역을 누르세요."]),
    ("ai_saved_pack", ["Language pack written:", "已生成语言包：", "已產生語言包：", "言語パックを作成しました：", "언어 팩 생성됨:"]),
    ("ai_provider", ["Service", "服务商", "服務商", "サービス", "서비스"]),
    ("ai_test", ["Test connection", "测试连接", "測試連線", "接続テスト", "연결 테스트"]),
    ("ai_testing", ["Testing…", "正在测试…", "正在測試…", "テスト中…", "테스트 중…"]),
    ("ai_test_ok", ["Connection works. Reply:", "连接成功，返回：", "連線成功，回傳：", "接続成功。応答:", "연결 성공. 응답:"]),
    ("ai_test_fail", ["Connection failed:", "连接失败：", "連線失敗：", "接続失敗:", "연결 실패:"]),
    ("ai_key_hint", ["Get a key here", "在这里申请密钥", "在這裡申請密鑰", "キーはこちら", "여기서 키 발급"]),
    ("ai_language_pick", ["Pick a language", "选择语言", "選擇語言", "言語を選択", "언어 선택"]),
    ("ai_auto_fill", ["Everything is filled in from these two dropdowns — no typing needed.", "服务商和语言选好就行，其余自动填。", "服務商和語言選好就行，其餘自動填。", "サービスと言語を選ぶだけで自動入力されます。", "서비스와 언어만 고르면 나머지는 자동으로 채워집니다."]),
    ("nav_setup", ["Quick setup", "推荐配置", "推薦設定", "かんたん設定", "빠른 설정"]),
    ("setup_intro", ["The tool reads your hardware and suggests ALVR settings that match it. Nothing is changed until you press Apply.", "工具会读取你的硬件，给出匹配的 ALVR 参数；不点「应用」不会改任何东西。", "工具會讀取你的硬體，給出匹配的 ALVR 參數；不按「套用」不會改任何東西。", "ハードウェアを読み取り、合った設定を提案します。「適用」を押すまで変更しません。", "하드웨어를 읽어 알맞은 설정을 제안합니다. 적용을 누르기 전에는 아무것도 바뀌지 않습니다."]),
    ("setup_cpu", ["CPU", "处理器", "處理器", "CPU", "CPU"]),
    ("setup_ram", ["Memory", "内存", "記憶體", "メモリ", "메모리"]),
    ("setup_gpu", ["Graphics", "显卡", "顯示卡", "GPU", "GPU"]),
    ("setup_disk", ["Free space", "可用空间", "可用空間", "空き容量", "여유 공간"]),
    ("setup_headset", ["Headset", "头显", "頭顯", "ヘッドセット", "헤드셋"]),
    ("setup_link", ["Connection", "连接方式", "連線方式", "接続", "연결"]),
    ("setup_wired", ["Wired", "有线", "有線", "有線", "유선"]),
    ("setup_wireless", ["Wireless", "无线", "無線", "無線", "무선"]),
    ("setup_redetect", ["Detect hardware again", "重新检测硬件", "重新偵測硬體", "ハードウェアを再検出", "하드웨어 다시 감지"]),
    ("setup_result", ["Suggested settings", "推荐参数", "推薦參數", "推奨設定", "추천 설정"]),
    ("setup_tier", ["Tier", "档位", "等級", "クラス", "등급"]),
    ("setup_codec", ["Codec", "编码器", "編碼器", "コーデック", "코덱"]),
    ("setup_bitrate", ["Bitrate", "码率", "位元率", "ビットレート", "비트레이트"]),
    ("setup_resolution", ["Per-eye resolution", "单眼分辨率", "單眼解析度", "片眼解像度", "편안 해상도"]),
    ("setup_refresh", ["Refresh rate", "刷新率", "刷新率", "リフレッシュレート", "주사율"]),
    ("setup_apply", ["Apply to ALVR", "应用到 ALVR", "套用到 ALVR", "ALVR に適用", "ALVR에 적용"]),
    ("setup_applied", ["Applied. Restart ALVR to see it.", "已应用，重启 ALVR 生效。", "已套用，重啟 ALVR 生效。", "適用しました。ALVR を再起動してください。", "적용했습니다. ALVR를 다시 시작하세요."]),
    ("setup_apply_failed", ["Could not apply:", "应用失败：", "套用失敗：", "適用できませんでした:", "적용 실패:"]),
    ("setup_bitrate_note", ["Bitrate applies in constant-bitrate mode.", "码率仅在固定码率模式下生效。", "位元率僅在固定位元率模式下生效。", "ビットレートは固定モードでのみ有効です。", "비트레이트는 고정 모드에서만 적용됩니다."]),
    ("tier_high", ["High", "高", "高", "高", "높음"]),
    ("tier_mid", ["Medium", "中", "中", "中", "보통"]),
    ("tier_entry", ["Entry", "入门", "入門", "エントリー", "입문"]),
    ("tier_low", ["Very low", "很低", "很低", "very low", "매우 낮음"]),
    ("ai_stopping", ["Stopping…", "正在停止…", "正在停止…", "停止しています…", "중지 중…"]),
    ("ai_need_fields", ["Fill in service, model, key, language and pack id first.", "请先填写服务商、模型、密钥、语言和语言包 ID。", "請先填寫服務商、模型、密鑰、語言和語言包 ID。", "サービス・モデル・キー・言語・パック ID を入力してください。", "서비스, 모델, 키, 언어, 팩 ID를 입력하세요."]),
    ("ai_nothing_left", ["Everything is already translated.", "所有条目都已翻译。", "所有條目都已翻譯。", "すべて翻訳済みです。", "모두 번역되었습니다."]),
    ("ai_translating", ["Translating strings:", "正在翻译条数：", "正在翻譯條數：", "翻訳中の件数:", "번역 중인 항목:"]),
    ("ai_write_failed", ["Could not write the language pack.", "写入语言包失败。", "寫入語言包失敗。", "言語パックを書き込めませんでした。", "언어 팩을 쓰지 못했습니다."]),
    ("ai_test_died", ["The test request crashed.", "测试请求异常结束。", "測試請求異常結束。", "テスト要求が異常終了しました。", "테스트 요청이 비정상 종료되었습니다."]),
    ("ai_key_note", ["The key is only sent to the endpoint above, and only when you press a button.", "密钥只会发给你上面填的接口，而且只在你点按钮时才发。", "密鑰只會發給你上面填的介面，而且只在你按按鈕時才發。", "キーは上記のエンドポイントにのみ、ボタンを押したときだけ送信されます。", "키는 위 엔드포인트로만, 버튼을 누를 때만 전송됩니다."]),
    ("help_quick_title", ["Quick start (3 steps)", "三步上手", "三步上手", "クイックスタート（3ステップ）", "빠른 시작(3단계)"]),
    ("help_quick_body", ["1) Close ALVR.\n2) Open the Localize page, tick the languages you want, press Apply.\n3) Start ALVR again and pick the language in the tab bar of ALVR's Settings page.\nIf you only want the recommended video settings, use Quick setup instead — it does not touch languages.", "1）先关掉 ALVR。\n2）打开「汉化」页，勾选需要的语言，点「应用」。\n3）重新打开 ALVR，在「设置」页顶部标签栏右侧的语言下拉里切换。\n如果你只想调画质，用「推荐配置」页即可，它不动语言。", "1）先關掉 ALVR。\n2）打開「漢化」頁，勾選需要的語言，按「套用」。\n3）重新打開 ALVR，在「設定」頁頂部標籤列右側的語言下拉裡切換。\n如果你只想調畫質，用「推薦設定」頁即可，它不動語言。", "1) ALVR を閉じます。\n2) 「漢化」ページで言語にチェックし「適用」。\n3) ALVR を開き、設定ページ上部の言語メニューで切り替えます。\n画質だけ調整したい場合は「かんたん設定」を使います（言語は変更しません）。", "1) ALVR를 닫습니다.\n2) 한글화 페이지에서 언어를 체크하고 적용.\n3) ALVR을 다시 열고 설정 페이지 상단의 언어 메뉴에서 선택합니다.\n화질만 조정하려면 빠른 설정을 사용하세요(언어는 건드리지 않습니다)."]),
    ("help_localize_title", ["Localize page — what each button does", "「汉化」页：每个按钮的作用", "「漢化」頁：每個按鈕的作用", "「漢化」ページの各ボタン", "한글화 페이지 버튼 설명"]),
    ("help_localize_body", ["Language list: click a row to include or exclude that pack. Only included packs are copied into ALVR, so ALVR's own language menu shows exactly these.\nApply: writes the packs, sets the default language and installs the localized build (or patches the official one when no build exists).\nRestore English: puts the original ALVR executables back.\nBack up settings: copies session.json into ALVR\\配置备份.\nGet more language packs: downloads packs you do not have yet from this project's repository.\nAdd language pack…: accepts .ini, or a .csv/.json translation you made in Excel or with an AI.", "语言列表：点一行即选中/取消。只有选中的包会被复制进 ALVR，所以 ALVR 的语言菜单里只有这些。\n应用：写入语言包、设定默认语言，并安装汉化版（没有对应版本时改为给官方版打补丁）。\n还原成英文：把官方 ALVR 可执行文件放回去。\n备份我的设置：把 session.json 复制到 ALVR\\配置备份。\n获取更多语言包：从本项目仓库下载你还没有的语言包。\n扩展语言包…：可以选 .ini，也可以选你在 Excel / AI 里做好的 .csv 或 .json 译文。", "語言清單：點一列即選取/取消。只有選取的包會被複製進 ALVR，所以 ALVR 的語言選單裡只有這些。\n套用：寫入語言包、設定預設語言，並安裝中文化版（沒有對應版本時改為給官方版打補丁）。\n還原成英文：把官方 ALVR 執行檔放回去。\n備份我的設定：把 session.json 複製到 ALVR\\配置备份。\n取得更多語言包：從本專案儲存庫下載你還沒有的語言包。\n擴充語言包…：可以選 .ini，也可以選你在 Excel / AI 裡做好的 .csv 或 .json 譯文。", "言語リスト：行をクリックで選択/解除。選んだパックだけが ALVR にコピーされます。\n適用：パックを書き込み、既定言語を設定し、ローカライズ版をインストールします。\n英語に戻す：公式の実行ファイルに戻します。\n設定をバックアップ：session.json を ALVR\\配置备份 にコピーします。\n言語パックを取得：未取得のパックをリポジトリからダウンロードします。\n言語パックを追加…：.ini のほか、Excel や AI で作った .csv/.json も使えます。", "언어 목록: 행을 클릭해 선택/해제합니다. 선택한 팩만 ALVR로 복사됩니다.\n적용: 팩을 쓰고 기본 언어를 설정하며 한글화 빌드를 설치합니다.\n영어로 복원: 공식 실행 파일로 되돌립니다.\n설정 백업: session.json을 ALVR\\配置备份로 복사합니다.\n언어 팩 더 받기: 아직 없는 팩을 저장소에서 내려받습니다.\n언어 팩 추가…: .ini 외에 Excel/AI로 만든 .csv/.json도 사용할 수 있습니다."]),
    ("help_setup_title", ["Quick setup page", "「推荐配置」页", "「推薦設定」頁", "「かんたん設定」ページ", "빠른 설정 페이지"]),
    ("help_setup_body", ["The page reads CPU, RAM, GPUs (with VRAM), free space and the headset ALVR remembers, then suggests a codec, bitrate, per-eye resolution and refresh rate.\nSwitch wired/wireless: that changes the bitrate target (wireless keeps more headroom).\nApply to ALVR: copies session.json into ALVR\\配置备份 first, then changes only these keys — refresh_rate, eye_resolution_*, target_eye_resolution_*, enable_foveated_encoding, controllers_enabled, preferred_codec, the constant bitrate and the transcoding width. Everything else stays as it was.\nAfter applying, restart ALVR. If the picture stutters, come back and pick wireless, or lower the tier by raising nothing — just use ALVR's own presets.", "本页会读取 CPU、内存、显卡（含显存）、可用空间和 ALVR 记得的头显，然后给出编码器、码率、单眼分辨率和刷新率。\n有线/无线：会改变码率目标（无线会留更多余量）。\n应用到 ALVR：先把 session.json 复制到 ALVR\\配置备份，然后只改这些键——refresh_rate、eye_resolution_*、target_eye_resolution_*、enable_foveated_encoding、controllers_enabled、preferred_codec、固定码率、转码宽度；其它字段一律不动。\n应用后请重启 ALVR。如果画面卡，回来选「无线」，或者干脆回到 ALVR 自己的预设档位。", "本頁會讀取 CPU、記憶體、顯示卡（含 VRAM）、可用空間和 ALVR 記得的頭顯，然後給出編碼器、位元率、單眼解析度和刷新率。\n有線/無線：會改變位元率目標（無線會留更多餘量）。\n套用到 ALVR：先把 session.json 複製到 ALVR\\配置备份，然後只改這些鍵——refresh_rate、eye_resolution_*、target_eye_resolution_*、enable_foveated_encoding、controllers_enabled、preferred_codec、固定位元率、轉碼寬度；其它欄位一律不動。\n套用後請重啟 ALVR。", "CPU・メモリ・GPU（VRAM）・空き容量・ヘッドセットを読み取り、コーデック・ビットレート・解像度・リフレッシュレートを提案します。\n有線/無線でビットレート目標が変わります。\nALVR に適用：先に session.json を ALVR\\配置备份 にコピーし、上記のキーだけを変更します。\n適用後は ALVR を再起動してください。", "CPU, 메모리, GPU(VRAM), 여유 공간, 헤드셋을 읽고 코덱·비트레이트·해상도·주사율을 제안합니다.\n유선/무선에 따라 비트레이트 목표가 달라집니다.\nALVR에 적용: 먼저 session.json을 ALVR\\配置备份에 복사하고 위 키만 변경합니다.\n적용 후 ALVR을 다시 시작하세요."]),
    ("help_translate_title", ["Translate page — add a language with your AI", "「翻译」页：用你的 AI 添加语种", "「翻譯」頁：用你的 AI 新增語種", "「翻訳」ページ：AI で言語を追加", "번역 페이지: AI로 언어 추가"]),
    ("help_translate_body", ["Pick a service (OpenAI, DeepSeek, Moonshot, GLM, SiliconFlow, local Ollama…), pick the language, paste your key. The model, endpoint and pack id fill themselves in.\nTest connection sends one short string so you see the reply before translating everything.\nTranslate missing strings sends the untranslated entries in batches of 25 and writes languages\\<id>.ini. Existing translations are kept, so you can run it again later.\nIf the language uses a script ALVR's font cannot draw (Hindi, Thai, Arabic, Hebrew…), a <id>.ttf is required — the tool tells you when it is missing.", "选服务商（OpenAI、DeepSeek、Moonshot、智谱、SiliconFlow、本地 Ollama…）→ 选语言 → 贴 Key，模型、接口地址、语言包 ID 都会自动填好。\n测试连接：先发一条短文本，确认能收到回复再整体翻译。\n翻译未翻译的条目：每 25 条一批发给 AI，结果写入 languages\\<id>.ini；已有的译文会保留，所以可以过几天再跑一次补齐。\n如果该语言的字形 ALVR 自带字体没有（印地语、泰语、阿拉伯语、希伯来语…），还需要一个 <id>.ttf，缺的时候工具会提示。", "選服務商（OpenAI、DeepSeek、Moonshot、智譜、SiliconFlow、本地 Ollama…）→ 選語言 → 貼 Key，模型、介面位址、語言包 ID 都會自動填好。\n測試連線：先發一條短文本，確認能收到回覆再整體翻譯。\n翻譯未翻譯的條目：每 25 條一批發給 AI，結果寫入 languages\\<id>.ini；已有的譯文會保留。\n如果該語言的字形 ALVR 自帶字型沒有（印地語、泰語、阿拉伯語、希伯來語…），還需要一個 <id>.ttf。", "サービス（OpenAI/DeepSeek/Moonshot/GLM/SiliconFlow/ローカル Ollama など）→ 言語 → キーを貼るだけ。モデル・URL・パック ID は自動入力されます。\n接続テストで 1 件だけ送って確認できます。\n未翻訳の項目を 25 件ずつ送信し、languages\\<id>.ini に書き込みます。既存の訳は保持されます。\nフォントが必要な言語は <id>.ttf も必要です。", "서비스(OpenAI/DeepSeek/Moonshot/GLM/SiliconFlow/로컬 Ollama 등) → 언어 → 키 붙여넣기. 모델·URL·팩 ID는 자동 입력됩니다.\n연결 테스트로 1건만 보내 확인할 수 있습니다.\n번역되지 않은 항목을 25개씩 보내 languages\\<id>.ini에 기록합니다. 기존 번역은 유지됩니다.\n글꼴이 필요한 언어는 <id>.ttf도 필요합니다."]),
    ("help_faq_title", ["Common problems", "常见问题", "常見問題", "よくある問題", "자주 묻는 문제"]),
    ("help_faq_body", ["ALVR not detected: use “Choose ALVR Dashboard.exe…” and pick the file directly.\nApplied but ALVR still English: restart ALVR; if it still shows English, check that language.ini exists in the ALVR folder and the pack is inside ALVR\\languages.\nSquares instead of letters: that language needs a <id>.ttf next to its .ini.\nPicture stutters after Quick setup: switch to wireless (lower bitrate) or lower the resolution in ALVR's own presets.\nWant the original back: press Restore English, then delete language.ini, languages\\, 配置备份\\ and the two .alvr-l10n folders.", "认不出 ALVR：用「选择 ALVR Dashboard.exe…」直接指定文件。\n应用了但 ALVR 还是英文：先重启 ALVR；仍是英文就检查 ALVR 目录下有没有 language.ini、ALVR\\languages 里有没有该语言包。\n显示成方框：该语言需要在 ini 旁边放 <id>.ttf。\n应用推荐配置后画面卡：改选「无线」（码率更低），或回到 ALVR 自己的预设里降一档分辨率。\n想完全恢复原样：点「还原成英文」，再删掉 language.ini、languages\\、配置备份\\ 和两个 .alvr-l10n 文件夹。", "認不出 ALVR：用「選擇 ALVR Dashboard.exe…」直接指定檔案。\n套用了但 ALVR 還是英文：先重啟 ALVR；仍是英文就檢查 ALVR 目錄下有沒有 language.ini、ALVR\\languages 裡有沒有該語言包。\n顯示成方框：該語言需要在 ini 旁邊放 <id>.ttf。\n套用推薦設定後畫面卡：改選「無線」（位元率更低），或回到 ALVR 自己的預設裡降一檔解析度。\n想完全恢復原樣：按「還原成英文」，再刪掉 language.ini、languages\\、配置备份\\ 和兩個 .alvr-l10n 資料夾。", "ALVR が見つからない：「ALVR Dashboard.exe を選択…」で直接指定してください。\n適用したのに英語のまま：ALVR を再起動。それでも駄目なら language.ini と ALVR\\languages を確認。\n文字が四角になる：<id>.ttf が必要です。\n画質が重い：無線を選ぶか解像度を下げてください。\n元に戻す：「英語に戻す」の後、language.ini・languages・配置备份・.alvr-l10n を削除。", "ALVR를 찾지 못함: ALVR Dashboard.exe 선택…으로 직접 지정하세요.\n적용했는데 영어 그대로: ALVR 재시작 후 language.ini와 ALVR\\languages를 확인하세요.\n글자가 네모로 보임: <id>.ttf가 필요합니다.\n화면이 끊김: 무선을 선택하거나 해상도를 낮추세요.\n원래대로: 영어로 복원 후 language.ini, languages, 配置备份, .alvr-l10n 삭제."]),
    ("help_files_title", ["Where the files are", "文件都在哪", "檔案都在哪", "ファイルの場所", "파일 위치"]),
    ("help_files_body", ["ALVR\\language.ini — which language the localized build starts with.\nALVR\\languages\\ — the installed packs (only the ones you ticked).\nALVR\\配置备份\\ — session.json backups, one per change.\nALVR\\.alvr-l10n\\ and ALVR\\installations\\<version>\\.alvr-l10n\\ — the original executables.\nTool folder\\languages\\ — the pack library this tool reads; Tool folder\\tool.ini — your choices, including the API settings if you asked to remember them.", "ALVR\\language.ini —— 汉化版启动时用哪个语言。\nALVR\\languages\\ —— 已安装的语言包（只有你勾选的那些）。\nALVR\\配置备份\\ —— session.json 的备份，每次改动一份。\nALVR\\.alvr-l10n\\ 和 ALVR\\installations\\<版本>\\.alvr-l10n\\ —— 原版可执行文件。\n工具目录\\languages\\ —— 工具读取的语言包库；工具目录\\tool.ini —— 你的选择，勾了「记住密钥」时也含 API 设置。", "ALVR\\language.ini —— 中文化版啟動時用哪個語言。\nALVR\\languages\\ —— 已安裝的語言包（只有你勾選的那些）。\nALVR\\配置备份\\ —— session.json 的備份，每次改動一份。\nALVR\\.alvr-l10n\\ 和 ALVR\\installations\\<版本>\\.alvr-l10n\\ —— 原版執行檔。\n工具目錄\\languages\\ —— 工具讀取的語言包庫；工具目錄\\tool.ini —— 你的選擇。", "ALVR\\language.ini — 起動時の言語。\nALVR\\languages\\ — インストール済みパック。\nALVR\\配置备份\\ — session.json のバックアップ。\nALVR\\.alvr-l10n\\ — 元の実行ファイル。\nツール\\languages\\ — パックのライブラリ、ツール\\tool.ini — 設定。", "ALVR\\language.ini — 시작 언어.\nALVR\\languages\\ — 설치된 팩.\nALVR\\配置备份\\ — session.json 백업.\nALVR\\.alvr-l10n\\ — 원본 실행 파일.\n도구\\languages\\ — 팩 라이브러리, 도구\\tool.ini — 설정."]),
    ("setup_headset_auto", ["Auto (from ALVR)", "自动（来自 ALVR）", "自動（來自 ALVR）", "自動（ALVR から）", "자동(ALVR에서)"]),
    ("setup_headset_pick", ["Headset model", "头显型号", "頭顯型號", "ヘッドセット機種", "헤드셋 모델"]),
    ("setup_ready", ["Nothing has been changed yet — press Apply when you are happy with the suggestion.", "尚未改动任何东西——看完推荐觉得可以，再点「应用」。", "尚未改動任何東西——看完推薦覺得可以，再按「套用」。", "まだ何も変更していません。納得できたら「適用」を押してください。", "아직 아무것도 바뀌지 않았습니다. 마음에 들면 적용을 누르세요."]),
];
