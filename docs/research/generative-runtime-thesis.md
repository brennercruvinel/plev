---
id: generative-runtime-thesis
title: "plev: thesis of the generative interaction runtime"
project: plev
type: thesis
status: active
date: 2026-09-23
date_basis: header
recorded: 2026-09-23
topics: [semantics, agents, ir, recording, generated-surfaces]
relations: [generative-runtime-program, operations-generative-runtime-guidelines, task-57, task-58, task-59, task-60, task-61, task-62, task-63, task-64, task-65, task-66, task-67, task-68, task-69, task-70, task-71, task-72, task-73, task-74, task-75, task-76, task-77, task-78, task-79, task-80]
source: docs/ref/generative-runtime-thesis.md
source_revision: ec0a224c
revised: 2026-10-30
review: reviewed
---

# plev: thesis of the generative interaction runtime

este é o documento-base da direção. junta, sem resumir, quatro fontes:
a síntese das mentorias (tese, arquitetura, princípios e superfície
técnica), as anotações do brenner, o arquivo de concepção da seção plev
(ideias de inovação, ideias de implementação, experimentos, contexto e
limites) e a rodada de ideias de funcionalidade que a arquitetura já paga.
não é especificação fechada nem roadmap: é a tese técnica sobre o que o
plev já contém em embrião e no que essa arquitetura pode se transformar.
o resultado é a formulação curta da tese: o plev transforma intenção em
interface, e o critério que a sustenta é que humano e agente operam a mesma
identidade lógica, com ações verificadas e registradas. importa porque
separa o que já existe no repositório do que ainda é hipótese a demonstrar
(seção 28).

o plano executável (goal, grafo, ordem, contratos entre tasks) está em
[`generative-runtime-program.md`](generative-runtime-program.md). as regras de trabalho do orquestrador e
dos agentes estão em [`generative-runtime-guidelines.md`](../ref/operations/generative-runtime-guidelines.md). cada goal virou
uma task em `docs/log/steps/` (task-57 a task-80).

a formulação curta:

> plev transforma intenção em interface.

a tese completa:

> plev transforma intenção, estado e capacidades em interação nativa,
> segura, dinâmica, compreensível e verificável.

---

## 1. interfaces não deveriam ser telas fixas

durante décadas, software foi construído a partir de uma premissa quase
invisível:

> primeiro desenhamos a interface; depois encaixamos as capacidades do
> software dentro dela.

criamos telas, menus, toolbars, painéis, formulários e fluxos. o usuário
aprende aquela organização e passa a operar o sistema através dela. mesmo
quando o contexto muda, a interface permanece praticamente a mesma.

hoje, normalmente:

```text
produto
  ↓
designer cria telas
  ↓
dev implementa componentes
  ↓
interface fica relativamente estática
  ↓
usuário aprende a operar aquela interface
```

a entrada de modelos de linguagem e agentes muda essa relação. um sistema
agora pode compreender intenção, contexto, estado, objetivo e capacidades
disponíveis. se isso é verdade, a interface não precisa mais ser
inteiramente determinada antes da execução. ela pode ser consequência do
que está acontecendo.

a inversão fundamental:

```text
modelo tradicional

UI
↓
ações
```

para:

```text
plev

intenção
+
estado
+
capacidades
+
restrições
↓
interface
```

a UI deixa de ser necessariamente a origem da interação. ela pode se
tornar seu resultado. a representação visual muda; a identidade da
capacidade não.

o ciclo invertido, de ponta a ponta:

```text
intenção do usuário
        ↓
modelo / agente entende contexto
        ↓
gera ou reorganiza a interface necessária agora
        ↓
plev materializa a cena
        ↓
GPU renderiza
        ↓
usuário interage
        ↓
estado muda
        ↓
interface se recompõe
```

a direção: transformar intenção em interface. não apenas desenhar
interfaces mais rápidas. não ser React em Rust. não ser Flutter em Rust.
construir um runtime onde interfaces possam ser materializadas,
modificadas, compreendidas, operadas e inspecionadas enquanto o software
está vivo.

competir dizendo "somos Flutter em Rust" seria uma batalha péssima e não é
particularmente inovador: o Flutter tem uma organização inteira, anos de
trabalho e um ecossistema gigantesco. dizer "vamos construir a camada
gráfica determinística, compilável e verificável onde humanos e agentes
constroem software nativo juntos, sem DOM" é outra coisa. e vários
componentes que já existem no repositório (parser de outras UIs,
contracts para agentes, design tokens centralizados, formato binário de
animação, render determinístico, saída de pixel cross-platform) apontam
meio naturalmente nessa direção.

a trajetória conceitual:

```text
plev hoje        GPU UI engine
     ↓
plev amanhã      native application runtime
     ↓
plev depois      semantic interaction runtime
     ↓
plev ambicioso   generative interaction runtime
```

a última versão é a que interessa. o posicionamento em três degraus, nas
palavras das anotações:

```text
plev = uma engine indie de UI GPU-first muito competente
plev = toolkit rust multiplataforma utilizável, runtime para aplicações nativas
plev = runtime de UI para a era de agentes
```

---

## 2. interface é dado até o fim

