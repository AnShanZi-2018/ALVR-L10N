# tools/ — 跟随 ALVR 新版本重建汉化版

ALVR 更新会覆盖掉汉化过的 `ALVR Dashboard.exe`。**语言包不用重做**，
只要把语言层重新套到新版本源码上编译一次即可。这个目录就是自动化这件事。

## 环境要求（只需装一次）

- Rust（<https://rustup.rs>）
- Visual Studio Build Tools，勾选「使用 C++ 的桌面开发」（含 MSVC + Windows SDK）
- Python 3
- 能访问 GitHub / crates.io

## 一条命令重建

```powershell
cd tools
.\rebuild.ps1 -Tag v20.14.1 -Install
```

| 参数 | 说明 |
| --- | --- |
| `-Tag` | ALVR 版本标签，例如 `v20.15.0`（对应 GitHub Releases 的 tag） |
| `-Install` | 编译完自动替换到 `-AlvrDir` 指定的 ALVR 目录（默认是本包的上一级） |
| `-AlvrDir` | ALVR 根目录，默认 `..`（即本包所在目录） |
| `-WorkDir` | 源码与缓存目录，默认 `%TEMP%\alvr-l10n-rebuild` |
| `-KeepSources` | 保留解压后的源码（默认也保留，方便排查） |

脚本流程：下载该版本源码 → `apply_localization.py` 打语言层 → `cargo build` →
把 `ALVR Dashboard.exe`（和 `ALVR Launcher.exe`）复制出来，直接覆盖安装。
覆盖前会把官方 exe 备份成 `*.exe.orig`。

## 如果新版本改了代码

`apply_localization.py` 用的是**锚点匹配**而不是行号，所以上游小改不会失效。
如果某次 ALVR 重构导致某个锚点消失，脚本会输出类似：

```
ERROR: .../settings.rs: anchor not found for language selector:
    '        if self.selected_top_tab_id == "presets" {'
```

这时打开 `apply_localization.py`，把对应的锚点字符串改成新版本里真实的那一行，
再跑一次即可。要改的地方一共就 5 处：dashboard `main()`、launcher `main()`、
`theme.rs` 的 `set_theme`、设置页的 `ui()`、以及 epaint 的 `layout_job`。

## 语言包与字体

- `languages/*.ini` 是纯文本，改完重新选一次语言就生效，不需要重新编译；
- 内嵌字体是按语言包实际用到的字符生成的子集。新增语言如果用到新字符
  （例如阿拉伯文），需要在源码里重新生成：
  1. 把该语言的全部译文收集到一个文本文件；
  2. 用 `fonttools` 重新 subset（参考 `build_font.py` 的做法），
     输出到 `alvr/gui_common/resources/alvr-ui-cjk-sc.ttf`（或 `-kr.ttf`）；
  3. 重新编译一次。
  这也是唯一需要动到「字体」而不是「ini」的情况。
