---
type: reference
tags: [learnings, session, orchestration, agents, devops, secops, text, accessibility, wgpu, llm, privacy]
date: 2026-09-26
status: reference
description: a sessão longa do programa do runtime generativo (2026-09-23 a 2026-09-26), dos PRs #44 a #127. o que funcionou e o que mordeu, para quem orquestra agentes, mexe no engine ou valida o trabalho depois.
---

# sessão do runtime generativo: o que funcionou e o que mordeu

registro da sessão que tocou o programa do runtime de interação generativa
do começo das ondas até a task-78 fase 2: contrato semântico, porta do
agente, gravação e inspector, IR e superfícies geradas, texto, teclado,
wgpu 29 e inferência em processo. escrito pra quem vier depois, humano ou
agente. as decisões estão nos ADRs; aqui fica o caminho.

## orquestração: o caminho que funcionou

1. **fechar antes de abrir.** o brenner parou o ciclo automático porque o
   orquestrador abria fase nova de cada task que integrava e virava
   resultado de demo em task nova, a 500 a 660 mil tokens por agente. as
   regras que vieram (`docs/mission/autonomy.md`, "contra gasto de
   tokens") pagaram: cada briefing diz o item do critério que fecha; o que
   sobra vira pendência; task nova só com o ok dele; um agente por vez,
   com o custo no journal.
2. **ler o critério de aceite antes de dizer o que falta.** a revisão das
   pendências achou três coisas que eu tinha posto na conta do brenner e
   eram trabalho de agente: o código do IME no campo, a ponte do leitor no
   ide e o papel de splitter no `sem`. "depende do brenner" é só a parte
   que é dele (o roteiro manual, a escolha, a instalação).
3. **a sonda antes do código, com arquivo:linha na versão travada.** foi
   ela que achou o COLRv1 que o swash 0.2.8 não desenha (daí o emoji em
   CBDT), o burn sem llama (daí o código trazido do burn-lm) e o wgpu 29
   resolvendo offline antes da subida.
4. **o orquestrador valida sozinho e abre as imagens.** o diff inteiro, o
   gate, o regress e o PNG aberto. foi olhando a entrega da task-75 fase
   2d que apareceu a regressão para quem usa leitor de tela (a parada
   única de Tab tirava o `Focus` do leitor), e ela voltou antes de entrar.
5. **decidir o que é do orquestrador.** o agente da subida do wgpu parou
   por uma linha no `wire` (um lint, o mesmo comportamento). não é
   contrato; o orquestrador aprova e segue, sem levar ao brenner.
6. **uma coisa por vez, também dentro da rodada.** depois do travamento,
   o candle até o fim, depois o burn, depois o wasm. a rodada terminou
   limpa e na memória.

## orquestração: o que mordeu

- afirmar sem ler o caminho inteiro: eu disse que o editor não desenhava a
  composição do IME porque um grep curto não achou; o `render_preedit_line`
  existia. antes de afirmar ausência, ler o render.
- o texto de marketing no README ("pixels idênticos em todo alvo") durou
  até a medição: no navegador são 79 de 82 páginas, e as outras três
  mostram outro conteúdo de propósito. número medido no lugar de adjetivo.
- `git add -A` num commit do orquestrador, contra a regra do how-to.
  caminhos explícitos, sempre.
- o `PLEV_BACKGROUND=1` abre a janela sem ícone no Dock e sem foco: quando
  a pessoa quer ver, a janela abre sem ele.

## máquina (DevOps)

- **memória.** o Mac tem 16 GB e travou compilando o candle e o burn com o
  modelo rodando. a regra: `CARGO_BUILD_JOBS=4`, nunca compilar e rodar o
  modelo juntos, um processo com o modelo por vez, `memory_pressure | tail
  -1` antes de cada rodada (abaixo de 40% livres, esperar).
- **a sessão pode cair com um agente no meio.** os commits dele ficam na
  branch; retoma-se o mesmo agente por mensagem, contando o que está sem
  commit.
- **disco.** `cargo clean --dry-run` antes do `cargo clean`; o cargo
  recusa limpar uma pasta sem `CACHEDIR.TAG` (um `target/` criado por um
  comando sem `CARGO_TARGET_DIR` dentro do worktree), e a lixeira não
  devolve espaço até ser esvaziada. limpar `target/` inteiro leva junto o
  `target/fonts-noto` (as fontes-fonte do `script/font-subset`): tirar
  antes e devolver depois.
- **o que se baixa fica fora do repositório e com sha256 registrado.** o
  modelo em `~/.cache/plev/models/` sumiu numa limpeza de cache; com o
  sha256 e o commit do Hugging Face no arquivo da task, o download de
  novo se confere. o `.fig` do Figma fica fora do repo (`*.fig` no
  `.gitignore`).
- **offline.** os agentes compilam com `--offline`. antes de subir uma
  dependência, conferir `cargo fetch --offline --target <alvo>` para
  cada alvo. se `~/.cargo/registry/{cache,src}` some, nenhum build
  offline passa até um `cargo fetch` com rede.
- **CI parado (billing do Actions desligado).** o gate local vale, e o PR
  diz isso; merge commit, nunca squash nem rebase.

## privacidade e segurança (SecOps)

- o que é da pessoa não vira estado do app: a composição do IME só se
  desenha, a área de transferência só a pessoa alcança. e o que a
  interface mostra também vaza: um Paste desligado contava ao agente se a
  área de transferência estava vazia (adr
  the-persons-text-and-clipboard-stay-the-persons).
- promessa de privacidade se testa no conjunto inteiro: o teste lê os
  bytes do arquivo de gravação e não acha a composição nele.
- nenhum agente chama o `claude -p` (`PLEV_CLAUDE` aponta para um stub em
  todo comando de agente, gate e regress).
- credencial colada no chat nunca se repete, se guarda nem se usa.

## arquitetura: padrões que ficam

- **o contrato é o mapa.** o teclado anda na ordem da projeção, a mesma do
  leitor de tela, e age pelo dispatcher como um clique (adr
  the-keyboard-walks-the-projection). um segundo mapa da página seria
  outra fonte de verdade.
- **quando o widget não tem política por item, a recusa vem no executor.**
  funciona, mas a projeção mente um pouco (o agente vê o item como
  acionável); a dívida fica escrita.
- **subir o renderer prova pixel por captura completa**, não pelo `regress
  check`, que não desenha frame (adr a-renderer-upgrade-is-proven-by-a-full-capture).
- **texto: visual na linha, lógico entre linhas; posição em f64 e shaping
  por linha num cache com orçamento** (adr
  text-moves-by-screen-order-and-shapes-by-line).
- **fontes e formatos se escolhem pelo que a versão travada desenha e
  lê.** o emoji é CBDT porque o swash 0.2.8 não desenha COLRv1; o `.fig`
  é um zip com fig-kiwi (esquema em deflate, mensagem em zstd, instâncias
  expandidas do SYMBOL), lido offline por `script/figma-fixture`.
- **inferência: uma interface, duas bibliotecas, o mesmo modelo.**
  decodificação gulosa e os 32 primeiros tokens lado a lado dizem se a
  comparação é justa; o burn não tem llama e o código trazido do burn-lm
  carrega uma cadeia de licença a olhar antes de publicar (adr
  ai-native-single-process).
- **dividir o device da GPU pede o mesmo wgpu dos dois lados**, o
  `instance` e o `adapter` expostos, e limites de computação pedidos só
  por quem computa (`RenderConfig::compute`, desligado por padrão).