o plev parte de uma propriedade estrutural particularmente importante: a
interface permanece representável como dados em vez de desaparecer dentro
de uma coleção opaca de objetos, callbacks e estado de framework.

a cena é um vetor de nós puros:

- hasheado por layer;
- gravável em binário com delta;
- renderizável sem janela;
- determinístico em quatro plataformas.

isso cria uma propriedade diferente de frameworks onde a interface acaba
representada como uma árvore de objetos com comportamento e closures
espalhados pela runtime.

a importância disso cresce com agentes operando interfaces. hoje grande
parte da automação visual funciona através de screenshots, visão,
coordenadas e interpretação aproximada. isso é caro, indireto e difícil de
auditar. no plev, um agente poderia trabalhar diretamente sobre a
estrutura utilizada pela própria engine.

essa propriedade permite uma arquitetura diferente:

```text
intenção
   ↓
modelo / agente
   ↓
plev IR
   ↓
compiler + verifier
   ↓
semantic scene
   ↓
compositor
   ↓
GPU
```

um modelo não deveria precisar escrever Rust, WGSL ou manipular
arbitrariamente a engine. ele deveria produzir ou transformar uma
representação limitada, tipada e verificável. a engine mantém autoridade
sobre o que pode existir e o que pode ser executado.

isso é essencial. LLMs são bons em composição e interpretação. não
deveriam ganhar autoridade irrestrita sobre memória, rendering, input ou
efeitos colaterais apenas porque são capazes de gerar código. o sistema
precisa separar geração de execução:

```text
o modelo propõe.
o runtime verifica.
o plev executa.
```

nas anotações, a mesma ideia em forma de pipeline:

```text
LLM
 ↓
component tree tipada
 ↓
compiler/verifier
 ↓
plev
```

em vez de agentes produzirem DOM/CSS arbitrário, eles produzem uma
estrutura limitada, verificável e determinística. você não quer um LLM
escrevendo código arbitrariamente a cada frame; quer o modelo montando
estruturas permitidas dentro de uma linguagem de interface segura.

---

## 3. plev IR

uma peça central dessa direção é uma representação intermediária própria.
não uma AST de Rust. não DOM. não HTML virtual. uma representação de
interface do plev.

```text
React ───────┐
SwiftUI ─────┤
Flutter ─────┤
HTML/CSS ────┤
Figma ───────┤
JSON schema ─┤
LLM ─────────┤
DSL ─────────┤
descrição de app existente
             ↓
           plev IR
             ↓
      semantic scene
             ↓
           plev
             ↓
            GPU
```

o que as anotações chamam de "o que vamos construir":

```text
Figma
   ↓
plev IR
   ↑
React importer
   ↑
LLM
   ↓
live application
```

o parser que já existe aponta nessa direção: ele transforma UI de outro
framework em código builder do plev, mapeia estilos para theme tokens e
reporta as construções que não consegue representar em vez de ignorá-las.

o plev IR não representa somente aparência. não basta:

```text
rectangle
text
padding
color
```

ele precisa preservar significado:

```text
id: search.query
role: input

state:
  value: "gpu renderer"

actions:
  - submit
  - clear

constraints:
  editable: true
  required: false
```

assim, a aparência pode mudar sem destruir a identidade da capacidade. um
campo pode aparecer como caixa de texto, command palette, componente
flutuante ou outra representação ainda não prevista. ele continua sendo
`search.query`. a apresentação é variável; a semântica permanece. essa
distinção é fundamental para interfaces generativas.

com isso, o LLM não precisa escrever Rust nem shader. ele produz uma
descrição validada:

```text
intenção
↓
UI IR
↓
validator
↓
plev
↓
GPU
```

---

## 4. a interface deixa de ser fixa

imagine um editor de vídeo. hoje ele nasce com:

```text
timeline fixa
toolbar fixa
inspector fixo
menus fixos
painéis fixos
controles
```

tudo existe antecipadamente, mesmo que você esteja fazendo uma tarefa só.

agora imagine:

> corta os silêncios desse vídeo e depois quero revisar os cortes.

o modelo entende a intenção. o sistema possui as capacidades:

```text
detect_silence
preview_segment
change_threshold
apply_cut
reject_cut
```

o plev materializa naquele momento uma superfície:

```text
┌──────────────────────────────────────────────┐
│ vídeo                                        │
│                                              │
├──────────────────────────────────────────────┤
│ 17 silêncios encontrados                     │
│                                              │
│ [preview] [preview] [preview] [preview]      │
│                                              │
│ intensidade do corte   ─────●────            │
│                                              │
│ [aplicar todos]   [revisar individualmente]  │
└──────────────────────────────────────────────┘
```

essa interface não precisava existir no código como uma
`SilenceReviewScreen`. talvez nunca tenha existido antes. ela apareceu
porque aquele estado e aquela intenção a tornaram adequada. terminou a
tarefa, ela desaparece.

depois o usuário pede:

> quero comparar a versão original com os cortes.

o runtime reorganiza:

```text
┌──────────────────────┬──────────────────────┐
│ original             │ resultado            │
│                      │                      │
│                      │                      │
├──────────────────────┴──────────────────────┤
│            diferenças na timeline           │
└─────────────────────────────────────────────┘
```

