"""Build the fonts embedded in ALVR-L10N's own UI (Chinese + Japanese/Korean names)."""
import re
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

APP = Path(r"D:\ALVR_ZH\l10n_assets\app")
SC_SRC = Path(r"C:\Windows\Fonts\NotoSansSC-VF.ttf")
KR_SRC = Path(r"D:\ALVR_ZH\fonts_dl\noto-kr.ttf")

BASE = "".join(chr(c) for c in range(0x20, 0x7F)) + "、。，：；！？（）【】「」《》—…·×←→↑↓"


def source_chars() -> set:
    chars = set(BASE)
    for path in (APP / "src").glob("*.rs"):
        chars |= set(path.read_text(encoding="utf-8"))
    for path in (APP / "languages").glob("*.ini") if (APP / "languages").exists() else []:
        chars |= set(path.read_text(encoding="utf-8"))
    # drop control characters / whitespace that would confuse the subsetter
    return {c for c in chars if ord(c) >= 0x20 and c not in "\u200b"}


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
    out_dir = APP / "assets"
    out_dir.mkdir(parents=True, exist_ok=True)
    chars = source_chars()
    hangul = {c for c in chars if 0xAC00 <= ord(c) <= 0xD7A3 or 0x1100 <= ord(c) <= 0x11FF}

    sc = chars - hangul
    size_sc = build(SC_SRC, out_dir / "ui-sc.ttf", sc)
    print(f"ui-sc.ttf: {size_sc / 1024:.0f} KB ({len(sc)} chars)")

    size_kr = build(KR_SRC, out_dir / "ui-kr.ttf", hangul | set("가나다라마바사아자차카타파하한국어"))
    print(f"ui-kr.ttf: {size_kr / 1024:.0f} KB ({len(hangul)} hangul chars)")


if __name__ == "__main__":
    main()
