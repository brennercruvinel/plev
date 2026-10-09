#!/usr/bin/env python3
"""Rewrite cargo commands written in docs and comments to the new packages.

`cargo run -p showcase --example regress` is `cargo run -p regress` now;
`-p sem` is `-p accord`. Examples that became crates name the crate, the
importer's examples name its binaries. Applied like textfix.py: whole
text files, and only the comments of .rs files.

    python3 xtask/scripts/cmdfix.py [--check] [files...]
"""
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pathfix  # noqa: E402
import textfix  # noqa: E402

DEMOS = {"snake": "arcade", "scene3d": "camera", "counter": "ticker", "input": "inputs",
         "mobile_input": "mobile", "sdf_shapes": "shapes", "lottie_player": "lottie",
         "monster_player": "player", "layers": "layers"}
EXAMPLES = [
    (r"-p showcase --example regress", "-p regress"),
    (r"-p showcase --example measure", "-p measure"),
    (r"-p showcase --example mirror", "-p mirroring"),
    (r"-p showcase --example demo", "-p uigallery --example videos"),
    (r"-p ide --bin ide-lab", "-p agenteval"),
    (r"-p ide --bin ide\b", "-p idestudio"),
    (r"-p claude-model --example propose", "-p intent"),
    (r"-p sem --example commit", "-p commit"),
    (r"-p parser --example transpile", "-p import --bin transpile"),
    (r"-p parser --example preview", "-p mockup"),
    (r"-p engine --example (?:lot2monsters|svg2monster)", "-p import --bin convert"),
    (r"-p engine --example text_probe", "-p engine --bin text_probe"),
    (r"--example (?:lot2monsters|svg2monster)\b", "-p import --bin convert"),
    (r"-p inspect --bin plev-inspect", "-p replay --bin plev-inspect"),
    (r"-p wire --bin plev", "-p socket --bin plev"),
] + [(rf"-p engine --example {old}\b", f"-p {new}") for old, new in DEMOS.items()]
PACKAGES = {"showcase": "uigallery", "ide": "idestudio", "sem": "accord", "uir": "plevir",
            "rec": "record", "wire": "socket", "inspect": "replay", "surface": "screen",
            "propose": "offers", "claude-model": "claude", "git": "gitdrv", "rope": "editor",
            "monster": "motion", "lot": "import", "svg": "import", "parser": "import",
            "narrate": "markup", "narrate-macro": "macros", "prime": "primes",
            "llm": "llmrun", "urnaui": "urnadbapp", "comps": "engine"}
RX_PKG = re.compile(r"(-p |--package )(" + "|".join(re.escape(k) for k in
                    sorted(PACKAGES, key=len, reverse=True)) + r")(?![\w-])")


def sub(text):
    for pat, rep in EXAMPLES:
        text = re.sub(pat, rep, text)
    return RX_PKG.sub(lambda m: m.group(1) + PACKAGES[m.group(2)], text)


def main(argv):
    check = "--check" in argv
    files = [a for a in argv if not a.startswith("--")] or textfix.files()
    n = 0
    for f in files:
        p = os.path.join(textfix.ROOT, f)
        src = open(p, encoding="utf-8").read()
        if f.endswith(".rs"):
            out, last = [], 0
            for kind, text, pos in pathfix.tokenize(src):
                if kind == "skip" and text.startswith(("//", "/*")):
                    new = sub(text)
                    if new != text:
                        out += [src[last:pos], new]
                        last = pos + len(text)
            out.append(src[last:])
            new_src = "".join(out)
        else:
            new_src = sub(src)
        if new_src != src:
            n += 1
            print("changed", f)
            if not check:
                open(p, "w", encoding="utf-8").write(new_src)
    print(n, "files changed")


if __name__ == "__main__":
    main(sys.argv[1:])