terminada a comparação, essa composição pode desaparecer. não existe mais
necessariamente uma aplicação com "telas". existe:

```text
estado + intenção + capacidades disponíveis → interface adequada
```

isso não significa que toda interface precise ser gerada. interfaces
permanentes continuam fazendo sentido. o plev precisa permitir um
espectro:

```text
interface fixa
       ↓
interface parametrizada
       ↓
interface adaptativa
       ↓
interface recomposta
       ↓
interface gerada sob demanda
```

a inovação está em fazer todas elas pertencerem ao mesmo runtime.

você deixa de programar:

```text
TelaHome
TelaSettings
TelaSearch
TelaResults
```

e passa a fornecer:

```text
capacidades
dados
ações
restrições
semântica
```

e o sistema constrói a interação necessária.

---

## 5. LLM como interface semântica, plev como interface perceptual

modelos de linguagem são excelentes para intenção. são ruins para algumas
propriedades fundamentais da interação humana. mesmo com modelos
extremamente bons, texto e voz não substituem bem:

- comparação visual;
- navegação e manipulação espacial;
- edição;
- manipulação direta;
- ajuste contínuo;
- acompanhamento simultâneo de múltiplos objetos;
- edição gráfica;
- visualização de relações;
- inspeção;
- conferência e confirmação antes de executar operações críticas.

não faz sentido transformar tudo em conversa. o modelo pode entender
"quero diminuir essa curva exatamente aqui". mas você não vai querer
dizer "aumente o ponto 37 da curva bezier em 4%". é melhor pegar o ponto e
arrastar.

portanto:

```text
LLM  = interface semântica
plev = interface perceptual e interativa
```

o modelo entende o objetivo. o plev materializa a superfície adequada
para compreender, verificar ou manipular aquele objetivo. essa
complementaridade é mais interessante do que imaginar que chat vai
substituir todas as interfaces.

não é só um renderer em Rust. é uma espécie de DOM novo, desenhado para
aplicações nativas, GPU e agentes, sem carregar os 30 anos de legado do
browser.

---

## 6. a mesma interface para humano e máquina

hoje um app é basicamente uma tela que o humano usa. o agente, quando
entra, precisa "olhar" essa tela por screenshot, DOM, accessibility tree
ou algum adaptador. frequentemente existem várias representações da mesma
aplicação:

```text
interface visual
accessibility tree
selectors de automação
test fixtures
analytics
agent tools
trace
```

elas podem divergir. um botão que visualmente parece correto pode não
existir adequadamente para acessibilidade. um teste pode procurar um
selector que não representa a semântica real. um agente pode operar
coordenadas. um recorder registra eventos em outra nomenclatura.

a oportunidade não é simplesmente permitir que agentes operem sem
screenshots. é fazer renderização, interação humana, automação por
agentes, acessibilidade, testes e auditoria derivarem do mesmo contrato
fundamental da interface. o agente lê a mesma estrutura usada pela
engine. o leitor de tela deriva sua representação dessa mesma interface.
o teste compara a mesma estrutura. a sessão registrada referencia as
mesmas identidades. acessibilidade, automação e auditoria deixam de ser
três subsistemas acoplados depois e passam a ser propriedades do formato.

o botão "criar commit":

```text
identity:
  commit.submit

role:
  button

state:
  enabled: true
  message: "fix parser"
  selected_files:
    - parser.rs
    - lexer.rs

actions:
  - commit
```

visualmente isso gera:

```text
background
border
shadow
icon
text
```

para humanos, é um botão visual. para o agente, é:

```text
id: commit.submit
estado: habilitado
arquivos selecionados: 3
mensagem: "fix parser"
ação possível: criar commit
```

para o leitor de tela, é um controle acessível. para o sistema de testes,
é um objeto verificável. para o recorder, é uma ação que pode ser
registrada e uma identidade persistente. tudo isso é o mesmo controle
lógico, não cinco sistemas separados.

a meta: um sistema onde a própria interface já nasce entendível e
operável por humano, agente, teste e leitor de tela ao mesmo tempo.

### a mesma estrutura não é a mesma árvore literal

cada interação deveria poder ser compreendida, executada e revisitada
através da mesma identidade. o controle que aparece, o significado que
expõe, a ação que aceita e o registro histórico daquela ação pertencem à
mesma interface lógica.

isso não significa que renderização e semântica precisam usar exatamente a
mesma árvore física. um botão gera primitivas diferentes (fundo, sombra,
texto, ícone) no vetor de desenho. para automação e acessibilidade,
continua sendo um único botão. essas representações ficam ligadas por uma
identidade comum.

o componente declara uma vez:

> sou este controle, tenho este significado, estou neste estado e aceito
> estas ações.

depois a renderização consome sua parte, a acessibilidade consome sua
parte, a automação consome sua parte e a gravação consome sua parte. não é
necessário colocar semântica de botão em cada primitiva enviada à GPU. o
vetor enxuto que o renderizador já consome fica preservado.

---

## 7. identidade não pode depender da posição

uma engine generativa torna identidade persistente ainda mais importante.
se a interface pode se reorganizar, isto não pode identificar um controle:

```text
terceiro botão
da segunda coluna
na coordenada x,y
```

a representação visual pode desaparecer e retornar, mudar de posição,
mudar de tamanho, ser substituída por outra representação. a capacidade
continua sendo `commit.submit`.

o plev precisa distinguir claramente:

```text
identidade lógica persistente

identidade temporária de renderização

posição atual na cena
```

essa separação é o que permite unir interface generativa, agentes,
accessibility, replay e testes sem transformar coordenadas temporárias em
identidade.

---

## 8. agir sobre estado conhecido, não sobre estado presumido

agentes criam um problema novo. entre observar uma aplicação e decidir
agir, o mundo pode mudar.

imagine um git client feito em plev. você está mexendo no app. ao mesmo
tempo, um agente está trabalhando. o agente observa:

```text
selected_files = [A, B, C]
message = "fix login"
commit.submit = enabled
```

ele decide criar o commit. enquanto o modelo raciocina, você desmarca B:

```text
selected_files = [A, C]
```

a observação ficou velha. num sistema comum, o agente continua, pressiona
commit e faz merda: o sistema executa uma intenção baseada em um estado
que já não existe.

no plev, a ação carrega precondições:

```text
execute commit.submit

if:
  selected_files == [A, B, C]
  message == "fix login"
```

em linguagem de gente: crie o commit, mas somente se os mesmos 3 arquivos
continuarem selecionados e a mensagem continuar igual.

a engine verifica o estado atual. se mudou:

```text
rejected

reason:
  selected_files changed

delta:
  removed: B
```

ou, na forma falada: ação recusada, estado mudou, o arquivo B foi removido
da seleção, aqui está a diferença. o agente pode então reconsiderar.

se nada relevante mudou:

```text
accepted

result:
  commit: abc123
```

ou: ação aceita, operação concluída, commit criado `abc123`, com estes
arquivos.

isso transforma automação numa interação condicionada pelo estado
observado. evita que um agente aja silenciosamente sobre um mundo que já
mudou. e permite que humano e agente usem a mesma interface ao mesmo
tempo. a interface deixa de ser apenas uma superfície gráfica e começa a
adquirir propriedades próximas de um sistema transacional.

---

## 9. action receipts

toda ação relevante deveria poder produzir um recibo. não apenas:

```text
clicked button
```

mas:

```text
requested
↓
validated
↓
accepted / rejected
↓
executed
↓
completed / failed
↓
result
```

exemplo:

```text
action:
  commit.submit

actor:
  agent:reviewer

observed_state:
  selected_files: 3
  message: "fix parser"

preconditions:
  selection unchanged
  message unchanged

validation:
  passed

execution:
  completed

result:
  commit: abc123
```

o recibo separa pedido, aceitação e conclusão, inclusive quando uma
operação demora ou falha. isso é especialmente importante para operações
assíncronas, demoradas, destrutivas e auditáveis. melhora agentes, mas não
serve só para agentes: serve para debugging, auditoria, testes,
aplicações críticas, colaboração e suporte.

o princípio fundamental é que invalidação seja semântica. as condições
consideram apenas o estado semanticamente relevante. uma animação de hover
não deveria tornar uma intenção obsoleta. uma mudança na seleção de
arquivos deveria.

---

## 10. humano e agente compartilham o software de verdade

a consequência não é simplesmente um "copilot". humano e agente podem
alternar a condução da mesma tarefa:

- o agente prepara uma operação;
- o humano assume naquele estado;
- o humano altera parte do estado;
- o agente continua;
- ou o humano entrega o controle de novo.

o runtime sabe:

```text
quem observou
quem propôs
quem modificou
quem executou
qual estado existia
qual resultado ocorreu
```

o chão comum é a identidade estrutural da interface. esse é um modelo de
software bem diferente daquele em que "agent mode" é uma camada externa
simulando mouse e teclado. você pode continuar usando a interface enquanto
o agente trabalha, sem que as ações dele se apoiem silenciosamente num
mundo que já mudou.

para leigo: um sistema operacional de interação dentro do aplicativo. não
um sistema operacional tipo Windows, mas uma camada que sabe:

```text
o que existe na tela
o que cada coisa significa
qual é o estado atual
quem pode agir
o que pode ser feito
quem fez
o que aconteceu depois
```

em termos de produto: o plev é uma forma de construir apps em que a
interface não é só visual. ela é entendível, operável e auditável por
máquinas desde o nascimento. esse é o salto. não é "React mais rápido",
não é "Flutter em Rust". é mais próximo de uma nova camada de aplicação
feita para software onde humanos e agentes trabalham juntos.

---

## 11. monster como memória da interação

o `.monster` pode deixar de ser apenas uma representação otimizada de
animação. hoje ele é pequeno, mas precisa de mais atenção. o formato
binário nasceu assim: lottie entra uma vez e é convertido, keyframes
completos permitem seek O(1), entre keyframes só são armazenados deltas, e
a aplicação não carrega runtime lottie. isso é exatamente a filosofia do
plev:

```text
formato externo complexo
       ↓ build time
representação otimizada própria
       ↓ runtime
GPU
```

