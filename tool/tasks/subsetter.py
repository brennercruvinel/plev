#!/usr/bin/env python3
"""font-subset: the embedded Noto faces of the non-Latin scripts (task-74 phase 2)
and of emoji (phase 5).

Reads the Noto sources from a directory outside the repo, checks each one's
sha256, subsets it and writes the result to rust/engine/assets/fonts/,
next to the license (LICENSE-Noto-OFL.txt). Stdlib plus fontTools (tested
with 4.64.0), never the network. The sources stay out of the repo; only the
subsets are committed. docs/decisoes/renderizacao/0071-embedded-fonts-by-script.md has the why.

    tool/tasks/subsetter.py [--src DIR] [--out DIR] [--check]

    --src    the sources (default: target/fonts-noto at the repo root)
    --out    where the subsets go (default: rust/engine/assets/fonts)
    --check  subset into a temporary directory and compare byte for byte
             with --out; exit 1 when any file differs

Arabic, Hebrew and Devanagari keep every code point of the source (they
are small). Chinese, Japanese and Korean keep a set built from the
stdlib's legacy codecs, so the set is the same on every machine:

    SC  GB2312 level 1 (rows 16 to 55, 3,755 hanzi)
    JP  JIS X 0208 kana (rows 4 and 5) and level 1 (rows 16 to 47, 2,965 kanji)
    KR  KS X 1001 hangul syllables (rows 16 to 40, 2,350)

plus row 1 (punctuation and symbols) and row 3 (full-width digits and
letters) of the same standard. Every subset keeps GSUB, GPOS and GDEF, its
hinting and its .notdef outline. The whole ones keep every layout feature;
the CJK ones keep fontTools' default feature list (what a shaper applies
on its own, the Indic, Arabic and jamo forms included) minus the vertical
features, and drop the vertical metrics (vhea, vmtx, VORG): the engine sets
no vertical text. What goes is the alternates only a feature the engine
never turns on reaches (aalt, jp78, jp83, jp90, nlck, fwid, hwid, pwid,
ruby, hist, dlig). Running the script twice writes the same bytes.

Emoji come from Noto Color Emoji in its CBDT build (PNG bitmaps in one
109 ppem strike): swash 0.2.8, the rasterizer in Cargo.lock, draws CBDT and
COLR v0 but not the COLR v1 of the current Google Fonts build. The subset
is a fixed set written below: the Emoticons block, the face emoji of the
Supplemental block, a list of common symbols, the five skin tones on a list
of hands and people, and the most used family and profession ZWJ
sequences. Its cmap holds only those code points, and the script fails if
any of them is in another embedded face besides U+2764 and U+200D (see
EMOJI_SHARED): cosmic-text tries emoji faces first in its last fallback
step, so a shared code point would move from the other face to the emoji.
Its license goes to LICENSE-NotoColorEmoji-OFL.txt.
"""

import argparse
import hashlib
import sys
import tempfile
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parents[2]
# The embedded faces; the emoji guard reads the Latin ones from here.
ASSETS = ROOT / "rust/engine/assets/fonts"

# The static hinted TTFs of notofonts.github.io and the region subsets of
# noto-cjk (Sans/SubsetOTF, release 2.004). The sha256 pins the bytes.
NOTOFONTS = "https://notofonts.github.io"
NOTO_CJK = "https://github.com/notofonts/noto-cjk/raw/main/Sans/SubsetOTF"

