"""
Parses pasted or uploaded decklists into (quantity, name) pairs.

Handles the common export shapes from Moxfield, Archidekt, TappedOut, and
plain "1 Card Name" text lists — including trailing set codes/collector
numbers ("1 Sol Ring (C21) 289"), foil markers, "1x" quantity prefixes, and
section headers ("Commander", "Creatures (23)", etc.) that should be skipped
rather than treated as card names.

This module only PARSES text into candidate (quantity, name) pairs — it never
touches the database. Matching those names against the local Scryfall index
(exact / approximate / not found) happens in server.py, and nothing is written
to a deck until the user confirms the parsed+matched preview. Never guess a
card into existence: an unmatched or ambiguous name is surfaced to the user,
not silently dropped or auto-corrected.
"""
import csv
import io
import re

# "1 Card Name", "1x Card Name", "1X Card Name", or bare "Card Name" (qty defaults to 1)
LINE_RE = re.compile(r"^\s*(?:(\d+)\s*[xX]?\s+)?(.+?)\s*$")

# Lines that are section labels, not cards — e.g. "Commander", "Creatures (23)",
# "Sideboard:". Only skipped when they don't start with a quantity, so a real
# card that happens to start with one of these words ("Landfall Ritual") is
# still read normally.
SECTION_HEADER_RE = re.compile(
    r"^(commander[s]?|deck|mainboard|maybeboard|sideboard|companion|"
    r"creatures?|land[s]?|instants?|sorceries|artifacts?|enchantments?|"
    r"planeswalkers?|battles?|other|spells?)\s*:?\s*(\(\d+\))?\s*$",
    re.I,
)


def _strip_set_info(name: str) -> str:
    """Removes trailing set/collector/foil annotations exporters commonly add.

    Runs the strips in a loop until nothing changes, since these annotations
    stack in either order — e.g. "Zulaport Cutthroat (C21) 12" needs the
    trailing number stripped AND the "(C21)" that's no longer at the end
    once the number is gone.
    """
    prev = None
    while prev != name:
        prev = name
        name = re.sub(r"\s*\([^)]*\)\s*$", "", name)        # "(C21)" / "(znr)"
        name = re.sub(r"\s*\[[^\]]*\]\s*$", "", name)        # "[ZNR]"
        name = re.sub(r"\s*\*[fF]\*\s*$", "", name)          # "*F*" foil marker
        name = re.sub(r"\s+[A-Z0-9]{2,6}\s+#?\d+[a-z]?\s*$", "", name)  # " ZNR 189"
        name = re.sub(r"\s+#?\d+\s*$", "", name)             # trailing lone collector number
        name = name.strip()
    return name


def parse_text(text: str):
    """Returns [(quantity, name), ...] from pasted plain-text decklist content."""
    entries = []
    for raw in (text or "").splitlines():
        line = raw.strip()
        if not line or line.startswith(("#", "//")):
            continue
        if SECTION_HEADER_RE.match(line):
            continue
        m = LINE_RE.match(line)
        if not m:
            continue
        qty_str, name = m.group(1), m.group(2).strip()
        name = _strip_set_info(name)
        if name:
            entries.append((int(qty_str) if qty_str else 1, name))
    return entries


_IGNORE_CELLS = {"name", "card", "card name", "quantity", "qty", "count"}


def parse_csv(text: str):
    """Reads a quantity+name CSV/TSV export (header row optional, either column order tolerated)."""
    entries = []
    sample = text[:2048]
    try:
        dialect = csv.Sniffer().sniff(sample, delimiters=",;\t")
    except csv.Error:
        dialect = csv.excel
    for row in csv.reader(io.StringIO(text), dialect):
        cells = [c.strip() for c in row if c and c.strip()]
        if not cells:
            continue
        if cells[0].isdigit():
            qty, name = int(cells[0]), (cells[1] if len(cells) > 1 else "")
        elif len(cells) > 1 and cells[1].isdigit():
            qty, name = int(cells[1]), cells[0]
        else:
            qty, name = 1, cells[0]
        name = _strip_set_info(name)
        if name and name.lower() not in _IGNORE_CELLS:
            entries.append((qty, name))
    return entries


def parse_xlsx(file_bytes: bytes):
    """Reads the first sheet of an .xlsx file: quantity+name in the first two columns (either order)."""
    try:
        import openpyxl
    except ImportError as e:
        raise RuntimeError(
            "Suporte a .xlsx precisa do pacote openpyxl (não incluído por padrão): "
            "pip3 install --user openpyxl — ou exporte a decklist como .txt/.csv, "
            "que já funciona sem instalar nada a mais."
        ) from e

    wb = openpyxl.load_workbook(io.BytesIO(file_bytes), data_only=True)
    ws = wb.active
    entries = []
    for row in ws.iter_rows(values_only=True):
        cells = [str(c).strip() for c in row if c is not None and str(c).strip()]
        if not cells:
            continue
        if cells[0].isdigit():
            qty, name = int(cells[0]), (cells[1] if len(cells) > 1 else "")
        elif len(cells) > 1 and cells[1].isdigit():
            qty, name = int(cells[1]), cells[0]
        else:
            qty, name = 1, cells[0]
        name = _strip_set_info(name)
        if name and name.lower() not in _IGNORE_CELLS:
            entries.append((qty, name))
    return entries
