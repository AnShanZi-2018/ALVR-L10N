"""Build one CJK subset font per language, sized to fit the ALVR font slot.

The patch-mode replaces the embedded "Ubuntu Light" blob in ALVR's exe, so every
per-language font must be smaller than that blob (< 361676 bytes for v20.14.1).
"""
import json
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

PACKS = Path(r"D:\ALVR_ZH\dist\languages")
OUT = Path(r"D:\ALVR_ZH\l10n_assets\fonts")
CHARS_FILE = Path(r"D:\ALVR_ZH\packs\font_chars.txt")

SC_SRC = Path(r"C:\Windows\Fonts\NotoSansSC-VF.ttf")
KR_SRC = Path(r"D:\ALVR_ZH\fonts_dl\noto-kr.ttf")

BASE = "".join(chr(c) for c in range(0x20, 0x7F)) + "·—…×÷±°→←↑↓≥≤≠²³µΩ▲▼◀▶★☆•○●□■◇◆　、。！？：；“”‘’（）《》【】「」『』①②③④⑤"


def pack_chars(ini_path: Path) -> set:
    text = ini_path.read_text(encoding="utf-8")
    chars = set(BASE)
    in_strings = False
    for line in text.splitlines():
        s = line.strip()
        if not s or s[0] in ";#":
            continue
        if s.startswith("["):
            in_strings = s.lower().startswith("[strings]")
            continue
        if not in_strings or "=" not in s:
            continue
        _, value = s.split("=", 1)
        chars |= set(value.replace("\\n", " "))
    return chars


def build(src: Path, out: Path, chars: set) -> int:
    font = TTFont(src, lazy=False)
    if "fvar" in font:
        instantiateVariableFont(font, {"wght": 400}, inplace=True, updateFontNames=True)

    options = subset.Options()
    options.glyph_names = False
    options.hinting = False
    options.desubroutinize = True
    options.notdef_outline = True
    options.name_IDs = ["*"]
    options.name_legacy = True
    options.layout_features = ["*"]
    options.drop_tables += ["DSIG"]

    subsetter = subset.Subsetter(options=options)
    subsetter.populate(text="".join(sorted(chars)))
    subsetter.subset(font)
    font.save(out)
    return out.stat().st_size


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    budget = 361676
    for ini in sorted(PACKS.glob("*.ini")):
        lang = ini.stem
        chars = pack_chars(ini)
        src = KR_SRC if lang == "ko-KR" else SC_SRC
        out = OUT / f"{lang}.ttf"
        size = build(src, out, chars)
        status = "OK" if size <= budget else "TOO BIG"
        print(f"{lang}: {size/1024:.0f} KB  ({len(chars)} chars, slot {budget/1024:.0f} KB)  {status}")


if __name__ == "__main__":
    main()