para interação, o monster pode evoluir para um flight recorder. não vídeo.
dados:

```text
14:02:01
human selected file A

14:02:04
agent observed commit state

14:02:06
human removed file B

14:02:07
agent requested commit

14:02:07
request rejected
reason: selection changed
```

depois:

```text
open session
↓
scrub timeline
↓
select control
↓
inspect semantic state
↓
inspect action
↓
inspect actor
↓
inspect result
```

cada sessão pode gerar um `.monster` navegável com scrub temporal. a
experiência: você pausa no instante anterior ao commit, seleciona o botão e
encontra o estado que o agente observou, a ação solicitada, sua origem e o
resultado. com a ligação ao código, chega também à implementação
correspondente àquela versão do aplicativo.

o arquivo não pode guardar apenas como desenhar de novo os frames. precisa
conseguir explicar o que aconteceu.

### replay e reexecução são promessas diferentes

replay: reconstruir o que foi apresentado, os estados, as ações, os
resultados e a origem das ações.

reexecução: restaurar também estado externo, relógio, arquivos, rede,
APIs, serviços, respostas assíncronas e efeitos colaterais, e executar tudo
de novo.

uma cena determinística ajuda muito no replay. sozinha, não garante
reexecução. a segunda exige controlar filesystem, rede, relógio, APIs e
toda fonte externa de estado, e não pode ser prometida implicitamente pela
primeira. a primeira promessa é replay inspecionável: assistir no player à
criação de um commit não cria outro commit.

---

## 12. bug report deixa de ser mídia

hoje: "manda um vídeo do bug". no plev: "manda os últimos 20 segundos da
sessão", ou "exporta o trecho". o replay é estruturado, não apenas visual.

o desenvolvedor abre exatamente o instante do problema e pergunta:

```text
qual controle era esse?
qual era seu estado?
qual foi a sequência anterior?
qual ação ocorreu?
quem iniciou?
qual era a precondição?
o que mudou?
qual foi o resultado?
```

### sessão vira teste

uma gravação serve como matéria-prima para um teste. o desenvolvedor
escolhe quais estados e resultados do replay viram asserções:

```text
sessão real
↓
selecionar comportamento relevante
↓
gerar assertions
↓
teste de regressão
```

o runtime registra comportamento observado, e esse comportamento pode se
transformar em especificação.

---

## 13. pixel para código

o plev controla a construção da interface e o hot reload já liga elementos
visuais ao código. preservando informação suficiente sobre origem, uma
interação visual pode navegar até sua implementação:

```text
cmd + click no texto
↓
elemento visual
↓
qual controle é esse (identidade)
↓
qual código gerou (source mapping)
↓
arquivo.rs
↓
linha correspondente
```

um "view source" para pixels. o baseline do hot reload de hoje sabe cada
literal por arquivo; com a posição de cada literal (proc-macro2 dá span
com span-locations), cmd+clique num texto ou numa cor na janela aberta
chega em `TextNodeKey` ou cor, daí no literal, daí em arquivo:linha:coluna
no editor. o dioxus só faz isso com macro; aqui dá pra fazer sem, em
qualquer app.

o inverso também se torna possível em certos casos:

```text
editar texto visualmente
↓
resolver literal de origem
↓
modificar fonte
↓
git diff
```

é o caminho inverso do hot reload: o override já é velho para novo, a
posição é conhecida, então escrever a string nova no arquivo é um replace
num span. copy de produto, label de widget, texto do showcase editado ao
vivo por quem não abre editor, e o resultado é um diff git normal. o mesmo
vale para cor, com um color picker.

no replay, selecionar um controle leva ao código que o produziu naquela
versão do app:

```text
estado observado -> controle -> identidade -> implementação
```

isso transforma o inspector em ferramenta de desenvolvimento, não apenas
de diagnóstico.

### quem leu este token

todo widget lê de `Theme`. registrar leitura de token por widget durante um
frame dá de graça o mapa token → widget → página. no showcase: hover num
widget mostra os tokens que ele usa; clicar num token mostra tudo que ele
pinta. é a documentação do design system que ninguém escreve.

---

## 14. plev como compiler target

o transpiler não deveria existir apenas para converter React em plev. isso
seria útil para migração, mas pequeno. ele pode virar uma espécie de
front-end compiler universal de UI, e o objetivo maior é o plev ser um
target:

```text
             ┌ React
             ├ HTML/CSS
             ├ SwiftUI
             ├ Flutter
             ├ Figma
             ├ LLM
             ├ DSL
             └ tooling
                 ↓
               plev IR
                 ↓
               verifier
                 ↓
               runtime
                 ↓
                 GPU
```

importers não precisam preservar cada detalhe arbitrário da origem.
precisam representar com clareza:

```text
suportado
transformado
aproximado
impossível
```

nunca fingir equivalência silenciosa. o parser atual já segue essa
filosofia: mapeia as construções reconhecidas e reporta, com arquivo e
linha, o que não consegue representar. e o runtime nunca precisa carregar
os modelos de execução de origem.

---

## 15. interface generativa precisa de restrições

uma UI criada dinamicamente não pode significar "o modelo desenha qualquer
coisa". isso destruiria consistência, segurança e usabilidade. a geração
acontece dentro de um espaço formal. o sistema fornece:

