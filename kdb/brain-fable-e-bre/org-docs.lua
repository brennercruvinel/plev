return {
  id  = "org-docs",
  typ = "plan",
  sts = "viva",
  dom = "organization",
  dat = "2026-06-11",
  ttl = "organizacao: 3-char, doc/, mdbook, fila de workflows, loop de validacao",
  lnk = { "idx-brain", "shc-absorcao", "plev" },
  txt = [[
nomenclatura: dirs/docs/assets em kebab-case ingles; source idiomatico a
linguagem; tokens estilo apl de 3 chars para pastas/arquivos/itens onde o
token fica inequivoco e le como sintaxe de variavel (glifo atomico,
contexto natural; alto valor para leitores neurodivergentes). candidatos
em src/: sig, win, thm, txt. nunca sacrificar clareza pela contagem. a
tipagem publica do plev deve ser leve, sem densidade semantica
desnecessaria.

file hygiene DESTE projeto: limite duro 369 linhas por arquivo fonte antes
de modularizar por responsabilidade unica; alvo operacional ~220 para
arquivos novos. base: memoria de trabalho humana segura 4 +/- 1 chunks
(cowan 2001); arquivo que nao cabe na janela mental forca context switch
e sobe taxa de bug. (o contrato nest com 333/220/300 foi referencia de
estilo, nao regra daqui.)

estrutura de docs (zed mdbook como referencia, "less but better"; o zed e
denso demais): kdb/ = aprendizados/pesquisa/brain; doc/ = docs da
aplicacao: doc/arc/{arc.md humano, arc.yaml maquina, arc.mmd mermaid}
atualizados JUNTOS a cada mudanca estrutural (sem segundo documento de
arquitetura paralelo), doc/changelog.md, doc/.conventions/. mdbook na raiz
na fila (ws-org), um book diataxis, renderer custom so se o
pos-processamento exigir (padrao docs_preprocessor do zed, versao
drasticamente mais leve). AGENTS.md na raiz e a fonte UNICA de instrucao
de agentes (codex e afins leem por default; claude/gemini/cursor apontam
pra ele; sem CLAUDE.md/GEMINI.md paralelos).

fila de workflows (ordem do plano plev aprovado):
1. ws-refs: clones com study.lua embutido (ref-study)
2. ws-anim (LIDER): estudo -> spec -> codec -> rhai -> auto-animate nativo
   -> gui de autoria -> aba motion (monster-formato, edt-flash-novo)
3. ws-showcase (paralelo, arvore disjunta): abas + biblioteca com focus
   states (shc-absorcao)
4. ws-parser: tree-sitter/topiary, corpus gpui-component + 4 cards,
   verificacao tripla (parser-transpiler)
5. ws-plev-web: shell semantico, url=statechart, llm.txt, cwv
   (plev)
6. ws-org: 3-char + mdbook; ws-ide: minerar warp/gitcomet/difftastic/
   gitlogue/CodeEditSourceEditor pro basic-ide

loop de validacao continua (papel do orquestrador, pedido explicito):
reviso cada fase contra o gate antes da proxima; falha = re-queue com o
achado; verificadores adversariais MEDEM (pixel, bytes, round-trip, cwv);
heartbeat periodico fiscaliza workflows em background (licao do agente em
loop e dos 6GB de orfaos; varrer processos apos capturas); gates fixos:
cargo test --workspace + wasm check + fmt; commits tematicos do
orquestrador; push plevdev ao fim de cada fase. agentes nunca commitam.
regra de ouro de construcao: backend testado antes de ui, sem emojis,
criar e nao copiar.
]],
}
