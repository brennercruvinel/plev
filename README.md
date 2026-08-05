# Spellbook — MTG Collection & Deck Manager

Protótipo local de gestão de coleção e decks de Magic: The Gathering (mono-black Commander). Roda 100% offline depois da configuração inicial — sem Node.js, só Python (FastAPI + SQLite) e um front-end estático em HTML/CSS/JS puro.

Documentação completa das decisões de design no vault Obsidian: `MTG/App - Protótipo Spellbook.md`.

## Por que este repositório não tem os dados

- `data/` (base do Scryfall + cache do EDHREC) tem ~430 MB — acima do limite de arquivo do GitHub, e totalmente reconstruível a partir da API pública do Scryfall.
- `webapp/app.db` é a coleção/decks reais — dado pessoal, fica só local.

Ambos estão no `.gitignore`. Para rodar em qualquer máquina, reconstrua os dois com os passos abaixo.

## Setup do zero

```bash
# 1. dependências Python (sem Node.js necessário)
pip3 install --user fastapi uvicorn "pydantic<3" python-multipart pymupdf

# 2. baixar a base de cartas do Scryfall (bulk data oficial, gratuito, sem API key)
python3 scryfall.py bulk oracle_cards --download
python3 scryfall.py bulk all_cards --download   # ~370MB, nomes em PT + imagens
mv oracle-cards-*.jsonl.gz all-cards-*.jsonl.gz data/

# 3. construir o índice local (SQLite, ~14s)
python3 mtgdb.py build

# 4. (opcional) cachear sinergia do EDHREC pros seus comandantes
python3 edhrec.py fetch "Syr Konrad, the Grim" --combos
python3 edhrec.py fetch "Toshiro Umezawa" --combos

# 5. popular o banco do app com os decks (edite webapp/seed.py com sua própria coleção)
cd webapp
python3 seed.py

# 6. rodar
python3 -m uvicorn server:app --port 8420
```

Acesse **http://127.0.0.1:8420**.

## Por que não tem GitHub Pages

GitHub Pages só serve arquivos estáticos — não roda o back-end Python (FastAPI) do qual todo o app depende (decks, coleção, scanner, partidas). Rodar localmente com `uvicorn` é a forma de usar isto em qualquer computador.

## Estrutura

```
scryfall.py     — CLI cliente da API do Scryfall (busca, decklist, bulk data)
mtgdb.py        — constrói/consulta o índice SQLite local de cartas (offline)
edhrec.py       — cache de sinergia/combos do EDHREC por comandante
webapp/
  server.py     — API FastAPI (decks, coleção, scanner, partidas, histórico)
  db.py         — schema do banco do app (SQLite)
  seed.py       — popula o banco com decks/coleção reais
  static/       — front-end (HTML/CSS/JS puro, sem build)
```

## Ferramentas usadas pelo app (nenhuma exige API key)

- Base de cartas: Scryfall bulk data (38k+ cartas, nomes oficiais em PT, imagens)
- Sinergia de deck: EDHREC (cache local por comandante)
- OCR do scanner: Tesseract.js (carregado via CDN na primeira vez, depois em cache do navegador)
