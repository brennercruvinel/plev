#!/usr/bin/env python3
"""The docs/ tree as data: every page's header, its links and its relations.

    tool/tasks/docbase.py index   rewrite the folder indexes and docs/CATALOG.json; check that CHANGELOG.md lists every record
    tool/tasks/docbase.py check   fail on a bad header, a dead link or anchor, a stale catalog

A header is YAML restricted to one `key: value` per line, where a value is
a bare word, a JSON string, `null` or an inline list `[a, "b c"]`, so the
standard library reads it and any YAML reader agrees. The fields and their
vocabularies are docs/Research/Workflows/docs-conventions.md.
"""
from collections import Counter, defaultdict
from pathlib import Path
from urllib.parse import unquote
import argparse
import json
import os
import re
import sys

REPO = Path(__file__).resolve().parents[2]
REQUIRED = ['id', 'title', 'project', 'type', 'status', 'date', 'date_basis', 'topics',
            'relations', 'source', 'source_revision', 'revised', 'review']
OPTIONAL = ['parent', 'recorded', 'evidence']
ORDER = ['id', 'title', 'project', 'type', 'parent', 'status', 'date', 'date_basis', 'recorded', 'topics',
         'relations', 'evidence', 'source', 'source_revision', 'revised', 'review']
KINDS = {'index', 'record', 'template', 'adr', 'task', 'learning', 'program', 'thesis',
         'experiment', 'draft', 'manual', 'reference', 'research', 'section'}
STATES = {'active', 'reference', 'proposed', 'accepted', 'superseded', 'pending',
          'in-progress', 'done', 'draft'}
BASES = {'header', 'text', 'git', 'reconstructed', 'editorial', 'undated', 'generated'}
REVIEWS = {'pending', 'reviewed', 'generated'}
# the themes adr/ and Research/ share: a decision and what was learned about
# the same subject sit under the same name in both trees
THEMES = {'Ecosystem', 'Generative', 'Importing', 'Interface', 'Recording', 'Rendering',
          'Semantics', 'Structure', 'Workflows'}
SKIP = {'docs/adr/TEMPLATE.md'}
HOMES = {
    'Tasks': ('Tasks', 'One folder per task: the plan, the execution, the delivery and what stayed open. '
              'A long task keeps its phases as pages beside its README.'),
    'Experiments': ('Experiments', 'What was measured or tested: the question, the setup, the results '
                    'and their limits, with the task pages that hold the full record.'),
    'Research': ('Research', 'What we know, by the themes the ADRs use: the references studied, the '
                 'comparisons, the thesis and the program, how each subsystem was designed and how '
                 'to work on the repository.'),
    'Learnings': ('Learnings', 'What the problems taught that is not the context of one decision. '
                  'A learning that belongs to an ADR lives in that ADR\'s folder.'),
}


def value(text):
    text = text.strip()
    if text == 'null':
        return None
    if text.startswith('['):
        inner = text[1:-1].strip()
        if not inner:
            return []
        items = re.findall(r'"(?:[^"\\]|\\.)*"|[^,\s][^,]*', inner)
        return [value(i) for i in items]
    if text.startswith('"'):
        return json.loads(text)
    return text


def scalar(v):
    if v is None:
        return 'null'
    if isinstance(v, list):
        return '[' + ', '.join(scalar(x) for x in v) + ']'
    s = str(v)
    if re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9._/-]*', s) and s not in ('null', 'true', 'false', 'yes', 'no'):
        return s
    return json.dumps(s, ensure_ascii=False)


def parse(text, where):
    if not text.startswith('---\n'):
        raise ValueError(f'{where}: no header')
    _, head, body = text.split('---', 2)
    meta = {}
    for line in head.strip().splitlines():
        key, sep, rest = line.partition(':')
        if not sep or key in meta or not re.fullmatch(r'[a-z_]+', key):
            raise ValueError(f'{where}: bad header line {line!r}')
        meta[key] = value(rest)
    return meta, body.lstrip('\n')


def emit(meta, body):
    keys = [k for k in ORDER if k in meta] + [k for k in meta if k not in ORDER]
    head = ''.join(f'{k}: {scalar(meta[k])}\n' for k in keys if k in meta)
    return '---\n' + head + '---\n\n' + body.strip() + '\n'


def outside_fences(body):
    fence = None
    for line in body.splitlines():
        m = re.match(r'^ {0,3}(`{3,}|~{3,})', line)
        if m:
            if fence is None:
                fence = m[1]
            elif m[1][0] == fence[0] and len(m[1]) >= len(fence):
                fence = None
            continue
        if fence is None:
            yield line


