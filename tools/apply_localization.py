#!/usr/bin/env python3
"""Apply the ALVR localization patch series to a fresh ALVR source tree.

The patch is *anchor based*: instead of relying on line numbers (which change with
every ALVR release) every edit is located by a small piece of code that has to stay
stable. If upstream rewrites one of those places the script tells you exactly which
anchor failed and leaves the tree untouched for that file.

Everything this script touches is language related:
  * a new `alvr/i18n` crate (reads language.ini + languages/*.ini)
  * a hook in `epaint` that translates text right before layout
  * register the CJK fallback fonts in `alvr_gui_common::theme`
  * install the translator in the dashboard / launcher `main()`
  * a language selector at the top of the settings tab

Usage:
    python apply_localization.py --source D:\\ALVR-src [--payload DIR] [--no-epaint]
"""

from __future__ import annotations

import argparse
import re
import shutil
import sys
import tarfile
import urllib.request
import zipfile
from pathlib import Path

MARK = "ALVR-L10N"

DASHBOARD_MAIN_INIT = f"""    // {MARK}: interface language, configured by `language.ini` + `languages/*.ini`
    // next to the executable (or in the ALVR folder).
    alvr_i18n::init();
    eframe::egui::epaint::text::set_text_translator(Some(alvr_i18n::translate));

"""

LAUNCHER_MAIN_INIT = f"""    // {MARK}: interface language, configured by `language.ini` + `languages/*.ini`.
    alvr_i18n::init();
    eframe::egui::epaint::text::set_text_translator(Some(alvr_i18n::translate));

"""

THEME_FONTS = f"""// {MARK}: fallback fonts with CJK coverage (Chinese/Japanese/Korean).
// Generated from the characters used by the language packs, see `languages/build_font.py`.
// Latin text keeps using the built-in fonts.
const CJK_SC_FONT: &[u8] = include_bytes!("../resources/alvr-ui-cjk-sc.ttf");
const CJK_KR_FONT: &[u8] = include_bytes!("../resources/alvr-ui-cjk-kr.ttf");

fn install_cjk_fonts(ctx: &Context) {{
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "alvr-ui-cjk-sc".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(CJK_SC_FONT)),
    );
    fonts.font_data.insert(
        "alvr-ui-cjk-kr".to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(CJK_KR_FONT)),
    );

    // Appended as fallbacks so Latin glyphs keep the original ALVR look.
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {{
        if let Some(list) = fonts.families.get_mut(&family) {{
            for name in ["alvr-ui-cjk-sc", "alvr-ui-cjk-kr"] {{
                if !list.iter().any(|existing| existing == name) {{
                    list.push(name.to_owned());
                }}
            }}
        }}
    }}

    ctx.set_fonts(fonts);
}}

"""

SETTINGS_LANGUAGE = f"""        // {MARK}: interface language selector. The choice is stored in `language.ini`
        // next to the executable (or in the ALVR folder) and applies immediately.
        ui.add_space(10.0);
        Frame::group(ui.style())
            .fill(theme::DARKER_BG)
            .inner_margin(egui::vec2(15.0, 12.0))
            .show(ui, |ui| {{
                ui.horizontal(|ui| {{
                    ui.label(RichText::new("Language").size(15.0));

                    let current_language = alvr_i18n::current_language();
                    let mut selected_language = current_language.clone();
                    egui::ComboBox::from_id_salt("alvr_interface_language")
                        .selected_text(alvr_i18n::language_display_name(&current_language))
                        .show_ui(ui, |ui| {{
                            for language in alvr_i18n::available_languages() {{
                                let name = if language.native_name.is_empty() {{
                                    language.english_name.clone()
                                }} else {{
                                    language.native_name.clone()
                                }};
                                ui.selectable_value(
                                    &mut selected_language,
                                    language.id.clone(),
                                    name,
                                );
                            }}
                        }});

                    if selected_language != current_language {{
                        alvr_i18n::set_language(&selected_language);
                        ui.ctx().request_repaint();
                    }}
                }});
            }});
        ui.add_space(10.0);

"""