SOURCES = [
    ("NotoSansArabic-Regular.ttf", "whole",
     "bdff3e5659d67e67def05b33f749683b9376ae819d65d3dd62ac4640b3aaef48",
     f"{NOTOFONTS}/arabic/fonts/NotoSansArabic/hinted/ttf/NotoSansArabic-Regular.ttf"),
    ("NotoSansArabic-Bold.ttf", "whole",
     "4e5462d2e8be880317b9f49b5b2da109ddb6a3563d91cc604b67f3535832a555",
     f"{NOTOFONTS}/arabic/fonts/NotoSansArabic/hinted/ttf/NotoSansArabic-Bold.ttf"),
    ("NotoSansHebrew-Regular.ttf", "whole",
     "cdefaf8efd47045f6820928eba84db5bed7557539328952b5f828315485e02ee",
     f"{NOTOFONTS}/hebrew/fonts/NotoSansHebrew/hinted/ttf/NotoSansHebrew-Regular.ttf"),
    ("NotoSansHebrew-Bold.ttf", "whole",
     "da9226e886c245a7e11673c24dec82bded64d8574c1e1f03983bf89297d2aaa8",
     f"{NOTOFONTS}/hebrew/fonts/NotoSansHebrew/hinted/ttf/NotoSansHebrew-Bold.ttf"),
    ("NotoSansDevanagari-Regular.ttf", "whole",
     "4e3c66638958c3e2ab5d37f47a8deb89fffeb7be9985c665a519bbc7ba762313",
     f"{NOTOFONTS}/devanagari/fonts/NotoSansDevanagari/hinted/ttf/NotoSansDevanagari-Regular.ttf"),
    ("NotoSansDevanagari-Bold.ttf", "whole",
     "6a09c8d797cfc803d32cdc731e809424d74cbaff59f503de34ade421a08e5bc2",
     f"{NOTOFONTS}/devanagari/fonts/NotoSansDevanagari/hinted/ttf/NotoSansDevanagari-Bold.ttf"),
    ("NotoSansSC-Regular.otf", "sc",
     "faa6c9df652116dde789d351359f3d7e5d2285a2b2a1f04a2d7244df706d5ea9",
     f"{NOTO_CJK}/SC/NotoSansSC-Regular.otf"),
    ("NotoSansSC-Bold.otf", "sc",
     "c6cb5a93abaa9edc8ee7463b7ebb7f42d618d40e6ed2f7a5371c97b0b64767c0",
     f"{NOTO_CJK}/SC/NotoSansSC-Bold.otf"),
    ("NotoSansJP-Regular.otf", "jp",
     "dff723ba59d57d136764a04b9b2d03205544f7cd785a711442d6d2d085ac5073",
     f"{NOTO_CJK}/JP/NotoSansJP-Regular.otf"),
    ("NotoSansJP-Bold.otf", "jp",
     "1b0edfb500b73a4fa8a4fcaae1bbbd403994e08e73e3e0da37e70d3853f42c5f",
     f"{NOTO_CJK}/JP/NotoSansJP-Bold.otf"),
    ("NotoSansKR-Regular.otf", "kr",
     "69975a0ac8472717870aefeab0a4d52739308d90856b9955313b2ad5e0148d68",
     f"{NOTO_CJK}/KR/NotoSansKR-Regular.otf"),
    ("NotoSansKR-Bold.otf", "kr",
     "5a6ceb287ed2fc6cfc6213144ebea68cbd94b20fc9eb873d8486493bf02d9bda",
     f"{NOTO_CJK}/KR/NotoSansKR-Bold.otf"),
]

# The OFL text shipped with the sources (the notofonts OFL.txt).
LICENSE_SOURCE = ("OFL.txt", "SIL Open Font License 1.1, as shipped with the Noto sources")
LICENSE_OUT = "LICENSE-Noto-OFL.txt"

# Noto Color Emoji v2.051, the CBDT/CBLC build (bitmaps), and its license.
EMOJI = ("NotoColorEmoji-CBDT.ttf",
         "72a635cb3d2f3524c51620cdde406b217204e8a6a06c6a096ff8ed4b5fd6e27b",
         "https://raw.githubusercontent.com/googlefonts/noto-emoji/v2.051/fonts/NotoColorEmoji.ttf")
EMOJI_LICENSE_SOURCE = "OFL-NotoColorEmoji.txt"
EMOJI_LICENSE_OUT = "LICENSE-NotoColorEmoji-OFL.txt"


def span(first, last):
    return list(range(first, last + 1))