def anchor(title):
    title = re.sub(r'<[^>]*>|[`*]', '', title).strip().lower()
    return re.sub(r'[^\w\- ]', '', title, flags=re.UNICODE).replace(' ', '-')


def headings(body):
    seen, out = Counter(), []
    for line in outside_fences(body):
        m = re.match(r'^(#{1,6}) +(.+?)\s*#*\s*$', line)
        if m:
            key = anchor(m[2])
            out.append({'level': len(m[1]), 'title': m[2],
                        'anchor': key + (f'-{seen[key]}' if seen[key] else '')})
            seen[key] += 1
    return out


def load(root):
    pages = {}
    for p in sorted(root.rglob('*.md')):
        where = 'docs/' + p.relative_to(root).as_posix()
        if where in SKIP:
            continue
        pages[where] = parse(p.read_text(), where)
    return pages


def link(target, current):
    return os.path.relpath(target, Path(current).parent)


def table(rows, current):
    out = ['| Date | Document | Status |', '|---|---|---|']
    for path, m in sorted(rows, key=lambda r: (str(r[1]['date'] or '9999'), r[1]['id'])):
        title = str(m['title']).replace('|', '/')
        out.append(f'| {m["date"] or "undated"} | [{title}]({link(path, current)}) | {m["status"]} |')
    return '\n'.join(out)


def write(root, path, ident, title, body):
    meta = {'id': ident, 'title': title, 'project': 'plev', 'type': 'index', 'status': 'active',
            'date': None, 'date_basis': 'generated', 'topics': ['navigation'], 'relations': [],
            'source': 'tool/tasks/docbase.py', 'source_revision': 'generated', 'revised': 'generated',
            'review': 'generated'}
    (root / path[5:]).write_text(emit(meta, f'# {title}\n\n{body}'))


def index(root):
    pages = {p: m for p, (m, _) in load(root).items() if m['type'] != 'index'}
    for home, (title, intro) in HOMES.items():
        path = f'docs/{home}/README.md'
        rows = [(p, m) for p, m in pages.items() if p.startswith(f'docs/{home}/') and not m.get('parent')]
        groups = defaultdict(list)
        for p, m in rows:
            parts = Path(p).parts
            own = parts[-1] == 'README.md'  # a page that is its own folder: a task, an experiment
            groups[parts[2] if len(parts) > 3 and not own else ''].append((p, m))
        body = intro + '\n'
        for group in sorted(groups):
            if group:
                body += f'\n## {group}\n'
            body += '\n' + table(groups[group], path) + '\n'
        write(root, path, f'index-{home}', title, body + '\n[plev docs](../README.md)')
    catalog = {'project': 'plev', 'documents': []}
    for p, (m, body) in load(root).items():
        catalog['documents'].append({'path': p, **m, 'headings': headings(body)})
    (root / 'CATALOG.json').write_text(json.dumps(catalog, ensure_ascii=False, indent=1) + '\n')


