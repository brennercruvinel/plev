#!/usr/bin/env python3
"""figma-fixture: the Figma fixture of the parser (task-71 phase 2).

Reads a .fig file (a zip holding `canvas.fig` in the fig-kiwi format) and
writes ONE screen as the Figma REST API export shapes it (`document` with
`children`, `type`, `name`, `absoluteBoundingBox`, `layoutMode`,
`itemSpacing`, `padding*`, `fills`, `cornerRadius`, `characters`, `style`),
only what the importer (crates/parser/src/figma.rs) reads, one key per line
and `children` last, so the importer can give every construct its line.

    script/figma-fixture <file.fig> [out.json]
        [--page "Dark Theme"] [--section settings] [--frame settings]

Default out: crates/parser/fixtures/figma/settings.json. Python 3.14 or
later (compression.zstd), stdlib only, never the network. The .fig stays
outside the repo; only the JSON is committed.

fig-kiwi: the header `fig-kiwi`, a u32 version, then chunks of
`u32 size + bytes`: the kiwi schema (raw deflate), then the message
(zstd, or raw deflate in older files). The message's `nodeChanges` are the
nodes, each with its `guid` and its `parentIndex` (parent guid and a
fractional `position` that orders siblings, back to front).

Instances: an INSTANCE gets a copy of its SYMBOL's children
(`symbolData.symbolID`), recursively, with ids `I<instance>;<child>` as
the API writes them. Each override (`symbolData.symbolOverrides`) matches
by its `guidPath`, one guid per level (the node's `guid` or its
`overrideKey`), and sets the fields listed in APPLIED;
`derivedSymbolData` gives the size and position of the children of a
resized instance. What is not applied (another field, or a path that
matches no layer) is written in the instance's `unappliedOverrides`, for
the importer to report. A color bound to a variable is the variable's value
in the mode in force (`variableModeBySetMap`, nearest up the tree), as the
design tool shows it; a library's variable, which the file does not hold,
is marked `"variable": "unresolved"`. Images: only the paint's type, never
the bytes.
Hidden paints and hidden effects are left out (they draw nothing). Not
covered: vector geometry, masks, constraints, component properties,
variables bound to anything but a color, and a library's variables.
"""

import argparse
import copy
import io
import json
import math
import struct
import sys
import zipfile
import zlib

try:
    from compression import zstd
except ImportError:  # pragma: no cover
    sys.exit("figma-fixture needs python 3.14 or later (compression.zstd)")

OUT = "crates/parser/fixtures/figma/settings.json"

# override fields the export applies; any other is reported as unapplied
APPLIED = {
    "visible", "opacity", "size", "name", "fillPaints", "strokePaints",
    "strokeWeight", "strokeAlign", "textData", "fontSize", "fontName",
    "lineHeight", "letterSpacing", "cornerRadius", "effects",
    "rectangleTopLeftCornerRadius", "rectangleTopRightCornerRadius",
    "rectangleBottomLeftCornerRadius", "rectangleBottomRightCornerRadius",
}

TYPES = {"SYMBOL": "COMPONENT", "ROUNDED_RECTANGLE": "RECTANGLE"}
WEIGHTS = {
    "thin": 100, "extralight": 200, "ultralight": 200, "light": 300,
    "regular": 400, "book": 400, "medium": 500, "semibold": 600,
    "demibold": 600, "bold": 700, "extrabold": 800, "heavy": 800,
    "black": 900,
}


class Reader:
    """The kiwi wire format: varints, zigzag ints, a byte-rotated float."""

    def __init__(self, data):
        self.b, self.i = data, 0

    def byte(self):
        v = self.b[self.i]
        self.i += 1
        return v

    def uint(self, bits=35):
        v = shift = 0
        while True:
            c = self.byte()
            v |= (c & 127) << shift
            shift += 7
            if c < 128 or shift >= bits:
                return v

    def int(self):
        v = self.uint()
        return ~(v >> 1) if v & 1 else v >> 1

    def int64(self):
        v = self.uint(70)
        return ~(v >> 1) if v & 1 else v >> 1

    def uint64(self):
        return self.uint(70)

    def float(self):
        first = self.byte()
        if first == 0:
            return 0.0
        rest = self.b[self.i:self.i + 3]
        self.i += 3
        bits = first | rest[0] << 8 | rest[1] << 16 | rest[2] << 24
        bits = (bits << 23 | bits >> 9) & 0xFFFFFFFF
        return struct.unpack("<f", struct.pack("<I", bits))[0]

    def string(self):
        j = self.b.index(0, self.i)
        v = self.b[self.i:j].decode("utf8", "replace")
        self.i = j + 1
        return v