ZWJ = 0x200D
# Emoji shown as themselves, one code point each.
EMOJI_SINGLES = (
    # the Emoticons block, and the faces of Supplemental Symbols and Pictographs
    span(0x1F600, 0x1F64F)
    + span(0x1F910, 0x1F92F) + span(0x1F970, 0x1F976) + span(0x1F978, 0x1F97A)
    + [0x1F9D0] + span(0x1FAE0, 0x1FAE8)
    # hearts
    + [0x2764, 0x1F9E1, 0x1F49B, 0x1F49A, 0x1F499, 0x1F49C, 0x1F5A4, 0x1F90D,
       0x1F90E, 0x1F494, 0x1F495, 0x1F49E, 0x1F493, 0x1F497, 0x1F496, 0x1F498,
       0x1F49D, 0x1F48B]
    # marks and signs
    + [0x2705, 0x274C, 0x274E, 0x2753, 0x2757, 0x2B50, 0x1F31F, 0x2728, 0x1F525,
       0x1F4AF, 0x1F4A5, 0x1F4AB, 0x1F4A2, 0x1F4A6, 0x1F4A8, 0x1F4A4, 0x1F4A1,
       0x1F6AB, 0x26D4, 0x1F534, 0x1F7E0, 0x1F7E1, 0x1F7E2, 0x1F535, 0x1F7E3,
       0x26AA, 0x26AB, 0x2795, 0x2796, 0x2797, 0x1F195, 0x1F197, 0x1F198,
       0x1F199, 0x1F193, 0x1F504, 0x1F501]
    # objects
    + [0x1F4CC, 0x1F4CE, 0x1F512, 0x1F513, 0x1F511, 0x1F514, 0x1F515, 0x1F4E3,
       0x1F4E2, 0x1F4AC, 0x1F4AD, 0x23F0, 0x231B, 0x23F3, 0x1F4C5, 0x1F4C6,
       0x1F4C8, 0x1F4C9, 0x1F4CA, 0x1F4DD, 0x1F4C1, 0x1F4C2, 0x1F4BB, 0x1F4F1,
       0x1F4E6, 0x1F517, 0x1F50D, 0x1F50E, 0x1F527, 0x1F528, 0x1F9EA, 0x1F52C,
       0x1F4E7, 0x1F4E9, 0x1F4F7, 0x1F3A8, 0x1F3B5, 0x1F3B6, 0x1F3AE, 0x1F4DA,
       0x1F4D6, 0x1F4B0, 0x1F4B5, 0x1F4B3, 0x1F3C6, 0x1F947, 0x1F948, 0x1F949,
       0x1F3AF, 0x1F451, 0x1F48E, 0x1F4BC, 0x1F393, 0x1F3EB, 0x1F373, 0x1F680,
       0x1F6A7, 0x1F6A8, 0x1F3C1, 0x1F6A9, 0x1F9E0, 0x1F440, 0x1F47B, 0x1F480,
       0x1F47D, 0x1F4A9, 0x1F389, 0x1F38A, 0x1F381, 0x1F388, 0x1F382]
    # nature and food
    + [0x1F30D, 0x1F30E, 0x1F30F, 0x1F31E, 0x1F319, 0x26C5, 0x26A1, 0x1F308,
       0x2614, 0x1F30A, 0x1F331, 0x1F332, 0x1F333, 0x1F335, 0x1F337, 0x1F338,
       0x1F339, 0x1F33B, 0x1F340, 0x1F341, 0x1F342, 0x1F33E, 0x1F344, 0x2615,
       0x1F355, 0x1F354, 0x1F35F, 0x1F37A, 0x1F37B, 0x1F377, 0x1F370, 0x1F36A,
       0x1F34E, 0x1F34C, 0x1F353, 0x1F951]
    # animals
    + [0x1F436, 0x1F431, 0x1F42D, 0x1F430, 0x1F98A, 0x1F43B, 0x1F43C, 0x1F428,
       0x1F42F, 0x1F981, 0x1F42E, 0x1F437, 0x1F438, 0x1F435, 0x1F414, 0x1F427,
       0x1F426, 0x1F986, 0x1F989, 0x1F41D, 0x1F41B, 0x1F98B, 0x1F40C, 0x1F41E,
       0x1F422, 0x1F40D, 0x1F419, 0x1F980, 0x1F433, 0x1F42C, 0x1F988, 0x1F984]
    # family and couples as one code point
    + [0x1F46A, 0x1F491, 0x1F48F]
)
# The five skin tones (Fitzpatrick modifiers), alone and after each of
# EMOJI_TONED.
EMOJI_TONES = span(0x1F3FB, 0x1F3FF)
EMOJI_TONED = [
    # hands
    0x1F44B, 0x1F91A, 0x1F590, 0x270B, 0x1F596, 0x1F44C, 0x1F90C, 0x1F90F,
    0x270C, 0x1F91E, 0x1F91F, 0x1F918, 0x1F919, 0x1F448, 0x1F449, 0x1F446,
    0x1F447, 0x261D, 0x1F44D, 0x1F44E, 0x270A, 0x1F44A, 0x1F44F, 0x1F64C,
    0x1F450, 0x1F932, 0x1F64F, 0x270D, 0x1F485, 0x1F4AA,
    # people and gestures
    0x1F476, 0x1F9D2, 0x1F466, 0x1F467, 0x1F9D1, 0x1F468, 0x1F469, 0x1F9D3,
    0x1F474, 0x1F475, 0x1F645, 0x1F646, 0x1F64B, 0x1F926, 0x1F937,
]
MAN, WOMAN, ADULT, BOY, GIRL, CHILD = 0x1F468, 0x1F469, 0x1F9D1, 0x1F466, 0x1F467, 0x1F9D2