def check(root):
    errors = []
    try:
        pages = load(root)
    except (ValueError, json.JSONDecodeError) as e:
        print(e, file=sys.stderr)
        return 1
    ids = defaultdict(list)
    anchors = {}
    for path, (m, body) in pages.items():
        ids[m.get('id')].append(path)
        missing = [k for k in REQUIRED if k not in m]
        extra = [k for k in m if k not in REQUIRED + OPTIONAL]
        if missing or extra:
            errors.append(f'{path}: missing {missing} unknown {extra}')
            continue
        if m['project'] != 'plev' or m['type'] not in KINDS or m['status'] not in STATES:
            errors.append(f'{path}: type/status {m["type"]}/{m["status"]}')
        if m['date_basis'] not in BASES or m['review'] not in REVIEWS:
            errors.append(f'{path}: date_basis/review {m["date_basis"]}/{m["review"]}')
        if m['date'] is not None and not re.fullmatch(r'\d{4}(-\d{2}(-\d{2})?)?', str(m['date'])):
            errors.append(f'{path}: date {m["date"]}')
        if m['review'] != 'generated' and not re.fullmatch(r'\d{4}-\d{2}-\d{2}', str(m.get('recorded'))):
            errors.append(f'{path}: recorded, the day the page was written, is YYYY-MM-DD')
        if not isinstance(m['topics'], list) or not isinstance(m['relations'], list):
            errors.append(f'{path}: topics and relations are lists')
        if not re.search(r'^# ', body, re.M):
            errors.append(f'{path}: no title')
        anchors[path] = {h['anchor'] for h in headings(body)} | set(re.findall(r'<a id="([^"]+)"', body))
    for ident, where in ids.items():
        if len(where) > 1:
            errors.append(f'id {ident} used by {where}')
    for path, (m, body) in pages.items():
        for ref in m.get('relations', []) + ([m['parent']] if m.get('parent') else []):
            if ref not in ids:
                errors.append(f'{path}: relation {ref} names no page')
        for target in re.findall(r'!?\[[^\]\n]*\]\(([^\s)]+)\)', '\n'.join(outside_fences(body))):
            if re.match(r'[a-zA-Z][\w+.-]*:', target) or target.startswith('#') and target[1:] in anchors[path]:
                continue
            raw, _, frag = unquote(target).partition('#')
            dest = os.path.normpath(str(Path(path).parent / raw)) if raw else path
            if not (REPO / dest).exists() and not (root.parent / dest).exists():
                errors.append(f'{path}: dead link {target}')
            elif frag and dest in anchors and frag not in anchors[dest]:
                errors.append(f'{path}: dead anchor {target}')
    changelog = pages.get('docs/CHANGELOG.md', ({}, ''))[1]
    for path, (m, body) in pages.items():
        if m.get('type') not in ('index', 'section', 'manual', 'template') and path != 'docs/CHANGELOG.md' \
                and f']({path[5:]})' not in changelog:
            errors.append(f'{path}: not listed under its date in docs/CHANGELOG.md')
    numbers = defaultdict(list)
    adr_index = pages.get('docs/adr/README.md', ({}, ''))[1]
    for path, (m, body) in pages.items():
        parts = path.split('/')
        if parts[1] == 'adr' and path != 'docs/adr/README.md':
            n = re.match(r'docs/adr/([A-Za-z]+)/((\d{4})-[a-z0-9-]+)/([a-z0-9-]+)\.md$', path)
            if not n or n[1] not in THEMES:
                errors.append(f'{path}: an ADR is adr/<Theme>/NNNN-kebab/NNNN-kebab.md, '
                              f'the theme one of {sorted(THEMES)}')
                continue
            if n[4] != n[2]:
                # a page beside the record: a learning of that decision
                if m.get('type') != 'learning' or f'adr-{n[3]}' not in m.get('relations', []):
                    errors.append(f'{path}: a page in the folder of ADR {n[3]} is a learning '
                                  f'that relates adr-{n[3]}')
                continue
            numbers[n[3]].append(path)
            if m.get('id') != f'adr-{n[3]}':
                errors.append(f'{path}: the id of ADR {n[3]} is adr-{n[3]}')
            if f'({n[1]}/{n[2]}/{n[2]}.md)' not in adr_index:
                errors.append(f'{path}: not listed in docs/adr/README.md')
        if parts[1] == 'Research' and path != 'docs/Research/README.md':
            if len(parts) != 4 or parts[2] not in THEMES:
                errors.append(f'{path}: research lives in Research/<Theme>/<name>.md, '
                              f'the theme one of {sorted(THEMES)}')
        if parts[1] in ('Tasks', 'Experiments') and path != f'docs/{parts[1]}/README.md':
            if len(parts) != 4:
                errors.append(f'{path}: each one lives in its own folder, {parts[1]}/<name>/README.md')
            elif parts[3] != 'README.md':
                owner = pages.get('/'.join(parts[:3]) + '/README.md', ({}, ''))[0].get('id')
                if m.get('parent') != owner:
                    errors.append(f'{path}: a page in {"/".join(parts[:3])}/ has parent {owner}')
    errors += [f'ADR {n} used by {p}' for n, p in numbers.items() if len(p) > 1]
    catalog = root / 'CATALOG.json'
    listed = {d['path'] for d in json.loads(catalog.read_text())['documents']} if catalog.exists() else set()
    if listed != set(pages):
        errors.append('docs/CATALOG.json is stale: run tool/tasks/docbase.py index')
    for e in errors:
        print(e, file=sys.stderr)
    print(f'{len(pages)} pages, {len(errors)} errors')
    return 1 if errors else 0


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('command', choices=['index', 'check'])
    ap.add_argument('--root', type=Path, default=REPO / 'docs')
    args = ap.parse_args()
    root = args.root.resolve()
    if args.command == 'index':
        index(root)
    sys.exit(check(root))