```text
componentes permitidos
ações possíveis
design tokens
constraints
layout primitives
capabilities
políticas de accessibility
permissões
```

o agente escolhe uma composição. o verifier garante que ela é válida. essa
relação é muito mais próxima de:

```text
LLM
↓
program synthesis
↓
typed IR
↓
verification
↓
execution
```

do que de:

```text
LLM
↓
HTML aleatório
```

esse pode ser um diferencial estrutural importante do plev.

---

## 16. interface adaptativa, não apenas generativa

o modelo pode não gerar a interface inteira. ele pode fazer
transformações, e a interface se transforma sem ser reconstruída do zero.

entrada:

```text
interface atual
+
estado
+
objetivo
+
contexto
```

saída:

```text
transformação segura
```

exemplos:

> tem informação demais aqui.

o sistema remove elementos irrelevantes, reduz densidade, agrupa controles,
destaca somente o estado relevante, aumenta o foco da tarefa atual.

> não consigo comparar esses números.
> tenho dificuldade para enxergar esses números.

a interface reorganiza: fonte maior, contraste diferente, menos
densidade, gráficos diferentes, outra visualização.

ou o sistema percebe que o usuário repete sempre esta sequência:

```text
review
approve
publish
```

e oferece uma capacidade composta:

```text
[aprovar e publicar]
```

que representa uma composição de ações. isso é interface adaptativa de
verdade, não responsive design.

---

## 17. aplicações AI-native

essa arquitetura combina particularmente bem com software que já nasce em
torno de IA local ou híbrida. imagine uma aplicação com:

```text
LLM local
modelo
embeddings
áudio
vídeo
canvas
milhares de tokens
streaming
visualizações
GPU compute
GPU rendering
local storage
```

em React/Electron, frequentemente você termina com:

```text
Chromium
+
JS heap / JS runtime
+
DOM
+
IPC
+
native backend
+
GPU
```

o plev pode oferecer outro modelo:

```text
um processo rust
+
runtime da aplicação
+
modelos
+
dados
+
renderização
```