def joined(*code_points):
    """A ZWJ sequence: the code points with U+200D between them."""
    out = []
    for cp in code_points:
        if out:
            out.append(ZWJ)
        out.append(cp)
    return out


EMOJI_FAMILIES = [
    joined(MAN, WOMAN, BOY), joined(MAN, WOMAN, GIRL), joined(MAN, WOMAN, GIRL, BOY),
    joined(MAN, WOMAN, BOY, BOY), joined(MAN, WOMAN, GIRL, GIRL),
    joined(WOMAN, WOMAN, BOY), joined(WOMAN, WOMAN, GIRL),
    joined(MAN, MAN, BOY), joined(MAN, MAN, GIRL),
    joined(WOMAN, BOY), joined(WOMAN, GIRL), joined(MAN, BOY), joined(MAN, GIRL),
    joined(ADULT, ADULT, CHILD), joined(ADULT, CHILD),
]
# Technologist, scientist, teacher, cook, artist, astronaut, mechanic,
# office worker, student, farmer: person, man and woman.
EMOJI_TOOLS = [0x1F4BB, 0x1F52C, 0x1F3EB, 0x1F373, 0x1F3A8, 0x1F680, 0x1F527,
               0x1F4BC, 0x1F393, 0x1F33E]
EMOJI_PROFESSIONS = [joined(who, tool) for who in (ADULT, MAN, WOMAN) for tool in EMOJI_TOOLS]
# Code points the emoji face may share with another embedded face: the
# heart, which Inter carries (Inclusive Sans does not, so a heart in
# Inclusive Sans text becomes the emoji), and the joiner, which a shaper
# hides and so never falls back.
EMOJI_SHARED = {0x2764, ZWJ}

# Past this a subset is a decision, not a build (the task's stop rule).
MAX_BYTES = 3 * 1024 * 1024


def rows(codec, first, last):
    """Every character the codec decodes in rows first..last of a 94x94
    EUC table (row r is lead byte 0xA0 + r, cells 0xA1..0xFE)."""
    out = []
    for row in range(first, last + 1):
        for cell in range(0xA1, 0xFF):
            try:
                text = bytes([0xA0 + row, cell]).decode(codec)
            except UnicodeDecodeError:
                continue
            out.extend(ord(c) for c in text)
    return out


def charset(kind):
    """The code points one subset keeps, and how many are the script's
    own letters (the count the ADR quotes)."""
    if kind == "sc":
        letters = rows("gb2312", 16, 55)
        expected = 3755
        codec = "gb2312"
    elif kind == "jp":
        letters = rows("euc_jp", 4, 5) + rows("euc_jp", 16, 47)
        expected = 83 + 86 + 2965
        codec = "euc_jp"
    elif kind == "kr":
        letters = rows("euc_kr", 16, 40)
        expected = 2350
        codec = "euc_kr"
    else:
        raise ValueError(kind)
    if len(letters) != expected:
        sys.exit(f"{codec}: {len(letters)} letters, expected {expected}")
    punctuation = rows(codec, 1, 1) + rows(codec, 3, 3)
    return sorted(set(letters + punctuation)), len(letters)


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