EPAINT_HOOK = f"""// {MARK}: localization hook ----------------------------------------------------
//
// ALVR ships with English strings only. Instead of rewriting every widget call, the
// application installs a translator that rewrites text right before it is laid out.

/// Maps a source (English) UI string to its localized replacement.
pub type TextTranslator = fn(&str) -> Option<String>;

static TEXT_TRANSLATOR: std::sync::RwLock<Option<TextTranslator>> = std::sync::RwLock::new(None);

/// Install a global text translator, or `None` to disable localization.
pub fn set_text_translator(translator: Option<TextTranslator>) {{
    if let Ok(mut lock) = TEXT_TRANSLATOR.write() {{
        *lock = translator;
    }}
}}

fn translated_job(mut job: LayoutJob) -> LayoutJob {{
    let Some(translator) = TEXT_TRANSLATOR.read().ok().and_then(|lock| *lock) else {{
        return job;
    }};

    // Only fully single-format jobs are translated; mixed-format text is left alone.
    if job.sections.len() != 1 || job.text.is_empty() {{
        return job;
    }}

    let Some(translated) = translator(&job.text) else {{
        return job;
    }};
    if translated == job.text {{
        return job;
    }}

    let leading_space = job.sections[0].leading_space;
    let format = job.sections[0].format.clone();
    job.text = translated;
    job.sections = vec![crate::text::LayoutSection {{
        leading_space,
        byte_range: 0..job.text.len(),
        format,
    }}];

    job
}}

"""


class PatchError(Exception):
    pass


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def write(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def insert_before(path: Path, anchor: str, payload: str, label: str) -> bool:
    """Insert `payload` before `anchor`. Returns True when the file changed."""
    text = read(path)
    if MARK in payload and payload.strip() in text:
        return False
    if anchor not in text:
        raise PatchError(f"{path}: anchor not found for {label}:\n    {anchor[:90]!r}")
    write(path, text.replace(anchor, payload + anchor, 1))
    return True


def insert_after(path: Path, anchor: str, payload: str, label: str) -> bool:
    text = read(path)
    if MARK in payload and payload.strip() in text:
        return False
    if anchor not in text:
        raise PatchError(f"{path}: anchor not found for {label}:\n    {anchor[:90]!r}")
    write(path, text.replace(anchor, anchor + payload, 1))
    return True


def append_line(path: Path, line: str) -> bool:
    text = read(path)
    if line in text:
        return False
    write(path, text.rstrip("\n") + "\n" + line + "\n")
    return True


def ensure_line_after_header(path: Path, section: str, line: str) -> bool:
    """Make sure `line` exists inside the `[section]` block of a Cargo.toml."""
    text = read(path)
    if line.strip() in text:
        return False
    pattern = re.compile(rf"^\[{re.escape(section)}\]\s*$", re.MULTILINE)
    match = pattern.search(text)
    if not match:
        # Create the section at the end of the file.
        write(path, text.rstrip("\n") + f"\n\n[{section}]\n{line}\n")
        return True
    # Insert right after the section header, keeping blank lines tidy.
    insert_at = match.end()
    rest = text[insert_at:]
    stripped = rest.lstrip("\n")
    offset = insert_at + (len(rest) - len(stripped))
    write(path, text[:insert_at] + "\n" + line + "\n" + text[offset:])
    return True


# ---------------------------------------------------------------------------
# vendored crates
# ---------------------------------------------------------------------------

def download(url: str, dest: Path) -> Path:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if not dest.exists():
        print(f"  downloading {url}")
        urllib.request.urlretrieve(url, dest)
    return dest


def extract_zip(archive: Path, target: Path) -> Path:
    if target.exists():
        shutil.rmtree(target)
    with zipfile.ZipFile(archive) as zf:
        zf.extractall(target.parent)
        top = {name.split("/")[0] for name in zf.namelist() if name}
    root = target.parent / sorted(top)[0]
    root.rename(target)
    return target


def extract_crate(archive: Path, target: Path) -> Path:
    if target.exists():
        shutil.rmtree(target)
    target.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive) as tf:
        tf.extractall(target.parent)
    extracted = target.parent / archive.name.replace(".crate", "")
    extracted.rename(target)
    return target


def locked_version(lock: Path, name: str) -> str | None:
    text = read(lock)
    match = re.search(rf'name = "{re.escape(name)}"\nversion = "([^"]+)"', text)
    return match.group(1) if match else None