PRIMITIVES = {-1: "bool", -2: "byte", -3: "int", -4: "uint", -5: "float",
              -6: "string", -7: "int64", -8: "uint64"}


def schema(data):
    r = Reader(data)
    defs = []
    for _ in range(r.uint()):
        name, kind, fields = r.string(), r.byte(), []
        for _ in range(r.uint()):
            fields.append((r.string(), r.int(), r.byte(), r.uint()))
        defs.append((name, kind, fields))
    return defs


def decode(defs, r, index):
    _, kind, fields = defs[index]
    if kind == 0:  # enum
        v = r.uint()
        return next((f[0] for f in fields if f[3] == v), v)
    if kind == 1:  # struct
        return {f[0]: value(defs, r, f) for f in fields}
    out, by_id = {}, {f[3]: f for f in fields}
    while True:  # message
        fid = r.uint()
        if fid == 0:
            return out
        f = by_id[fid]
        out[f[0]] = value(defs, r, f)


def one(defs, r, t):
    if t < 0:
        p = PRIMITIVES[t]
        if p == "bool":
            return bool(r.byte())
        if p == "byte":
            return r.byte()
        return getattr(r, p)()
    return decode(defs, r, t)


def value(defs, r, field):
    _, t, is_array, _ = field
    if is_array:
        if t == -2:  # a byte array: a blob, skipped
            n = r.uint()
            r.i += n
            return {"bytes": n}
        return [one(defs, r, t) for _ in range(r.uint())]
    return one(defs, r, t)


def load(path):
    with zipfile.ZipFile(path) as z:
        data = z.read("canvas.fig")
    if data[:8] != b"fig-kiwi":
        sys.exit(f"{path}: canvas.fig is not fig-kiwi")
    off, chunks = 12, []
    while off < len(data):
        n = struct.unpack("<I", data[off:off + 4])[0]
        chunks.append(data[off + 4:off + 4 + n])
        off += 4 + n

    def unz(c):
        if c[:4] == b"\x28\xb5\x2f\xfd":
            return zstd.decompress(c)
        return zlib.decompress(c, -15)

    defs = schema(unz(chunks[0]))
    root = next(i for i, d in enumerate(defs) if d[0] == "Message")
    return decode(defs, Reader(unz(chunks[1])), root)


def gid(g):
    return f"{g['sessionID']}:{g['localID']}"


class Canvas:
    def __init__(self, message):
        self.nodes = message["nodeChanges"]
        self.by_id = {gid(n["guid"]): n for n in self.nodes}
        self.kids = {}
        for n in self.nodes:
            p = n.get("parentIndex")
            if p:
                self.kids.setdefault(gid(p["guid"]), []).append(n)
        for v in self.kids.values():
            v.sort(key=lambda n: n["parentIndex"].get("position", ""))

    def children(self, node):
        return self.kids.get(gid(node["guid"]), [])

    def find(self, parent, pred, what):
        found = next((c for c in self.children(parent) if pred(c)), None)
        if found is None:
            sys.exit(f"figma-fixture: no {what}")
        return found


def num(v, places=2):
    r = round(float(v), places)
    return int(r) if r == int(r) else r


def modes_of(node, modes):
    """The variable modes in force at `node`: its parent's, updated by its
    own `variableModeBySetMap` (local sets only)."""
    entries = (node.get("variableModeBySetMap") or {}).get("entries", [])
    if not entries:
        return modes
    out = dict(modes)
    for e in entries:
        set_id = e.get("variableSetID", {})
        if "guid" in set_id:
            out[gid(set_id["guid"])] = gid(e["variableModeID"])
    return out