# Vertical-text features, dropped with the vertical metrics.
VERTICAL = {"valt", "vchw", "vert", "vhal", "vkrn", "vpal", "vrt2"}


def options(kind):
    opts = subset.Options()
    if kind == "whole":
        opts.layout_features = ["*"]
    else:
        opts.layout_features = [f for f in opts.layout_features if f not in VERTICAL]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    opts.glyph_names = False
    opts.hinting = True
    opts.recalc_timestamp = False
    if kind != "whole":
        opts.drop_tables += ["vhea", "vmtx", "VORG"]
    return opts


def subset_one(src, dst, kind):
    opts = options(kind)
    font = subset.load_font(str(src), opts)
    if kind == "whole":
        unicodes = sorted(font.getBestCmap())
        letters = len(unicodes)
    else:
        unicodes, letters = charset(kind)
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=unicodes)
    sub.subset(font)
    subset.save_font(font, str(dst), opts)
    font.close()
    kept = len(TTFont(str(dst), lazy=True).getBestCmap())
    return letters, kept


def license_text(src_dir):
    """The copyright and trademark lines of every source family (their
    name records 0 and 7), then the OFL text as shipped."""
    lines = []
    for name, _, _, _ in SOURCES:
        names = TTFont(str(src_dir / name), lazy=True)["name"]
        for record in (0, 7):
            line = names.getDebugName(record)
            if line and line not in lines:
                lines.append(line)
    head = "\n".join(lines)
    note = (
        "The files NotoSans*-Regular and NotoSans*-Bold in this directory are "
        "subsets of the\nNoto fonts above, made by tool/tasks/subsetter.py "
        "(docs/decisoes/renderizacao/0071-embedded-fonts-by-script.md).\n"
    )
    body = (src_dir / LICENSE_SOURCE[0]).read_text(encoding="utf-8")
    return f"{head}\n\n{note}\n{body}"


def checked_source(src_dir, name, digest, url):
    src = src_dir / name
    if not src.is_file():
        sys.exit(f"missing source {src} (from {url})")
    got = sha256(src)
    if got != digest:
        sys.exit(f"{name}: sha256 {got}, expected {digest} (from {url})")
    return src


def emoji_ligatures(font, sequences):
    """The glyph each sequence ligates to in the font's GSUB (every
    sequence of the source is one ligature of its lookup type 4)."""
    cmap = font.getBestCmap()
    table = {}
    for lookup in font["GSUB"].table.LookupList.Lookup:
        if lookup.LookupType != 4:
            continue
        for sub in lookup.SubTable:
            for first, ligatures in sub.ligatures.items():
                for lig in ligatures:
                    table.setdefault(tuple([first] + lig.Component), lig.LigGlyph)
    glyphs = []
    for seq in sequences:
        names = tuple(cmap.get(cp) for cp in seq)
        glyph = table.get(names)
        if glyph is None:
            sys.exit("no ligature for " + " ".join(f"U+{cp:04X}" for cp in seq))
        glyphs.append(glyph)
    return glyphs


def emoji_guard(unicodes, out_dir):
    """Fail when an emoji code point is in another embedded face (the Latin
    faces in the repo, the Noto subsets just built) and not in EMOJI_SHARED."""
    faces = [out_dir / name for name, *_ in SOURCES]
    faces += [
        p for p in sorted(ASSETS.glob("*.[ot]tf"))
        if p.name not in {name for name, *_ in SOURCES} | {EMOJI[0]}
    ]
    wanted = set(unicodes) - EMOJI_SHARED
    for face in faces:
        shared = sorted(wanted & set(TTFont(str(face), lazy=True).getBestCmap()))
        if shared:
            sys.exit(f"{face.name} also maps " + " ".join(f"U+{cp:04X}" for cp in shared))