def git_rev(source: Path, url_fragment: str) -> str | None:
    """Find the pinned revision of a git dependency anywhere in the workspace."""
    manifests = [source / "Cargo.toml"] + sorted(source.glob("alvr/*/Cargo.toml"))
    pattern = re.compile(
        rf'git\s*=\s*"{re.escape(url_fragment)}"[^,\n]*,\s*rev\s*=\s*"([0-9a-f]+)"'
    )
    for manifest in manifests:
        if not manifest.exists():
            continue
        match = pattern.search(read(manifest))
        if match:
            return match.group(1)
    return None


def vendor_epaint(source: Path, cache: Path) -> None:
    """Vendor epaint at the version ALVR's lockfile pins and install the hook."""
    version = locked_version(source / "Cargo.lock", "epaint")
    if not version:
        raise PatchError("Cargo.lock has no epaint entry")

    archive = download(
        f"https://static.crates.io/crates/epaint/epaint-{version}.crate",
        cache / f"epaint-{version}.crate",
    )
    vendor = source / "vendor" / "epaint"
    if not vendor.exists():
        extract_crate(archive, vendor)

    fonts = vendor / "src" / "text" / "fonts.rs"
    text = read(fonts)
    if MARK not in text:
        anchor = "use epaint_default_fonts::"
        index = text.find(anchor)
        if index < 0:
            raise PatchError(f"{fonts}: cannot find the epaint_default_fonts import")
        line_end = text.find("\n", index)
        write(fonts, text[: line_end + 1] + "\n" + EPAINT_HOOK + "\n" + text[line_end + 1 :])
        text = read(fonts)

    old = "self.galley_cache.layout(&mut self.fonts, job)"
    if old in text:
        write(
            fonts,
            text.replace(
                old,
                f"// {MARK}: localize before the galley is cached, so a language switch\n"
                "        // takes effect on the next frame.\n"
                "        self.galley_cache.layout(&mut self.fonts, translated_job(job))",
                1,
            ),
        )

    mod_rs = vendor / "src" / "text" / "mod.rs"
    append_line(mod_rs, f"pub use fonts::{{set_text_translator, TextTranslator}}; // {MARK}")
    print(f"  epaint {version} patched")


def vendor_git_crates(source: Path, cache: Path) -> None:
    """Vendor the two git dependencies (git-over-https may be blocked for users)."""
    root_cargo = source / "Cargo.toml"
    text = read(root_cargo)
    changed = False

    schema_rev = git_rev(source, "https://github.com/alvr-org/settings-schema-rs")
    if schema_rev:
        target = source / "vendor" / "settings-schema"
        if not target.exists():
            archive = download(
                f"https://codeload.github.com/alvr-org/settings-schema-rs/zip/{schema_rev}",
                cache / f"settings-schema-{schema_rev}.zip",
            )
            extract_zip(archive, target)
            print("  settings-schema vendored")
        patch = (
            '[patch."https://github.com/alvr-org/settings-schema-rs"]\n'
            'settings-schema = { path = "vendor/settings-schema/settings-schema" }\n'
        )
        if "github.com/alvr-org/settings-schema-rs\"]" not in read(root_cargo):
            write(root_cargo, read(root_cargo).rstrip("\n") + "\n\n" + patch)
            text = read(root_cargo)
            changed = True
            print("  settings-schema patch added")
    else:
        print("  WARNING: settings-schema git revision not found; build may need network access")

    openxr_rev = git_rev(source, "https://github.com/Ralith/openxrs")
    if openxr_rev and 'github.com/Ralith/openxrs"]' not in read(root_cargo):
        target = source / "vendor" / "openxrs"
        if not target.exists():
            archive = download(
                f"https://codeload.github.com/Ralith/openxrs/zip/{openxr_rev}",
                cache / f"openxrs-{openxr_rev}.zip",
            )
            extract_zip(archive, target)
            print("  openxrs vendored")
        patch = (
            '[patch."https://github.com/Ralith/openxrs"]\n'
            'openxr = { path = "vendor/openxrs/openxr" }\n'
        )
        write(root_cargo, read(root_cargo).rstrip("\n") + "\n\n" + patch)
        changed = True
        print("  openxrs patch added")

    if changed:
        print("  git dependencies patched")


# ---------------------------------------------------------------------------

