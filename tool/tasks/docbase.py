#!/usr/bin/env python3
"""The docs/ tree as data: every page's header, its links and its relations.

    tool/tasks/docbase.py index   rewrite the indexes, the timeline and docs/CATALOG.json
    tool/tasks/docbase.py check   fail on a bad header, a dead link or anchor, a stale catalog

A header is YAML restricted to one `key: value` per line, where a value is
a bare word, a JSON string, `null` or an inline list `[a, "b c"]`, so the
standard library reads it and any YAML reader agrees. The fields and their
vocabularies are docs/referencia/editorial/convencoes.md.
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
OPTIONAL = ['parent', 'evidence']
KINDS = {'indice', 'registro', 'modelo', 'decisao', 'tarefa', 'aprendizado', 'programa',
         'tese', 'experimento', 'rascunho', 'manual', 'referencia', 'pesquisa', 'secao'}
STATES = {'ativo', 'referencia', 'proposto', 'aceito', 'substituido', 'pendente',
          'em-andamento', 'concluido', 'rascunho'}
BASES = {'cabecalho', 'texto', 'git', 'reconstruida', 'registro-editorial', 'sem-data', 'gerado'}
REVIEWS = {'pendente', 'revisado', 'gerado'}
HOMES = {
    'historia': ('História', 'A trajetória, os programas e os registros de cada período.'),
    'tarefas': ('Tarefas', 'Cada trabalho com objetivo, execução, entrega e o que ficou pendente.'),
    'decisoes': ('Decisões', 'Os ADRs: contexto, decisão, consequências e alternativas, numerados numa sequência só.'),
    'pesquisas': ('Pesquisas', 'Referências estudadas, comparações, a tese e os rascunhos do artigo.'),
    'experimentos': ('Experimentos', 'Método, medições e resultados, com os limites de cada um.'),
    'aprendizados': ('Aprendizados', 'O que os problemas e as implementações ensinaram.'),
    'referencia': ('Referência', 'Arquitetura de cada subsistema, operação do repositório e convenções.'),
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
    keys = REQUIRED[:5] + [k for k in ('parent',) if k in meta] + REQUIRED[5:]
    keys += [k for k in meta if k not in keys]
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
        pages[where] = parse(p.read_text(), where)
    return pages


def link(target, current):
    return os.path.relpath(target, Path(current).parent)


def table(rows, current):
    out = ['| Data | Documento | Estado |', '|---|---|---|']
    for path, m in sorted(rows, key=lambda r: (str(r[1]['date'] or '9999'), r[1]['id'])):
        title = str(m['title']).replace('|', '/')
        out.append(f'| {m["date"] or "sem data"} | [{title}]({link(path, current)}) | {m["status"]} |')
    return '\n'.join(out)


def write(root, path, ident, title, body):
    meta = {'id': ident, 'title': title, 'project': 'plev', 'type': 'indice', 'status': 'ativo',
            'date': None, 'date_basis': 'gerado', 'topics': ['navegacao'], 'relations': [],
            'source': 'tool/tasks/docbase.py', 'source_revision': 'gerado', 'revised': 'gerado',
            'review': 'gerado'}
    (root / path[5:]).write_text(emit(meta, f'# {title}\n\n{body}'))


def index(root):
    pages = {p: m for p, (m, _) in load(root).items() if m['type'] != 'indice'}
    for home, (title, intro) in HOMES.items():
        path = f'docs/{home}/README.md'
        rows = [(p, m) for p, m in pages.items() if p.startswith(f'docs/{home}/') and not m.get('parent')]
        groups = defaultdict(list)
        for p, m in rows:
            parts = Path(p).parts
            groups[parts[2] if len(parts) > 3 else ''].append((p, m))
        body = intro + '\n'
        for group in sorted(groups):
            if group:
                body += f'\n## {group}\n'
            body += '\n' + table(groups[group], path) + '\n'
        write(root, path, f'indice-{home}', title, body + '\n[Acervo plev](../README.md)')
    years = defaultdict(list)
    for p, m in pages.items():
        if not m.get('parent') and m['type'] not in ('modelo', 'manual'):
            years[str(m['date'])[:4] if m['date'] else 'sem data'].append((p, m))
    body = ('Os documentos pela data do que registram. A data vem do cabeçalho de cada página, '
            'com a base em `date_basis`; [cronologia e fontes](cronologia-e-fontes.md) explica '
            'onde uma data foi revista depois do fato.\n')
    for year in sorted(years):
        body += f'\n## {year}\n\n' + table(years[year], 'docs/historia/linha-do-tempo.md') + '\n'
    write(root, 'docs/historia/linha-do-tempo.md', 'linha-do-tempo', 'Linha do tempo', body)
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
    numbers = defaultdict(list)
    for path in pages:
        m = re.match(r'docs/decisoes/[a-z]+/(\d{4})-[a-z0-9-]+\.md$', path)
        if path.startswith('docs/decisoes/') and not path.endswith('README.md'):
            if not m:
                errors.append(f'{path}: a decision is decisoes/<tema>/NNNN-kebab.md')
            else:
                numbers[m[1]].append(path)
    errors += [f'decision {n} used by {p}' for n, p in numbers.items() if len(p) > 1]
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