def subset_emoji(src, dst, out_dir):
    """The emoji subset: EMOJI_SINGLES and the skin tones by code point,
    the toned, family and profession sequences by their ligature glyph.
    No layout closure, so the kept rules are exactly those sequences."""
    unicodes = sorted(set(EMOJI_SINGLES + EMOJI_TONES + EMOJI_TONED + EMOJI_TOOLS + [ZWJ]))
    sequences = (
        [[base, tone] for base in EMOJI_TONED for tone in EMOJI_TONES]
        + EMOJI_FAMILIES + EMOJI_PROFESSIONS
    )
    emoji_guard(unicodes, out_dir)
    opts = subset.Options()
    opts.layout_features = ["*"]
    opts.layout_closure = False
    opts.name_IDs = ["*"]
    opts.glyph_names = False
    opts.recalc_timestamp = False
    opts.drop_tables += ["vhea", "vmtx"]
    font = subset.load_font(str(src), opts)
    missing = [cp for cp in unicodes if cp not in font.getBestCmap()]
    if missing:
        sys.exit("not in the source: " + " ".join(f"U+{cp:04X}" for cp in missing))
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=unicodes, glyphs=emoji_ligatures(font, sequences))
    sub.subset(font)
    subset.save_font(font, str(dst), opts)
    font.close()
    kept = TTFont(str(dst), lazy=True)
    return len(unicodes) + len(sequences), len(kept.getBestCmap()), len(kept.getGlyphOrder())


def emoji_license_text(src_dir):
    """The copyright and trademark lines of the emoji source, then its OFL
    text as shipped."""
    names = TTFont(str(src_dir / EMOJI[0]), lazy=True)["name"]
    lines = [names.getDebugName(record) for record in (0, 7)]
    head = "\n".join(line for line in lines if line)
    note = (
        f"The file {EMOJI[0]} in this directory is a subset of Noto Color Emoji "
        "v2.051\n(CBDT build), made by tool/tasks/subsetter.py "
        "(docs/decisoes/renderizacao/0071-embedded-fonts-by-script.md).\n"
    )
    body = (src_dir / EMOJI_LICENSE_SOURCE).read_text(encoding="utf-8")
    return f"{head}\n\n{note}\n{body}"


def build(src_dir, out_dir):
    out_dir.mkdir(parents=True, exist_ok=True)
    report = []
    for name, kind, digest, url in SOURCES:
        src = checked_source(src_dir, name, digest, url)
        dst = out_dir / name
        letters, kept = subset_one(src, dst, kind)
        size = dst.stat().st_size
        if size > MAX_BYTES:
            sys.exit(f"{name}: {size} bytes, past the {MAX_BYTES} byte limit")
        report.append((name, src.stat().st_size, size, letters, kept))
    (out_dir / LICENSE_OUT).write_text(license_text(src_dir), encoding="utf-8")

    name, digest, url = EMOJI
    src = checked_source(src_dir, name, digest, url)
    dst = out_dir / name
    emoji, kept, glyphs = subset_emoji(src, dst, out_dir)
    size = dst.stat().st_size
    if size > MAX_BYTES:
        sys.exit(f"{name}: {size} bytes, past the {MAX_BYTES} byte limit")
    report.append((name, src.stat().st_size, size, emoji, kept))
    (out_dir / EMOJI_LICENSE_OUT).write_text(emoji_license_text(src_dir), encoding="utf-8")
    return report, glyphs


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--src", type=Path, default=ROOT / "target" / "fonts-noto")
    parser.add_argument("--out", type=Path, default=ASSETS)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()

    if args.check:
        with tempfile.TemporaryDirectory() as tmp:
            build(args.src, Path(tmp))
            names = [name for name, *_ in SOURCES] + [LICENSE_OUT, EMOJI[0], EMOJI_LICENSE_OUT]
            differ = [
                n for n in names
                if (Path(tmp) / n).read_bytes() != (args.out / n).read_bytes()
            ]
        if differ:
            sys.exit("differ from a fresh build: " + ", ".join(differ))
        print(f"{len(names)} files match a fresh build")
        return

    report, glyphs = build(args.src, args.out)
    total = 0
    print("file | source bytes | subset bytes | letters (emoji) | code points kept")
    for name, before, after, letters, kept in report:
        total += after
        print(f"{name} | {before} | {after} | {letters} | {kept}")
    print(f"total | | {total}")
    print(f"{EMOJI[0]}: {glyphs} glyphs")


if __name__ == "__main__":
    main()
