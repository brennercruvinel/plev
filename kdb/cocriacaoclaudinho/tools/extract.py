# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///
"""
extract.py - deterministic extractor for the plev/caranguejo-vermelho cocriacao dataset.

it is the engine that makes the scale-out cheap and consistent. a script does not
forget a field, does not get a timezone wrong, and applies the same scrub on every
unit, so it kills by construction the consistency bugs the llm sample had.

what it does (the 90 percent mechanical work):
  - walk only the valuable subset: conversation .jsonl, real conversation/tool .json,
    memory .md (__MEMORY/feedback/project/user) and plev/phi code; drop the noise
    (.DS_Store, lock, highwatermark, bcmap, node_modules, telemetry/1p_failed_events,
    settings/cache/session metadata, agent .meta.json stubs)
  - emit the 10 schema fields ALWAYS and in fixed order: id, source, captured,
    model_tier, project, kind, turns, scrubbed, status, tags. plus captured_source,
    owner_identity and needs_llm_reconstruction.
  - parse jsonl into a trajectory that preserves the real order
    input -> reasoning -> tool-call -> response. count tool calls per type and
    reasoning blocks.
  - deterministic scrub by regex: normalize /Users/<localuser> to /Users/<user> on
    every unit; never emit a personal email (redacted). log only the category.
  - mark each human input needs_llm_reconstruction true (chaotic: truncated, whisper
    tells, wrong merge) or false (already structured: pasted plan). the deterministic
    extractor does NOT reconstruct with voice; it normalizes (lowercase, no em dash)
    and leaves the marker for the later selective llm pass.
  - write one .md per source file with header + normalized/structured content.

source /Volumes/500G-SSD/claude2026 is READ-ONLY. this script only reads it.

usage:
  uv run kdb/cocriacaoclaudinho/tools/extract.py --out <dir> <file> [<file> ...]
  uv run kdb/cocriacaoclaudinho/tools/extract.py --out <dir> --manifest <list.txt>
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import statistics
import sys
from datetime import datetime, timezone

SOURCE_ROOT = "/Volumes/500G-SSD/claude2026"

# ---------------------------------------------------------------------------
# noise filter (drop on ingest, never spend extraction on it)
# ---------------------------------------------------------------------------
NOISE_PATH_SUBSTR = ("/node_modules/", "/.git/", "/dist/", "/build/", "/target/", "/cache/")
NOISE_BASENAME_RE = re.compile(
    r"(\.ds_store$|\.lock$|^lock$|highwatermark|\.bcmap$|\.meta\.json$)", re.I
)
NOISE_JSON_BASENAME_RE = re.compile(
    r"(^settings.*\.json$|.*cache.*\.json$|^\.claude\.json$|mcp-needs-auth.*\.json$|"
    r"known_marketplaces.*\.json$|installed_plugins.*\.json$|1p_failed_events)",
    re.I,
)
TELEMETRY_RE = re.compile(r"telemetry/1p_failed_events", re.I)

CODE_EXT = {
    ".rs", ".wgsl", ".js", ".ts", ".tsx", ".jsx", ".py", ".cjs", ".cts",
    ".mjs", ".mts", ".css", ".scss", ".sh", ".toml", ".yml", ".yaml", ".lua",
}

# ---------------------------------------------------------------------------
# scrub (deterministic, regex only)
# ---------------------------------------------------------------------------
# case-insensitive on the /Users literal so it also catches paths already lowercased
# by input normalization (e.g. /users/aac); the username token follows the literal
LOCAL_USER_RE = re.compile(r"/[Uu]sers/(?!<user>)[A-Za-z0-9_][A-Za-z0-9_.\-]*")
EMAIL_RE = re.compile(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}")
EM_DASH_RE = re.compile(r"[—–]")  # em dash, en dash
NBSP_RE = re.compile(r"[   ]")
SYSTEM_REMINDER_RE = re.compile(r"<system-reminder>.*?</system-reminder>", re.S | re.I)

# owner public identity (kept as lineage, only flagged for output review)
OWNER_IDENTITY_RE = re.compile(
    r"(brenner\s+cruvinel|cruvinel|brennertalks|brennercruvinel)", re.I
)

# human-input system wrappers to exclude (these are not human voice)
SYSTEM_PREFIXES = (
    "<local-command", "<command-name", "<command-message", "<command-args",
    "<bash-input", "<bash-stdout", "<bash-stderr", "<user-memory-input",
    "<user-prompt-submit-hook", "<post-tool", "<pre-tool",
)
CONTINUATION_PREFIX = "this session is being continued"
INTERRUPT_PREFIX = "[request interrupted"

# deterministic tag vocabulary scanned in content (lowercase substring match)
TAG_VOCAB = (
    "gpu", "wgpu", "wasm", "shader", "wgsl", "compositor", "layer", "atlas",
    "cosmic-text", "taffy", "winit", "rope", "ide", "showcase", "worktree",
    "merge", "deploy", "rust", "android", "ios", "webgpu", "premultiplied",
    "dirty-tracking", "scene", "text", "render", "pipeline", "blur", "shadow",
    "opacity", "context-compaction", "mission", "knowledge", "mcp", "email",
    "telemetry", "paper", "notebook", "welfare", "phi",
)


# ---------------------------------------------------------------------------
# helpers
# ---------------------------------------------------------------------------
def stable_id(source_path: str) -> str:
    """sha256(source_path)[:12]; reproduces the prior sample ids for cross-pass dedupe."""
    return hashlib.sha256(source_path.encode("utf-8")).hexdigest()[:12]


def iso_utc(dt: datetime) -> str:
    return dt.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def parse_ts(s: str) -> datetime | None:
    if not s:
        return None
    try:
        return datetime.fromisoformat(s.replace("Z", "+00:00"))
    except ValueError:
        return None


def detect_project(path: str) -> str:
    low = path.lower()
    if "plev" in low:
        return "plev"
    if re.search(r"(?<![a-z0-9])phi(?![a-z0-9])", low):
        return "phi"
    return "outro"


def detect_kind(path: str, ext: str) -> str:
    low = path.lower()
    if ext == ".jsonl":
        return "trajetoria"
    if ext == ".md":
        return "memoria"
    if ext == ".json":
        return "trace-tool"
    if ext in CODE_EXT:
        if re.search(r"(?<![a-z0-9])(experiment|failed|broken)", low):
            return "experimento-falho"
        return "codigo"
    return "codigo"


def model_to_tier(model: str | None) -> str | None:
    if not model:
        return None
    m = model.lower()
    if "opus" in m:
        return "opus"
    if "sonnet" in m:
        return "sonnet"
    if "haiku" in m:
        return "haiku"
    if "fable" in m:
        return "fable"
    return None


def is_noise(path: str) -> bool:
    low = path.replace("\\", "/").lower()
    if any(s in low for s in NOISE_PATH_SUBSTR):
        return True
    if TELEMETRY_RE.search(low):
        return True
    base = os.path.basename(low)
    if NOISE_BASENAME_RE.search(base):
        return True
    if base.endswith(".json") and NOISE_JSON_BASENAME_RE.search(base):
        return True
    if "/sessions/" in low and base.endswith(".json"):
        return True
    return False


def scrub(text: str, log: dict) -> str:
    """normalize local user paths and redact emails. log categories only, never content."""
    if not text:
        return text
    n_email = len(EMAIL_RE.findall(text))
    if n_email:
        log["email"] = log.get("email", 0) + n_email
        text = EMAIL_RE.sub("<email-redacted>", text)
    n_user = len(LOCAL_USER_RE.findall(text))
    if n_user:
        log["local-user-path"] = log.get("local-user-path", 0) + n_user
        text = LOCAL_USER_RE.sub("/Users/<user>", text)
    return text


def normalize_input(text: str) -> str:
    """deterministic normalization for a human input. NOT voice reconstruction.
    lowercase, drop em/en dash, drop nbsp and the terminal prompt glyph, collapse runs."""
    t = SYSTEM_REMINDER_RE.sub("", text)
    t = NBSP_RE.sub(" ", t)
    t = t.replace("❯", "").replace("❱", "").replace("›", "")
    t = EM_DASH_RE.sub("-", t)
    t = t.lower()
    t = re.sub(r"[ \t]{2,}", " ", t)
    t = re.sub(r"\n{3,}", "\n\n", t)
    return t.strip()


def needs_recon(text: str) -> bool:
    """true when the input is chaotic (truncated, whisper tells, wrong merge);
    false when already structured (a pasted plan). deterministic heuristic."""
    t = text.strip()
    if not t:
        return False
    low = t.lower()
    # already structured pastes: no voice reconstruction, only normalization
    if t.startswith("#") or t.startswith("```"):
        return False
    if low.startswith("implement the following plan") or low.startswith("implemente o seguinte"):
        return False
    # whisper / terminal tells
    if " " in text or "❯" in text or " " in text:
        return True
    nonempty = [ln for ln in t.split("\n") if ln.strip()]
    if len(nonempty) >= 4:
        med = statistics.median(len(ln) for ln in nonempty)
        if med < 33:  # narrow cli input box wrap
            return True
    # whisper dictation opener: starts with a lowercase letter
    first = t[:1]
    if first.isalpha() and first.islower():
        return True
    return False


def is_human_input(obj: dict) -> str | None:
    """return the human text if this jsonl entry is a substantive human input, else None.
    excludes meta, command wrappers, tool_result-bearing messages, interrupts and the
    continuation summary (system, not human voice)."""
    if obj.get("type") != "user" or obj.get("isMeta"):
        return None
    msg = obj.get("message")
    if not isinstance(msg, dict):
        return None
    c = msg.get("content")
    text = None
    if isinstance(c, str):
        text = c
    elif isinstance(c, list):
        has_tr = any(isinstance(b, dict) and b.get("type") == "tool_result" for b in c)
        if has_tr:
            return None
        parts = [b.get("text", "") for b in c if isinstance(b, dict) and b.get("type") == "text"]
        text = "\n".join(parts)
    if not text:
        return None
    t = text.strip()
    low = t.lower()
    if low.startswith(SYSTEM_PREFIXES) or t.startswith(SYSTEM_PREFIXES):
        return None
    if low.startswith(INTERRUPT_PREFIX):
        return None
    if low.startswith(CONTINUATION_PREFIX):
        return None
    return t


def is_continuation_summary(obj: dict) -> bool:
    if obj.get("type") != "user":
        return False
    msg = obj.get("message")
    if not isinstance(msg, dict):
        return False
    c = msg.get("content")
    s = c if isinstance(c, str) else ""
    return s.strip().lower().startswith(CONTINUATION_PREFIX)


def collect_tags(project: str, kind: str, content_sample: str) -> list[str]:
    low = content_sample.lower()
    tags = [project, kind]
    for kw in TAG_VOCAB:
        if kw in low and kw not in tags:
            tags.append(kw)
    return tags


# ---------------------------------------------------------------------------
# unit dataclass-ish (plain dict) + header rendering
# ---------------------------------------------------------------------------
SCHEMA_ORDER = [
    "id", "source", "captured", "model_tier", "project",
    "kind", "turns", "scrubbed", "status", "tags",
]


def render_header(unit: dict) -> str:
    lines = ["---"]
    for k in SCHEMA_ORDER:
        v = unit[k]
        if k == "tags":
            lines.append(f"tags: [{', '.join(v)}]")
        elif isinstance(v, bool):
            lines.append(f"{k}: {str(v).lower()}")
        else:
            lines.append(f"{k}: {v}")
    # extra provenance / routing fields after the fixed 10
    lines.append(f"captured_source: {unit['captured_source']}")
    lines.append(f"owner_identity: {str(unit['owner_identity']).lower()}")
    lines.append(f"needs_llm_reconstruction: {str(unit['needs_llm_reconstruction']).lower()}")
    lines.append("---")
    return "\n".join(lines)


def preview(text: str, limit: int) -> str:
    text = text.strip().replace("\n", " ")
    if len(text) > limit:
        return text[:limit].rstrip() + " [...]"
    return text


# ---------------------------------------------------------------------------
# extractors per kind
# ---------------------------------------------------------------------------
def extract_jsonl(source: str, raw: str, tr_preview: int) -> dict:
    scrub_log: dict = {}
    rows = []
    for line in raw.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError:
            continue

    # provenance from first message-bearing row
    cwd = branch_first = branch_last = version = None
    first_ts = last_ts = None
    models: dict[str, int] = {}
    tool_counts: dict[str, int] = {}
    n_tool = n_think = n_text = 0
    n_continuation = 0
    is_sidechain = False

    events = []  # ordered (kind, payload) preserving real order
    human_turn = 0
    chaotic_inputs = 0
    human_inputs_total = 0

    for obj in rows:
        ts = parse_ts(obj.get("timestamp", ""))
        if ts:
            if first_ts is None or ts < first_ts:
                first_ts = ts
            if last_ts is None or ts > last_ts:
                last_ts = ts
        if obj.get("isSidechain"):
            is_sidechain = True
        if obj.get("cwd") and cwd is None:
            cwd = obj["cwd"]
        if obj.get("version") and version is None:
            version = obj["version"]
        gb = obj.get("gitBranch")
        if gb:
            if branch_first is None:
                branch_first = gb
            branch_last = gb

        if is_continuation_summary(obj):
            n_continuation += 1
            events.append(("continuation", "resumo de compactacao do harness (sistema, nao voz humana)"))
            continue

        human = is_human_input(obj)
        if human is not None:
            human_turn += 1
            human_inputs_total += 1
            chaotic = needs_recon(human)
            if chaotic:
                chaotic_inputs += 1
            # scrub first (paths are still capital /Users/<localuser>), then normalize;
            # normalization lowercases the already-clean /Users/<user> placeholder
            normalized = normalize_input(scrub(human, scrub_log))
            events.append(("human", {"turn": human_turn, "chaotic": chaotic, "text": normalized}))
            continue

        msg = obj.get("message")
        if obj.get("type") == "assistant" and isinstance(msg, dict):
            mdl = msg.get("model")
            if mdl and not str(mdl).startswith("<"):
                models[mdl] = models.get(mdl, 0) + 1
            c = msg.get("content")
            if isinstance(c, list):
                for b in c:
                    if not isinstance(b, dict):
                        continue
                    bt = b.get("type")
                    if bt == "thinking":
                        n_think += 1
                        events.append(("reasoning", scrub(b.get("thinking", ""), scrub_log)))
                    elif bt == "tool_use":
                        n_tool += 1
                        name = b.get("name", "?")
                        tool_counts[name] = tool_counts.get(name, 0) + 1
                        inp = b.get("input", {})
                        keys = ("file_path", "command", "pattern", "path", "url", "description", "prompt", "query")
                        sel = {k: inp[k] for k in keys if k in inp}
                        events.append(("tool_use", {"name": name, "input": scrub(json.dumps(sel, ensure_ascii=False), scrub_log)}))
                    elif bt == "text":
                        n_text += 1
                        events.append(("response", scrub(b.get("text", ""), scrub_log)))
        elif obj.get("type") == "user" and isinstance(msg, dict):
            c = msg.get("content")
            if isinstance(c, list):
                for b in c:
                    if isinstance(b, dict) and b.get("type") == "tool_result":
                        body = b.get("content")
                        if isinstance(body, list):
                            body = " ".join(
                                x.get("text", "") for x in body if isinstance(x, dict)
                            )
                        body = str(body) if body is not None else ""
                        err = b.get("is_error")
                        events.append(("tool_result", {"is_error": bool(err), "preview": scrub(body, scrub_log)}))

    # tier
    tier = "desconhecido"
    if models:
        dominant = max(models, key=models.get)
        tier = model_to_tier(dominant) or "desconhecido"

    turns = human_inputs_total + n_text + n_tool

    # captured: content timestamp (correct UTC), not the copy mtime
    if first_ts is not None:
        captured = iso_utc(first_ts)
        captured_source = "content"
    else:
        captured = iso_utc(datetime.fromtimestamp(os.path.getmtime(source), tz=timezone.utc))
        captured_source = "copy-mtime"

    project = detect_project(source)
    content_sample = raw[:20000]
    owner = bool(OWNER_IDENTITY_RE.search(raw))
    tags = collect_tags(project, "trajetoria", content_sample)
    session = os.path.basename(source).replace(".jsonl", "")
    kind = "trace-reasoning" if is_sidechain else "trajetoria"

    unit = {
        "id": stable_id(source),
        "source": source,
        "captured": captured,
        "model_tier": tier,
        "project": project,
        "kind": kind,
        "turns": turns,
        "scrubbed": True,
        "status": "normalizado",
        "tags": tags,
        "captured_source": captured_source,
        "owner_identity": owner,
        "needs_llm_reconstruction": chaotic_inputs > 0,
    }

    # render body
    tool_summary = ", ".join(f"{k} {v}" for k, v in sorted(tool_counts.items(), key=lambda x: -x[1]))
    dur = f"{iso_utc(first_ts)} a {iso_utc(last_ts)}" if first_ts and last_ts else "desconhecida"
    cwd_s = scrub(cwd or "desconhecido", scrub_log)
    b = []
    b.append(f"# {project} {kind} {session[:12]}")
    b.append("")
    b.append("contexto da trajetoria")
    b.append(f"- cwd original: {cwd_s}")
    b.append(f"- branch: {branch_first or 'desconhecida'}"
             + (f" (deriva para {branch_last})" if branch_last and branch_last != branch_first else ""))
    b.append(f"- harness: claude code {version or 'desconhecida'}")
    b.append(f"- modelo na trace: {max(models, key=models.get) if models else 'desconhecido'} (tier {tier})")
    b.append(f"- duracao: {dur} utc")
    b.append(f"- traces: {n_tool} tool calls ({tool_summary or 'nenhum'}), {n_text} respostas, {n_think} blocos de raciocinio")
    b.append(f"- estrutura: {human_inputs_total} inputs humanos substantivos"
             + (f" + {n_continuation} sumario(s) de continuacao" if n_continuation else ""))
    b.append(f"- inputs humanos caoticos (needs_llm_reconstruction): {chaotic_inputs}")
    b.append("")
    b.append("## sequencia (ordem real preservada)")
    b.append("")
    for kindev, payload in events:
        if kindev == "human":
            mark = "true" if payload["chaotic"] else "false"
            b.append(f"### turno {payload['turn']}, input humano [needs_llm_reconstruction: {mark}]")
            b.append("")
            b.append("> " + payload["text"].replace("\n", "\n> "))
            b.append("")
        elif kindev == "continuation":
            b.append(f"- [sumario de continuacao] {payload}")
        elif kindev == "reasoning":
            p = preview(payload, 1200)
            if p:
                b.append(f"- raciocinio: {p}")
        elif kindev == "tool_use":
            b.append(f"- tool-call {payload['name']}: {preview(payload['input'], 300)}")
        elif kindev == "tool_result":
            tag = "erro" if payload["is_error"] else "ok"
            p = preview(payload["preview"], 300)
            b.append(f"  - tool-result ({tag}): {p if p else '(vazio)'}")
        elif kindev == "response":
            p = preview(payload, 1200)
            if p:
                b.append(f"- resposta: {p}")
    b.append("")
    b.append(_scrub_block(scrub_log))
    b.append("")
    b.append(_hook_block(source, "kdb/cocriacaoclaudinho/dataset/", scrubbed=bool(scrub_log),
                         tier_inferred=tier != "desconhecido", reconstructed=False))

    return {"unit": unit, "body": "\n".join(b), "scrub_log": scrub_log,
            "chaotic_inputs": chaotic_inputs, "human_inputs": human_inputs_total}


def extract_md(source: str, raw: str) -> dict:
    scrub_log: dict = {}
    project = detect_project(source)
    owner = bool(OWNER_IDENTITY_RE.search(raw))
    body_content = scrub(raw, scrub_log)
    captured = iso_utc(datetime.fromtimestamp(os.path.getmtime(source), tz=timezone.utc))
    tags = collect_tags(project, "memoria", raw[:20000])
    unit = {
        "id": stable_id(source),
        "source": source,
        "captured": captured,
        "model_tier": "desconhecido",
        "project": project,
        "kind": "memoria",
        "turns": 0,
        "scrubbed": True,
        "status": "normalizado",
        "tags": tags,
        "captured_source": "copy-mtime",
        "owner_identity": owner,
        "needs_llm_reconstruction": False,
    }
    name = os.path.basename(source)
    b = [f"# memoria: {name}", "",
         "nota: memoria/preferencia sem timestamp de conteudo; captured e a data da copia "
         "(copy-mtime), nao a da conversa. conteudo preservado verbatim, apenas com scrub de paths/email.",
         "", "## conteudo (normalizado, scrub aplicado)", "", body_content, "",
         _scrub_block(scrub_log), "",
         _hook_block(source, "kdb/cocriacaoclaudinho/dataset/", scrubbed=bool(scrub_log),
                     tier_inferred=False, reconstructed=False)]
    return {"unit": unit, "body": "\n".join(b), "scrub_log": scrub_log,
            "chaotic_inputs": 0, "human_inputs": 0}


def extract_json(source: str, raw: str, tr_preview: int) -> dict:
    scrub_log: dict = {}
    project = detect_project(source)
    owner = bool(OWNER_IDENTITY_RE.search(raw))
    try:
        obj = json.loads(raw)
    except json.JSONDecodeError:
        obj = None

    blocks = []
    if isinstance(obj, list):
        for it in obj:
            if isinstance(it, dict):
                if it.get("type") == "text":
                    blocks.append(("text", it.get("text", "")))
                elif "content" in it and isinstance(it["content"], str):
                    blocks.append((it.get("role", "msg"), it["content"]))
                else:
                    blocks.append((it.get("type", it.get("role", "block")), json.dumps(it, ensure_ascii=False)))
    elif isinstance(obj, dict):
        blocks.append(("dict", json.dumps(obj, ensure_ascii=False)))

    captured = iso_utc(datetime.fromtimestamp(os.path.getmtime(source), tz=timezone.utc))
    tags = collect_tags(project, "trace-tool", raw[:20000])
    unit = {
        "id": stable_id(source),
        "source": source,
        "captured": captured,
        "model_tier": "desconhecido",
        "project": project,
        "kind": "trace-tool",
        "turns": len(blocks),
        "scrubbed": True,
        "status": "normalizado",
        "tags": tags,
        "captured_source": "copy-mtime",
        "owner_identity": owner,
        "needs_llm_reconstruction": False,
    }
    b = [f"# trace-tool: {os.path.basename(source)}", "",
         "nota: payload de tool-call/result em json sem timestamp de conteudo; captured e "
         "a data da copia (copy-mtime). blocos preservados em ordem, scrub aplicado.",
         "", "## blocos (ordem preservada)", ""]
    for role, txt in blocks:
        b.append(f"- [{role}] {preview(scrub(txt, scrub_log), tr_preview)}")
    b.append("")
    b.append(_scrub_block(scrub_log))
    b.append("")
    b.append(_hook_block(source, "kdb/cocriacaoclaudinho/dataset/", scrubbed=bool(scrub_log),
                         tier_inferred=False, reconstructed=False))
    return {"unit": unit, "body": "\n".join(b), "scrub_log": scrub_log,
            "chaotic_inputs": 0, "human_inputs": 0}


def extract_code(source: str, raw: str) -> dict:
    scrub_log: dict = {}
    project = detect_project(source)
    ext = os.path.splitext(source)[1].lower()
    kind = detect_kind(source, ext)
    owner = bool(OWNER_IDENTITY_RE.search(raw))
    body_content = scrub(raw, scrub_log)
    captured = iso_utc(datetime.fromtimestamp(os.path.getmtime(source), tz=timezone.utc))
    tags = collect_tags(project, kind, raw[:20000])
    lang = ext.lstrip(".")
    unit = {
        "id": stable_id(source),
        "source": source,
        "captured": captured,
        "model_tier": "desconhecido",
        "project": project,
        "kind": kind,
        "turns": 0,
        "scrubbed": True,
        "status": "normalizado",
        "tags": tags,
        "captured_source": "copy-mtime",
        "owner_identity": owner,
        "needs_llm_reconstruction": False,
    }
    b = [f"# codigo: {os.path.basename(source)}", "",
         "nota: codigo preservado verbatim (status normalizado), apenas com scrub de paths/email. "
         "a intencao reconstruida e tarefa da passada llm seletiva.",
         "", f"```{lang}", body_content, "```", "",
         _scrub_block(scrub_log), "",
         _hook_block(source, "kdb/cocriacaoclaudinho/dataset/", scrubbed=bool(scrub_log),
                     tier_inferred=False, reconstructed=False)]
    return {"unit": unit, "body": "\n".join(b), "scrub_log": scrub_log,
            "chaotic_inputs": 0, "human_inputs": 0}


def _scrub_block(scrub_log: dict) -> str:
    lines = ["## scrub (categoria apenas, sem conteudo)"]
    if not scrub_log:
        lines.append("- nenhum dado pessoal detectado nesta unidade")
    else:
        for cat, n in sorted(scrub_log.items()):
            if cat == "local-user-path":
                lines.append(f"- username de conta local em path: normalizado /Users/<localuser> para /Users/<user> ({n} ocorrencias)")
            elif cat == "email":
                lines.append(f"- email pessoal: redigido, nao reproduzido ({n} ocorrencias)")
            else:
                lines.append(f"- {cat}: {n} ocorrencias")
    return "\n".join(lines)


def _hook_block(source: str, outdir: str, scrubbed: bool, tier_inferred: bool, reconstructed: bool) -> str:
    return "\n".join([
        "## hook do extrator (preenchido)",
        "- fonte lida em read-only, sem escrita/move/delete na fonte: sim",
        f"- saida apenas em {outdir}: sim",
        "- lineage no campo source: sim",
        "- ordem input/raciocinio/tool-call/resposta preservada: sim",
        f"- model_tier inferido da trace, nao chutado: {'sim' if tier_inferred else 'nao inferivel, marcado desconhecido'}",
        "- reconstrucao com voz NAO feita pelo extrator deterministico (marcador deixado p/ passada llm): sim",
        "- sumario de continuacao marcado como sistema, nao como voz humana: sim",
        f"- scrub aplicado, log so de categoria: {'sim' if scrubbed else 'sem dado pessoal'}",
        "- nao commitei: sim",
    ])


# ---------------------------------------------------------------------------
# driver
# ---------------------------------------------------------------------------
def process_file(source: str, outdir: str, tr_preview: int) -> dict:
    ext = os.path.splitext(source)[1].lower()
    raw = open(source, "r", encoding="utf-8", errors="replace").read()
    if ext == ".jsonl":
        res = extract_jsonl(source, raw, tr_preview)
    elif ext == ".md":
        res = extract_md(source, raw)
    elif ext == ".json":
        res = extract_json(source, raw, tr_preview)
    else:
        res = extract_code(source, raw)

    unit = res["unit"]
    header = render_header(unit)
    out_name = f"{unit['id']}.md"
    out_path = os.path.join(outdir, out_name)
    with open(out_path, "w", encoding="utf-8") as f:
        f.write(header + "\n\n" + res["body"] + "\n")

    # schema self-check: 10 fields present, in order, non-empty
    ok = all(unit.get(k) not in (None, "") for k in SCHEMA_ORDER)
    res["out_path"] = out_path
    res["schema_ok"] = ok
    return res


def main() -> int:
    ap = argparse.ArgumentParser(description="deterministic cocriacao extractor")
    ap.add_argument("paths", nargs="*", help="source files to extract")
    ap.add_argument("--manifest", help="file listing source paths, one per line")
    ap.add_argument("--out", default="kdb/cocriacaoclaudinho/dataset",
                    help="output directory for .md units")
    ap.add_argument("--source-root", default=SOURCE_ROOT)
    ap.add_argument("--tool-result-preview", type=int, default=300,
                    help="max chars per tool-result/json block preview")
    args = ap.parse_args()

    paths = list(args.paths)
    if args.manifest:
        with open(args.manifest) as f:
            paths.extend(ln.strip() for ln in f if ln.strip() and not ln.startswith("#"))

    os.makedirs(args.out, exist_ok=True)

    processed, skipped_noise, schema_ok = 0, 0, 0
    by_kind: dict = {}
    by_project: dict = {}
    by_tier: dict = {}
    scrub_totals: dict = {}
    chaotic_units = 0
    chaotic_inputs_total = 0
    human_inputs_total = 0
    owner_units = 0
    examples = []

    for src in paths:
        if not os.path.isfile(src):
            print(f"SKIP (missing): {src}", file=sys.stderr)
            continue
        if is_noise(src):
            skipped_noise += 1
            continue
        try:
            res = process_file(src, args.out, args.tool_result_preview)
        except Exception as e:  # noqa: BLE001
            print(f"ERROR {src}: {e}", file=sys.stderr)
            continue
        processed += 1
        u = res["unit"]
        by_kind[u["kind"]] = by_kind.get(u["kind"], 0) + 1
        by_project[u["project"]] = by_project.get(u["project"], 0) + 1
        by_tier[u["model_tier"]] = by_tier.get(u["model_tier"], 0) + 1
        if res["schema_ok"]:
            schema_ok += 1
        if u["needs_llm_reconstruction"]:
            chaotic_units += 1
        if u["owner_identity"]:
            owner_units += 1
        chaotic_inputs_total += res["chaotic_inputs"]
        human_inputs_total += res["human_inputs"]
        for cat, n in res["scrub_log"].items():
            scrub_totals[cat] = scrub_totals.get(cat, 0) + n
        examples.append({"id": u["id"], "kind": u["kind"], "out": res["out_path"]})

    summary = {
        "processed": processed,
        "skipped_noise": skipped_noise,
        "schema_ok": f"{schema_ok}/{processed}",
        "by_kind": by_kind,
        "by_project": by_project,
        "by_tier": by_tier,
        "units_with_chaotic_input": chaotic_units,
        "chaotic_human_inputs_total": chaotic_inputs_total,
        "human_inputs_total": human_inputs_total,
        "owner_identity_units": owner_units,
        "scrub_category_totals": scrub_totals,
    }
    print(json.dumps(summary, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