class Paints:
    """Paints as the API writes them. A color bound to a variable is the
    variable's value in the mode in force (the nearest `variableModeBySet
    Map` up the tree, else the set's first mode), as a design tool shows
    it; a variable the file does not hold (a library's) stays unresolved and
    is marked, for the importer to report."""

    def __init__(self, canvas):
        self.by_id = canvas.by_id

    def variable(self, alias, modes, depth=0):
        g = alias.get("guid")
        v = self.by_id.get(gid(g)) if g else None
        if v is None or v.get("type") != "VARIABLE" or depth > 8:
            return None
        set_id = v.get("variableSetID", {})
        mode = modes.get(gid(set_id["guid"])) if "guid" in set_id else None
        entries = (v.get("variableDataValues") or {}).get("entries", [])
        e = next((e for e in entries if gid(e["modeID"]) == mode), entries[0] if entries else None)
        value = (e or {}).get("variableData", {}).get("value", {})
        if "colorValue" in value:
            return value["colorValue"]
        if "alias" in value:
            return self.variable(value["alias"], modes, depth + 1)
        return None

    def one(self, p, modes):
        if p.get("visible") is False:
            return None
        out = {"type": p.get("type", "SOLID")}
        if out["type"] == "SOLID":
            c = p.get("color", {})
            bound = p.get("colorVar")
            if bound and "alias" in bound.get("value", {}):
                resolved = self.variable(bound["value"]["alias"], modes)
                if resolved is None:
                    out["variable"] = "unresolved"
                else:
                    c = resolved
            out["color"] = {k: num(c.get(k, 0), 4) for k in "rgba"}
        if p.get("opacity", 1) != 1:
            out["opacity"] = num(p["opacity"], 4)
        return out

    def all(self, ps, modes):
        return [q for q in (self.one(p, modes) for p in ps or []) if q]


def effects(es):
    out = []
    for e in es or []:
        if e.get("visible") is False:
            continue
        item = {"type": e.get("type")}
        if "radius" in e:
            item["radius"] = num(e["radius"])
        out.append(item)
    return out


def weight(style):
    words = style.lower().replace("italic", "").replace(" ", "").replace("-", "")
    return WEIGHTS.get(words, 400)


class Scope:
    """The overrides and derived layout of one instance. A `guidPath` has
    one guid per instance level: the nested instances it goes through (at
    any depth of their component), then the layer (at any depth of the
    last one). `chain` holds the nested instances crossed so far."""

    def __init__(self, instance):
        sd = instance.get("symbolData", {})
        self.overrides = [
            {"path": [gid(g) for g in o["guidPath"]["guids"]],
             "fields": {k: v for k, v in o.items() if k != "guidPath"},
             "used": False}
            for o in sd.get("symbolOverrides", []) or []
        ]
        self.derived = [
            {"path": [gid(g) for g in d["guidPath"]["guids"]], "fields": d}
            for d in instance.get("derivedSymbolData", []) or []
        ]
        self.chain = []

    def into(self, instance_keys):
        s = copy.copy(self)
        s.chain = self.chain + [instance_keys]
        return s

    def matching(self, items, node_keys):
        return [
            it for it in items
            if len(it["path"]) == len(self.chain) + 1
            and all(p in k for p, k in zip(it["path"], self.chain))
            and it["path"][-1] in node_keys
        ]

    def unapplied(self):
        out = []
        for o in self.overrides:
            fields = sorted(k for k in o["fields"] if not o["used"] or k not in APPLIED)
            if fields:
                out.append({"path": ";".join(o["path"]), "fields": fields,
                            "matched": o["used"]})
        return out


def keys(node):
    k = {gid(node["guid"])}
    if node.get("overrideKey"):
        k.add(gid(node["overrideKey"]))
    return k


