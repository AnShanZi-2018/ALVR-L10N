# 宣传文案（复制即用）

## Discord（ALVR 官方服务器）

频道建议：`#community` 或 `#show-and-tell`（没有的话找 `#off-topic`）

```
Community tool: multi-language ALVR dashboard (zh-CN / zh-TW / ja-JP / ko-KR / ru-RU / fr-FR)

Since the dashboard is English-only (alvr-org/ALVR#1927), I built a standalone tool that adds a
language layer on top of a normal ALVR install — no fork, no rebuilt source tree.

• ships a localized ALVR Dashboard.exe / Launcher.exe per release (v20.14.1 now); a language
  selector appears in the tab bar, the settings tree is translated too
• for versions without a bundled build it patches the official exe in place (byte-preserving
  string + font swap)
• only the packs you tick land in ALVR/languages, so ALVR's own menu lists exactly those
• "Restore English" puts the originals back; session.json is backed up before every change
• bonus: quick-setup page (reads CPU/GPU/VRAM and suggests codec/bitrate/resolution) and a
  translate page that uses your own OpenAI-compatible endpoint to generate a new language pack

Repo + releases: https://github.com/AnShanZi-2018/ALVR-L10N
Upstream thread: https://github.com/alvr-org/ALVR/discussions/3460

MIT, community-made, not affiliated with the project, no driver/APK changes.
Native-speaker corrections on the translations are very welcome.
```

## B站（动态或专栏）

**标题**（≤ 40 字）
```
ALVR 终于能看懂了：6 种语言的汉化工具，装完就能用
```

**正文**
```
ALVR 的串流面板一直是纯英文，官方也明确说过短期内不做多语言（issue #1927）。
所以我写了个独立的小工具，不改源码、不重装，给 ALVR 套一层语言支持。

■ 能做什么
· 简中 / 繁中 / 日 / 韩 / 俄 / 法 六种语言
· 汉化版是整个界面重编译的，设置项也翻译了，标签栏右边还能随时切语言
· 没有对应版本的 ALVR 会自动退化成"给官方 exe 打补丁"，不会让你卡在旧版本
· 只把你勾选的语言包放进 ALVR\languages，ALVR 自己的语言菜单里就只出现这些
· 随时「还原成英文」；每次改动前都会备份 session.json，启动失败会自动还原

■ 顺手加的两个功能
· 推荐配置：读 CPU / 显卡 / 显存 / 内存，直接给出编码器、码率、分辨率、刷新率，一键写进 ALVR
· 翻译页：接你自己的 OpenAI 兼容接口（OpenAI / DeepSeek / 月之暗面 / 智谱 / 本地 Ollama），
  自动把 651 条界面文本翻成新语言，等于人人都能加语种

■ 哪里拿
GitHub 搜 ALVR-L10N，或直接 releases 页下载 zip，解压双击就能用（Windows）
https://github.com/AnShanZi-2018/ALVR-L10N/releases

■ 说明
MIT 开源、社区作品、和 ALVR 官方无关；不动驱动、不动头显里的 APK。
翻译有错欢迎指正，尤其是母语者——语言包就是纯文本，改完提 PR 就行。

#ALVR #VR #串流 #Quest3 #汉化 #开源
```

**分区**：科技 → 软件应用（或 数码 → 虚拟现实）

**封面**：见 `G:\DEVENV\ALVR-work\promo\alvr-l10n-cover.png`（1280×800）