isso não significa que toda aplicação deva abandonar tecnologias web.
significa que o plev é especialmente interessante onde memória, latência,
GPU, streaming e interação rica fazem o custo das abstrações intermediárias
importar. o foco é a próxima geração de desktop AI. a formulação das
anotações vai além ("talvez para além das fronteiras da arquitetura de von
neumann"); isso fica registrado como horizonte de pesquisa, sem critério de
aceite no programa.

---

## 18. GPU-first não é estética

a GPU não deveria ser apenas argumento de performance. ela é parte da
possibilidade de interfaces que se recompõem rapidamente. se a interface
pode mudar constantemente, você não quer ficar produzindo:

```text
HTML
↓
CSS
↓
DOM mutations
↓
style calculation
↓
layout
↓
paint
↓
compositor
```

você pode ter:

```text
agent / application logic
          ↓
      plev scene
          ↓
 compositor incremental
          ↓
         GPU
```

uma interface generativa pode criar visualizações novas, milhares de
elementos, animações transitórias, gráficos, vídeo, canvas, spatial UI,
texto, overlays, comparações e superfícies temporárias. nesse contexto,
GPU-first é infraestrutura para materialização dinâmica. o objetivo não é
ganhar benchmark de botão; é tornar barata a criação de superfícies
interativas que talvez existam por poucos segundos.

---

## 19. incrementalidade precisa atravessar a engine inteira

hoje a cena é reconstruída por frame e as layers evitam trabalho de GPU
quando não mudaram. isso é inteligente. mas uma engine grande, e ainda mais
uma engine generativa, precisa evitar trabalho também em:

```text
layout
text shaping
hit testing
scene construction
style resolution
GPU geometry
```

o plev precisa chegar ao ponto em que "mudei 1 caractere" não significa
reconstruir uma quantidade grande de estado de CPU desnecessariamente. o
sistema precisa conhecer dependências:

```text
mudança
↓
quais valores dependem dela?
↓
quais controles foram semanticamente afetados?
↓
quais layouts precisam ser refeitos?
↓
quais layers mudaram?
↓
quais buffers precisam mudar?
```

mantra para todos os agentes: criar um sistema de dependências incremental
independente da representação visual, capaz de propagar invalidação mínima
entre estado, semântica, layout, scene construction, hit testing e
rendering. essa incrementalidade é uma propriedade do runtime inteiro.

---

## 20. texto não é detalhe

mostrar texto é relativamente fácil. para o plev virar plataforma de
aplicações, é preciso uma experiência robusta para:

- seleção;
- cursor;
- bidi;
- árabe;
- CJK;
- IME;
- emoji;
- ligatures;
- clipboard;
- undo/redo;
- accessibility;
- syntax highlighting;
- documentos grandes;
- milhões de linhas quando o domínio exigir.

isso é outra história, e é uma montanha. GPUI/Zed passou anos nisso. é um
dos pontos onde engines alternativas parecem impressionantes em demo e
quebram em software cotidiano. para o plev ser uma plataforma séria, text
editing precisa ficar absurdamente sólido.

---

## 21. accessibility é parte do contrato

esse ponto sozinho mata muitos frameworks alternativos. VoiceOver, NVDA,
TalkBack, keyboard navigation, focus semantics e action semantics precisam
ser cidadãos do runtime. se não funcionar muito bem, grandes produtos
simplesmente não podem adotar. o plev já inclui componentes de
accessibility na arquitetura, mas estar presente e estar production-ready
são coisas completamente diferentes.

acessibilidade não deveria ser adicionada depois que os pixels estão
prontos. se a tese é que humano, agente e máquina compartilham o mesmo
modelo lógico, accessibility é uma manifestação natural desse modelo. não
porque é checklist de framework, mas porque uma estrutura semântica boa
para agentes também deveria ser capaz de explicar a interface a
tecnologias assistivas. esse é um teste de qualidade da própria semântica.

---

## 22. integração de plataforma continua existindo

aplicação desktop não é só pixels. mesmo com renderer próprio, aplicação
real convive com o sistema operacional:

```text
menus
system tray
notifications
file picker / file dialogs
drag and drop
clipboard
IME
multiple windows
native dialogs
printing
camera
microphone
deep links
sleep / wake
GPU device loss
accessibility
```

essas bordas consumiram muitos anos de desenvolvimento em outros projetos.
elas não diminuem a tese: são parte da diferença entre renderer e
application runtime.

---

## 23. determinismo visual

o mantra das anotações: "one pixel-identical frame on every target".
conseguir garantir:

```text
mesma UI
mesmo layout
mesmas fonts
mesmos pixels
macOS = Windows = Linux = browser = mobile
```

fonts, drivers, GPU, DPI, text rasterization e color management tornam
isso infernal. o valor maior não é fetichizar pixels idênticos; é reduzir
variação não controlada. o plev deveria conseguir dizer claramente:

```text
esta diferença existe porque o produto pediu
```

em vez de:

```text
esta diferença apareceu por divergência silenciosa entre plataformas
```

determinismo serve a previsibilidade, testes e reprodutibilidade. não vira
dogma.

---

## 24. UI asset compiler

a filosofia do monster pode ser ampliada. hoje:

```text
Lottie
↓ build time
monster
↓ runtime
GPU
```

a mesma ideia serve para:

```text
SVG
fonts
icons
animations
images
layouts
shaders
design assets
```

quando uma representação externa complexa não precisa sobreviver inteira
em runtime:

```text
source format
↓
compile
↓
representação nativa do plev
↓
runtime rápido
```

isso gera um asset pipeline de verdade para UI: deslocar trabalho
previsível para build time e manter o runtime pequeno, determinístico e
rápido.

### gravar interação real como asset

spring, fade, gesto: o monster descobre delta a partir de frames
amostrados. gravar alguém usando a UI, exportar `.monster` e tocar como
demo, onboarding ou substituto de lottie com o pixel exato do app. handoff
ao contrário: o app gera o motion, não o designer.

---

## 25. headless é primeira classe

se interface é dado, janela não é requisito para compreendê-la. o mesmo
runtime permite:

```text
scene
↓
headless renderer
↓
PNG
SVG
PDF
CI snapshot
Open Graph
reports
tests
```

o `text_probe` já desenha offscreen para png. generalizar: qualquer página
do showcase, qualquer tema, qualquer viewport, qualquer frame de `.monster`
vira png ou svg. snapshot de CI sempre atual, og-image de página, pdf de
relatório do urnaui desenhado pela mesma engine.

mais importante: um agente trabalha com a estrutura da aplicação sem
precisar de framebuffer ou screenshot como representação primária. a imagem
continua disponível quando informação visual realmente importa, mas deixa
de ser o único mecanismo de observação.

---

## 26. testes estruturais e visuais coexistem

nem toda regressão visual precisa começar comparando milhões de pixels. se
a engine conhece:

```text
scene
layer
identity
semantic state
```

um teste descobre primeiro "o que mudou?", depois "isso deveria ter
mudado?", e só então recorre a diff visual quando relevante. cada layer já
tem hash: o CI grava o hash por página, tema e viewport; só quando o hash
muda os dois lados são renderizados e o diff aparece, e a diferença chega
com o nome da layer.

uma regressão chega como:

```text
settings.security changed
└─ control: encryption.toggle
   └─ semantic state unchanged
      └─ visual representation changed
```

isso é muito mais informativo do que:

```text
snapshot mismatch: 37,821 pixels
```

---

## 27. cena por socket

`SceneNode` é dado puro. a UI pode rodar num lugar e desenhar noutro. o app
no android manda a cena para o desktop, que renderiza com o inspector por
cima; ou o contrário, lógica no servidor e cliente fino renderizando
(server-driven UI no nível de cena, não de JSON de widget). o urnaui
explorando um corpus remoto sem baixar nada é o caso óbvio.

---

## 28. o que já existe perto, e o que não afirmar

já existem sistemas que operam interfaces sem depender de screenshot. o
Playwright MCP (https://github.com/microsoft/playwright-mcp) usa snapshots
estruturados de acessibilidade. o Playwright Trace Viewer
(https://playwright.dev/docs/trace-viewer) permite examinar ações e
estados do DOM.

por isso não é seguro basear a diferenciação do plev só em "agentes operam
sem pixels", nem afirmar sem validação "primeira engine em que agentes agem
por identidade". a hipótese forte é o contrato unificado da própria
engine: o controle apresentado, sua identidade, significado, ações,
acessibilidade, automação e registro pertencem ao mesmo sistema. isso é
uma hipótese de diferenciação a demonstrar, não um fato.

uma frase possível para a direção:

> plev: interfaces que humanos e agentes operam, e que qualquer pessoa
> pode inspecionar depois.

a pergunta que ajuda a definir o núcleo do produto:

> quando alguém abrir o primeiro `.monster`, o que precisa conseguir
> descobrir para confiar naquela delegação?

---

## 29. o que o plev não precisa ser

- não precisa competir com Flutter em número de widgets;
- não precisa reproduzir todo comportamento histórico da web;
- não precisa implementar cada sistema do zero;
- não precisa substituir wgpu, cosmic-text, taffy ou qualquer peça boa só
  para reivindicar pureza;
- não precisa ser ao mesmo tempo renderer, IDE, Figma, sistema operacional
  e framework de modelo;
- não precisa transformar toda interface numa alucinação dinâmica produzida
  por LLM.

o núcleo é mais preciso:

> permitir que capacidades, estado e intenção sejam materializados como
> interação segura e semanticamente compreensível.

tudo que fortalece isso pertence naturalmente ao projeto. todo o resto
precisa justificar sua existência.

---

## 30. o contrato que importa

no nível mais básico, um componente precisa conseguir responder:

```text
quem sou?
o que significo?
qual é meu estado?
quais ações aceito?
quais condições tornam essas ações válidas?
quem está autorizado a executá-las?
o que aconteceu quando foram executadas?
```

se isso existe, várias outras propriedades derivam daí:

```text
rendering
accessibility
agents
testing
recording
debugging
auditing
replay
adaptive UI
generative UI
```

esse contrato importa mais do que qualquer widget específico.

mantra para todos os agentes: o plev deve permitir que uma capacidade
mantenha identidade e semântica enquanto sua representação visual pode
nascer, desaparecer ou se transformar em tempo real.

---

## 31. a transformação conceitual

o plev começa como GPU UI engine. a trajetória possível passa por native
application runtime, semantic interaction runtime e generative interaction
runtime. nesse estágio, um software deixa de precisar ser pensado como:

```text
HomeScreen
SearchScreen
SettingsScreen
ResultsScreen
```

e pode ser descrito em termos de:

```text
capabilities
data
actions
constraints
semantics
policies
```

com a interação apropriada materializada conforme necessário. a mudança
conceitual mais importante:

```text
antes

interface define
o que o usuário consegue fazer
```

```text
depois

capacidades definem
o que pode ser feito

plev decide
como isso precisa aparecer agora
```

---

## 32. uma nova relação entre software e intenção

software tradicional exige que o usuário aprenda a linguagem da aplicação:
onde fica o botão, qual menu abre, qual painel contém a opção, qual
sequência de telas leva ao resultado.

agentes começaram a inverter essa relação: o usuário expressa a intenção e
o sistema tenta entender. mas conversa ainda não substitui interação. o
plev ocupa exatamente o espaço entre as duas coisas:

```text
humano
↓
intenção

modelo
↓
interpretação

plev
↓
materialização

humano + agente
↓
interação

runtime
↓
verificação + execução + registro
```

a interface passa a existir a serviço da intenção, não o contrário.

---

## 33. o que o plev pode se tornar

não um framework de frontend. não apenas uma biblioteca gráfica. não um
clone de uma plataforma existente. mas:

> um runtime nativo onde interfaces podem ser construídas e transformadas
> em tempo real a partir de intenção, estado e capacidades, mantendo
> identidade, semântica, segurança, acessibilidade, verificabilidade e
> histórico durante todo o processo.


- dirty tracking correto (hash-based, o melhor padrao que vi nos repos analisados)
- text rendering funcional com atlas management completo (grow, evict, LRU)
- signal system equivalente ao leptos em 1/50 do tamanho
- spring solver analitico que a maioria dos frameworks nem tenta
- SDF rendering com inigo quilez formula (padrao da industria)
- premultiplied alpha enforcement em toda a stack (zero bugs de blending possiveis)


- humano e agente compreendem a mesma aplicação;
- os dois operam sobre a mesma semântica;
- a interface se reorganiza sem perder identidade;
- a engine registra o que aconteceu;
- o desenvolvedor navega do resultado visual até a origem;
- o sistema recusa ações baseadas num mundo que já mudou;
- a sessão pode ser inspecionada depois;
- uma experiência observada vira teste;
- uma interface pode existir por vinte segundos e nunca ter sido
  codificada como "tela".

e nada disso exige abandonar a precisão de um runtime nativo e GPU-first.
a interface deixa de ser uma imagem estática projetada antecipadamente e
se torna uma materialização temporária das possibilidades do sistema.
talvez essa seja a propriedade necessária para uma geração de software em
que humanos e agentes não apenas usam a mesma aplicação: eles constroem a
interação juntos enquanto ela acontece.