class Writer:
    def __init__(self, canvas):
        self.canvas = canvas
        self.paints = Paints(canvas)

    def node(self, n, parent_xy, inst_path, scopes, parent_auto, modes):
        """`n` as the API writes it; `inst_path` the instances it is inside
        (their guids, outermost first), which make its id."""
        own = gid(n["guid"])
        ident = "I" + ";".join(inst_path + [own]) if inst_path else own
        n = dict(n)
        # the overrides and the derived layout of every instance this layer
        # is inside, outermost last so the one closest to the screen wins
        here = keys(n)
        for s in reversed(scopes):
            for d in s.matching(s.derived, here):
                for k in ("size", "transform"):
                    if k in d["fields"]:
                        n[k] = d["fields"][k]
        for s in reversed(scopes):
            for o in s.matching(s.overrides, here):
                o["used"] = True
                for k, v in o["fields"].items():
                    if k in APPLIED:
                        n[k] = v
        modes = modes_of(n, modes)
        t = n.get("transform", {})
        x = parent_xy[0] + t.get("m02", 0)
        y = parent_xy[1] + t.get("m12", 0)
        size = n.get("size", {})
        kind = TYPES.get(n.get("type"), n.get("type"))
        out = {"id": ident, "name": n.get("name", ""), "type": kind}
        if n.get("visible") is False:
            out["visible"] = False
        if n.get("opacity", 1) < 1:
            out["opacity"] = num(n["opacity"], 3)
        if t.get("m01", 0) or t.get("m10", 0):
            out["rotation"] = num(math.atan2(t.get("m10", 0), t.get("m00", 1)), 4)
        out["absoluteBoundingBox"] = {
            "x": num(x), "y": num(y),
            "width": num(size.get("x", 0)), "height": num(size.get("y", 0)),
        }
        container = kind in ("FRAME", "INSTANCE", "COMPONENT")
        auto = container and n.get("stackMode") in ("HORIZONTAL", "VERTICAL")
        if auto:
            out["layoutMode"] = n["stackMode"]
            if n.get("stackSpacing"):
                out["itemSpacing"] = num(n["stackSpacing"])
            h, v = n.get("stackHorizontalPadding", 0), n.get("stackVerticalPadding", 0)
            pads = {
                "paddingLeft": h, "paddingRight": n.get("stackPaddingRight", h),
                "paddingTop": v, "paddingBottom": n.get("stackPaddingBottom", v),
            }
            for k, p in pads.items():
                if p:
                    out[k] = num(p)
            if n.get("stackPrimaryAlignItems", "MIN") != "MIN":
                out["primaryAxisAlignItems"] = n["stackPrimaryAlignItems"]
            if n.get("stackCounterAlignItems", "MIN") != "MIN":
                out["counterAxisAlignItems"] = n["stackCounterAlignItems"]
            fixed = n.get("stackPrimarySizing") == "FIXED"
            out["primaryAxisSizingMode"] = "FIXED" if fixed else "AUTO"
            hug = str(n.get("stackCounterSizing", "")).startswith("RESIZE_TO_FIT")
            out["counterAxisSizingMode"] = "AUTO" if hug else "FIXED"
        if parent_auto:
            if n.get("stackChildAlignSelf") == "STRETCH":
                out["layoutAlign"] = "STRETCH"
            if n.get("stackChildPrimaryGrow"):
                out["layoutGrow"] = num(n["stackChildPrimaryGrow"])
            if n.get("stackPositioning") == "ABSOLUTE":
                out["layoutPositioning"] = "ABSOLUTE"
        fills = self.paints.all(n.get("fillPaints"), modes)
        if fills:
            out["fills"] = fills
        strokes = self.paints.all(n.get("strokePaints"), modes)
        if strokes and n.get("strokeWeight", 1) > 0:
            out["strokes"] = strokes
            out["strokeWeight"] = num(n.get("strokeWeight", 1))
        fx = effects(n.get("effects"))
        if fx:
            out["effects"] = fx
        if n.get("cornerRadius"):
            out["cornerRadius"] = num(n["cornerRadius"])
        corners = [n.get(f"rectangle{c}CornerRadius") for c in
                   ("TopLeft", "TopRight", "BottomRight", "BottomLeft")]
        if all(c is not None for c in corners) and len(set(corners)) > 1:
            out["rectangleCornerRadii"] = [num(c) for c in corners]
        if container and not n.get("frameMaskDisabled", False):
            out["clipsContent"] = True
        if kind == "TEXT":
            out["characters"] = n.get("textData", {}).get("characters", "")
            out["style"] = self.style(n)
        children = self.canvas.children(n)
        if kind == "INSTANCE":
            symbol = self.canvas.by_id.get(gid(n["symbolData"]["symbolID"]))
            out["componentId"] = gid(n["symbolData"]["symbolID"])
            scope = Scope(n)
            # an override of the component itself is the instance's own
            for o in scope.overrides:
                if symbol and len(o["path"]) == 1 and o["path"][0] in keys(symbol):
                    o["used"] = True
            children = self.canvas.children(symbol) if symbol else []
            # the instances this one is inside go one level down
            scopes = [s.into(here) for s in scopes] + [scope]
            inst_path = inst_path + [own]
        else:
            scope = None
        kids = []
        for c in children:
            kids.append(self.node(c, (x, y), inst_path, scopes, auto, modes))
        if scope is not None:
            missing = scope.unapplied()
            if missing:
                out["unappliedOverrides"] = missing
        if kids:
            out["children"] = kids
        return out

    @staticmethod
    def style(n):
        size = n.get("fontSize", 12)
        font = n.get("fontName", {})
        st = {"fontFamily": font.get("family", ""),
              "fontWeight": weight(font.get("style", "Regular")),
              "fontSize": num(size)}
        lh = n.get("lineHeight", {})
        units, v = lh.get("units", "PERCENT"), lh.get("value", 100)
        if units == "PIXELS":
            st["lineHeightPx"] = num(v)
        elif units == "RAW":
            st["lineHeightPx"] = num(v * size)
        elif v != 100:
            st["lineHeightPx"] = num(v / 100 * size)
        ls = n.get("letterSpacing", {})
        if ls.get("value"):
            px = ls["value"] / 100 * size if ls.get("units") == "PERCENT" else ls["value"]
            st["letterSpacing"] = num(px)
        if n.get("textAlignHorizontal", "LEFT") != "LEFT":
            st["textAlignHorizontal"] = n["textAlignHorizontal"]
        st["textAutoResize"] = n.get("textAutoResize", "NONE")
        return st


