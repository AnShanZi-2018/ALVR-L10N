# Posting this to the ALVR community

Drafts for the upstream ALVR Discussions post. Keep it factual: it is a community
tool, ALVR itself is not modified, and everything is reversible.

Upstream context worth quoting:

* zmerp in [issue #1927](https://github.com/alvr-org/ALVR/issues/1927): *"The new
  dashboard does not support translations."* — so a user-facing workaround is useful
  until upstream adds i18n.
* [issue #281](https://github.com/alvr-org/ALVR/issues/281) was the old (web UI)
  translation effort, closed when the dashboard was rewritten in Rust.

---

## English

**Title:** Community tool: multi-language ALVR dashboard (zh-CN / zh-TW / ja-JP / ko-KR / ru-RU / fr-FR)

Hi everyone,

since ALVR's dashboard is English-only (see #1927), I wrote a small standalone tool
that adds a language layer on top of a normal ALVR installation — **no fork, no
modified source tree required**.

**What it does**

* ships a localized build of `ALVR Dashboard.exe` / `ALVR Launcher.exe` per ALVR
  release (v20.14.1 for now), so the whole UI including the settings tree is
  translated and a language selector appears in the tab bar;
* for ALVR versions without a bundled build it falls back to patching the official
  executable in place (string + font substitution, byte-length preserving);
* only the languages you tick are copied into `ALVR/languages`, so ALVR's own menu
  lists exactly those;
* `Restore English` puts the original executables back; `session.json` is backed up
  before anything is written.

**Also included** (because they were the annoying parts for me): a quick-setup page
that reads CPU/GPU/VRAM/RAM and suggests codec/bitrate/resolution, and a translate
page where you plug in your own OpenAI-compatible endpoint (OpenAI, DeepSeek,
Moonshot, GLM, SiliconFlow, local Ollama) to produce a new language pack.

Repository / releases: https://github.com/AnShanZi-2018/ALVR-L10N

It is MIT, community-made, not affiliated with the ALVR project, and does not touch
drivers or the client APK. Feedback and corrections — especially from native
speakers of the included languages — are very welcome. If upstream ever adds proper
i18n, this tool becomes unnecessary and I will happily point people there.

---

## 中文

**标题：** 社区工具：ALVR 面板多语言（简中 / 繁中 / 日 / 韩 / 俄 / 法）

ALVR 的 Dashboard 目前只有英文（见 #1927），所以我做了个独立小工具，在正常安装的
ALVR 上叠一层语言支持——不需要 fork，也不需要自己编译。

* 每个 ALVR 版本配一份汉化好的 `ALVR Dashboard.exe` / `ALVR Launcher.exe`（目前是
  v20.14.1），整个界面和设置树都翻译，并在标签栏加语言下拉；
* 没有对应版本的 ALVR 会退化为直接给官方 exe 打补丁（字符串 + 字体替换，长度不变）；
* 只把你勾选的语言包复制进 `ALVR/languages`，ALVR 自己的语言菜单里就只出现这些；
* `还原成英文` 会把原版 exe 放回去，写入前都会备份 `session.json`。

另外还带了：读 CPU/显卡/显存/内存 给推荐参数的「推荐配置」页，以及接自己的
OpenAI 兼容接口（OpenAI、DeepSeek、Moonshot、智谱、SiliconFlow、本地 Ollama）
自动生成新语言包的「翻译」页。

仓库 / 发布：https://github.com/AnShanZi-2018/ALVR-L10N

MIT 许可，社区作品，与 ALVR 官方无关，不动驱动也不动客户端 APK。欢迎指正，尤其是
母语者帮忙校对翻译。如果上游以后支持了 i18n，这个工具就没必要存在了，我会优先引导
大家去用官方方案。