def vendor_openvr_headers(source: Path, cache: Path) -> None:
    """ALVR's `openvr` folder is a git submodule; zip downloads do not include it."""
    target = source / "openvr" / "headers"
    if (target / "openvr_driver.h").is_file():
        return
    archive = download(
        "https://codeload.github.com/ValveSoftware/openvr/zip/refs/heads/master",
        cache / "openvr-master.zip",
    )
    extract_dir = cache / "openvr-extract"
    if extract_dir.exists():
        shutil.rmtree(extract_dir)
    extract_dir.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(archive) as zf:
        zf.extractall(extract_dir)
    root = next(entry for entry in extract_dir.iterdir() if entry.is_dir())
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(root / "headers", target, dirs_exist_ok=True)
    print("  openvr headers vendored")


def apply(source: Path, payload: Path, do_epaint: bool = True) -> None:
    cache = payload.parent / "cache"
    cache.mkdir(parents=True, exist_ok=True)

    if not (source / "alvr").is_dir():
        raise PatchError(f"{source} does not look like an ALVR source tree")

    # 1. the i18n crate
    dest_i18n = source / "alvr" / "i18n"
    dest_i18n.mkdir(parents=True, exist_ok=True)
    shutil.copytree(payload / "alvr-i18n", dest_i18n, dirs_exist_ok=True)
    print("  alvr/i18n written")

    resources = source / "alvr" / "gui_common" / "resources"
    resources.mkdir(parents=True, exist_ok=True)
    for font in (payload / "resources").glob("*.ttf"):
        shutil.copy2(font, resources / font.name)
    print("  CJK fonts installed")

    # 2. workspace plumbing
    root_cargo = source / "Cargo.toml"
    if "alvr_i18n = { path" not in read(root_cargo):
        ensure_line_after_header(root_cargo, "workspace.dependencies", 'alvr_i18n = { path = "alvr/i18n" }')
        print("  workspace dependency added")

    if do_epaint:
        vendor_epaint(source, cache)
        ensure_line_after_header(root_cargo, "patch.crates-io", 'epaint = { path = "vendor/epaint" }')

    vendor_git_crates(source, cache)
    vendor_openvr_headers(source, cache)

    for crate in ("dashboard", "launcher"):
        cargo = source / "alvr" / crate / "Cargo.toml"
        if cargo.exists() and "alvr_i18n" not in read(cargo):
            ensure_line_after_header(cargo, "dependencies", "alvr_i18n.workspace = true")
            print(f"  alvr/{crate}: dependency added")

    # 3. install the translator in both binaries
    dashboard_main = source / "alvr" / "dashboard" / "src" / "main.rs"
    insert_before(
        dashboard_main,
        "    let (server_events_sender, server_events_receiver) = mpsc::channel();",
        DASHBOARD_MAIN_INIT,
        "dashboard translator init",
    )
    launcher_main = source / "alvr" / "launcher" / "src" / "main.rs"
    insert_after(launcher_main, "    let (ui_message_sender, ui_message_receiver)", LAUNCHER_MAIN_INIT, "launcher translator init")
    print("  translator installed")

    # 4. fonts
    theme = source / "alvr" / "gui_common" / "src" / "theme.rs"
    if MARK not in read(theme):
        insert_before(theme, "pub const ACCENT: Color32", THEME_FONTS, "CJK font setup")
        insert_after(theme, "pub fn set_theme(ctx: &Context) {", "\n    install_cjk_fonts(ctx);\n", "CJK font install")
    print("  theme patched")

    # 5. language selector in the settings tab
    settings = source / "alvr" / "dashboard" / "src" / "dashboard" / "components" / "settings.rs"
    if MARK not in read(settings):
        insert_before(
            settings,
            '        if self.selected_top_tab_id == "presets" {',
            SETTINGS_LANGUAGE,
            "language selector",
        )
    print("  settings tab patched")

    print("done")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", required=True, help="ALVR source tree to patch")
    parser.add_argument("--payload", default=str(Path(__file__).parent / "payload"))
    parser.add_argument("--no-epaint", action="store_true", help="skip vendoring epaint")
    args = parser.parse_args()

    try:
        apply(Path(args.source), Path(args.payload), do_epaint=not args.no_epaint)
    except PatchError as error:
        print(f"ERROR: {error}", file=sys.stderr)
        print("The localization patch did not apply cleanly to this ALVR version.", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