def dump(obj, indent=0):
    """One key per line, values compact, `children` last."""
    pad = "  " * indent
    lines = ["{"]
    items = [(k, v) for k, v in obj.items() if k != "children"]
    kids = obj.get("children")
    for i, (k, v) in enumerate(items):
        comma = "," if i < len(items) - 1 or kids else ""
        text = dump(v, indent + 1) if isinstance(v, dict) and "id" in v else \
            json.dumps(v, separators=(", ", ": "))
        lines.append(f"{pad}  {json.dumps(k)}: {text}{comma}")
    if kids:
        lines.append(f'{pad}  "children": [')
        for i, c in enumerate(kids):
            comma = "," if i < len(kids) - 1 else ""
            lines.append(f"{pad}    {dump(c, indent + 2)}{comma}")
        lines.append(f"{pad}  ]")
    lines.append(f"{pad}}}")
    return "\n".join(lines)


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("fig")
    ap.add_argument("out", nargs="?", default=OUT)
    ap.add_argument("--page", default="Dark Theme")
    ap.add_argument("--section", default="settings")
    ap.add_argument("--frame", default="settings")
    a = ap.parse_args()
    canvas = Canvas(load(a.fig))
    doc = next(n for n in canvas.nodes if n.get("type") == "DOCUMENT")
    page = canvas.find(doc, lambda n: n.get("type") == "CANVAS" and a.page in n.get("name", ""),
                       f"page with {a.page!r}")
    section = canvas.find(page, lambda n: n.get("name") == a.section, f"section {a.section!r}")
    frame = canvas.find(section, lambda n: n.get("name") == a.frame and n.get("type") == "FRAME",
                        f"frame {a.frame!r}")
    w = Writer(canvas)
    st = section.get("transform", {})
    sx, sy = st.get("m02", 0), st.get("m12", 0)
    screen = w.node(frame, (sx, sy), [], [], False, modes_of(section, {}))
    ss = section.get("size", {})
    out = {
        "name": a.fig.rsplit("/", 1)[-1].removesuffix(".fig"),
        "document": {
            "id": gid(doc["guid"]), "name": doc.get("name", "Document"), "type": "DOCUMENT",
            "children": [{
                "id": gid(page["guid"]), "name": page.get("name", ""), "type": "CANVAS",
                "children": [{
                    "id": gid(section["guid"]), "name": section.get("name", ""), "type": "SECTION",
                    "absoluteBoundingBox": {"x": num(sx), "y": num(sy),
                                            "width": num(ss.get("x", 0)),
                                            "height": num(ss.get("y", 0))},
                    "children": [screen],
                }],
            }],
        },
    }
    text = dump(out) + "\n"
    with io.open(a.out, "w", encoding="ascii") as f:
        f.write(text)
    count = text.count('"id": ')
    print(f"{a.out}: {count} nodes, {text.count(chr(10))} lines")


if __name__ == "__main__":
    main()
