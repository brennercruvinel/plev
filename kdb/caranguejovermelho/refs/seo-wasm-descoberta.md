---
title: "seo, wasm e descoberta por ai: a sintese tecnica do capitulo P9"
date: 2026-06-25
tags: [seo, geo, json-ld, schema, wasm, ssr, nlweb, mcp, descoberta-por-ai, refs]
fontes:
  - { url: "https://developers.google.com/search/docs/appearance/structured-data/faqpage", status: validado, nota: "fonte primaria google, aviso de deprecacao do faq rich result" }
  - { url: "https://www.searchenginejournal.com/google-drops-faq-rich-results-from-search/574429/", status: validado, nota: "matt g. southern, 10 mai 2026, confirma timeline e que faqpage segue tipo valido" }
  - { url: "https://github.com/nlweb-ai/NLWeb", status: validado, nota: "repo canonico do nlweb, python, mcp server, schema.org, postgres/pgvector" }
  - { url: "https://github.com/microsoft/NLWeb", status: validado, nota: "resolve por redirect para nlweb-ai/NLWeb" }
  - { url: "https://news.microsoft.com/source/features/company-news/introducing-nlweb-bringing-conversational-interfaces-directly-to-the-web/", status: validado, nota: "microsoft source, 19 mai 2025, r.v. guha, lista de early adopters" }
  - { url: "https://www.anthropic.com/news/model-context-protocol", status: validado, nota: "anuncio do mcp pela anthropic" }
  - { url: "https://en.wikipedia.org/wiki/Model_Context_Protocol", status: validado, nota: "mcp introduzido em novembro de 2024" }
  - { url: "https://vercel.com/blog/the-rise-of-the-ai-crawler", status: validado, nota: "estudo vercel + merj, gptbot e claudebot nao executam js" }
  - { url: "https://www.searchenginejournal.com/google-says-llms-txt-comparable-to-keywords-meta-tag/544804/", status: validado, nota: "mueller compara llms.txt a meta keywords; validado via listagem de busca" }
  - { url: "https://searchengineland.com/no-llms-txt-is-not-the-new-meta-keywords-458199", status: validado, nota: "carolyn shelby, 9 jul 2025, posicao contraria a do mueller" }
  - { url: "https://github.com/google/schema-dts", status: validado, nota: "tipos typescript do google para schema.org em json-ld" }
  - { url: "https://schema.org", status: validado, nota: "vocabulario canonico, institucional" }
  - { url: "https://www.seroundtable.com/schema-llms-copilot-bing-microsoft-39093.html", status: nao_confirmado, nota: "fabrice canel no smx munich mar 2025; nao re-fetchado nesta passada" }
status_validacao: "12 fontes validadas, 1 nao confirmada; 3 sub-claims marcados nao confirmado na prosa"
---

# seo, wasm e descoberta por ai: a sintese tecnica do capitulo P9

## tese

descoberta por ai nao falha no conteudo, falha na entrega. um motor generativo nao
cita o que nao consegue ler, e a leitura acontece no HTML que chega antes de
qualquer cliente rodar. esse e o ponto onde o caranguejo vermelho tem uma vantagem
concreta e um risco concreto ao mesmo tempo. o blog ja serve um `@graph` JSON-LD
limpo, gerado no servidor pelo Zola, que e um gerador estatico escrito em rust: o
grafo esta no HTML inicial por construcao. o showcase do plev, ao contrario, desenha
no cliente via wgpu sobre WASM, e uma surface de GPU nao tem texto para o crawler
ler. o mesmo projeto resolve o problema de um lado e o cria do outro. este documento
e a sintese tecnica desse contraste, ancorada no que o tema ja implementa e validada
contra fonte primaria onde deu.

## o grafo que o tema ja serve

o tema do blog centraliza a geracao de schema num unico partial,
`templates/partials/schema.html`, e emite um `@graph` por pagina. o comentario no
topo do arquivo declara a regra que importa: no persistente (Organization, WebSite,
Person) ancorado na raiz do dominio com `@id` estavel, no de pagina (BlogPosting ou
WebPage) ancorado na url da pagina, e referencia por `@id` em vez de duplicar o
objeto. isso e o oposto do padrao de plugin, onde cada extensao cospe o proprio bloco
e dois `Organization` divergentes brigam na mesma pagina.

o que o partial monta hoje:

- `Organization` em `base_url/#organization`, com `sameAs` derivado automatico das
  socials do footer (github, instagram, mastodon, twitter) mais qualquer perfil extra
  de `config.extra.organization_sameas`, e `knowsAbout` vindo de
  `config.extra.organization_knows_about`, que no `config.toml` esta como
  `["web development", "zola", "static site generators", "seo"]`.
- `WebSite` em `base_url/#website`, com `publisher` apontando para o `@id` da
  Organization, e um `SearchAction` opcional so quando existe `search_url` real.
- `Person` em `base_url/#person`, com `knowsAbout` e `sameAs` proprio do autor.
- por pagina, `BlogPosting` em `current_url#article` quando ha data, ou `WebPage` em
  `current_url#webpage` quando nao ha, mais um `BreadcrumbList` quando a pagina tem
  ancestrais. o `BlogPosting` referencia autor e publisher por `@id`, nunca repete o
  objeto, e carrega `speakable` apontando `.tldr`, `h1` e `h2`.

esse desenho ja segue a regra de ouro do grafo. o que falta no `config.toml`, e o
documento deixa explicito nos comentarios, e o `organization_sameas` com o wikidata
e o `author_sameas` do Person. sem o wikidata, o no existe mas nao se conecta ao
knowledge graph publico, que e onde a fusao de entidade ganha confianca.

### ancoragem de @id: no persistente na raiz, no de pagina na url

o `@id` e o endereco estavel do no dentro do grafo. nao e enfeite, e o mecanismo de
fusao: dois objetos com o mesmo `@id` o parser trata como a mesma entidade, e uma
referencia `{ "@id": "..." }` resolve para o objeto declarado em outro lugar da
pagina ou do site. a disciplina de ancoragem tem tres regras, e o tema acerta as
tres:

1. entidade persistente, que vale em todo o site, ancora na raiz com fragmento:
   `dominio.com/#organization`, `dominio.com/#website`, `dominio.com/#person`. o
   fragmento e estavel entre paginas, entao toda pagina aponta para o mesmo no.
2. entidade de pagina ancora na url da propria pagina com fragmento:
   `dominio.com/artigo#article`. muda de pagina para pagina, como deve.
3. declara o persistente antes do de pagina, para um parser que le em ordem resolver
   o no certo. no partial, Organization e WebSite e Person vem primeiro, o
   BlogPosting depois.

os tres bugs que toda auditoria de schema pega saem todos de `@id` mal gerido:
colisao, dois nos com o mesmo `@id` que o parser funde por engano; referencia orfa,
um `@id` apontando para entidade que nao existe; e duplicacao por CMS, plugins
injetando blocos rivais. centralizar a geracao num partial unico, como o tema faz,
mata os tres de uma vez. e o argumento mais forte para nao delegar schema a plugin.

## schema.org: o que declarar, e o que schema nao faz

schema.org e o vocabulario com que voce diz, explicito, o que a pagina e e como as
coisas se ligam. a prioridade pratica: `Organization` com `sameAs` para o wikidata
primeiro, porque e o que conecta a marca ao knowledge graph; depois `Article` ou
`BlogPosting` para o conteudo, `BreadcrumbList` para a arquitetura, `Person` do
autor com `sameAs`. dois campos subusados que pesam: `knowsAbout` em Organization e
Person, com os topicos reais de autoridade, e `sameAs` no Person, que ancora a
entidade do autor individual, nao so a marca.

o caveat que separa engenharia de superstição: a evidencia de que schema, sozinho,
aumenta citacao por ai e mista. construa o grafo porque ele e contrato de dado limpo,
remove friccao para quem ja parseia HTML, e e o input que a camada agentica vai
consumir (ver nlweb adiante), nao porque alguem prometeu citacao garantida. a regra
que amarra: concordancia tripla. texto visivel, HTML semantico e JSON-LD dizem a
mesma coisa. schema que descreve o que nao esta na pagina e ignorado na melhor
hipotese, sinal de spam na pior.

para gerar o grafo com tipo, e nao com string solta, existe ferramenta. o google
mantem `github.com/google/schema-dts`, tipos TypeScript para schema.org em JSON-LD
(validado, Apache-2.0, cerca de 1,2k stars na consulta de jun/2026). em rust o
equivalente e mais direto ainda, e e o ponto da secao de SSR: struct com `derive` de
`Serialize` e `serde_json` gerando o `@graph`, o tipo garantindo que campo
obrigatorio nao some em silencio. a unica confirmacao first-party de que schema ajuda
um motor grande veio do bing, com fabrice canel no SMX munich de marco de 2025, mas
nao re-validei essa fonte nesta passada e ela esta marcada nao confirmada.

## faqpage: o que o google aposentou em maio de 2026

aviso pratico para nao perseguir ornamento morto. a documentacao primaria do google,
em `developers.google.com/search/docs/appearance/structured-data/faqpage`, carrega o
aviso de deprecacao (validado). o timeline confirmado:

- 7 de maio de 2026: o FAQ rich result, o sanfonado de pergunta e resposta embaixo do
  link, parou de aparecer na busca do google.
- junho de 2026: saem o filtro de aparicao no search console, o relatorio de FAQ rich
  result e o suporte no rich results test; a propria documentacao do recurso foi
  removida.
- agosto de 2026: sai o dado de FAQ na api do search console. quem roda dashboard ou
  export de bigquery contra essa api precisa ajustar a chamada antes do prazo, ou come
  retorno nulo em silencio.

o que continua: `FAQPage` segue tipo schema.org valido, e a marcacao pode ficar na
pagina sem causar problema. o search engine journal confirma isso de forma explicita
(matt g. southern, 10 de maio de 2026, validado): a marcacao nao penaliza, so deixou
de produzir resultado visivel no google. a regra pratica: mantenha `FAQPage` onde a
pergunta e a resposta sao reais e visiveis, porque conteudo liderado por pergunta e
forte para extracao por motor generativo, e a marcacao segue rastreavel por bingbot e
pelos crawlers de RAG. remova so onde a secao existia para o enfeite de SERP.

nota de validacao: a afirmacao mais forte que circula, de que o google "continua
usando o FAQPage para entender a pagina", nao aparece na pagina primaria do google
nem no artigo do SEJ. as duas fontes confirmam que o tipo segue valido e que a
marcacao nao e penalizada, e so. tratei "o google ainda usa para entender" como nao
confirmado.

## o buraco do wasm client-side: o crawler le html vazio

aqui mora o problema central do capitulo. a maioria dos crawlers de ai nao executa
javascript. se o conteudo so existe depois do JS rodar no cliente, o bot chega, le
HTML vazio e vai embora. nao tem meio termo, e o erro e binario: ou o conteudo esta
no HTML inicial, ou ele e invisivel para o motor.

o dado nao e folclore, foi medido. o estudo da vercel com a merj rastreou as
requisicoes de bot na rede da vercel e em `nextjs.org` (validado). no mes analisado,
o GPTBot da openai gerou 569 milhoes de requisicoes e o ClaudeBot da anthropic 370
milhoes. o achado central: nenhum dos dois executa JS. o GPTBot baixou arquivos
javascript em 11,50% das requisicoes e o ClaudeBot em 23,84%, mas baixar nao e rodar,
e nenhum rodou. o mesmo vale para o crawler da meta, o bytespider da bytedance e o
PerplexityBot. a excecao e o AppleBot, que renderiza JS com um crawler de navegador,
como o googlebot. a recomendacao do estudo e direta: renderize no servidor o conteudo
critico, deixe o cliente so para enriquecimento (contador de view, widget interativo).

### o caso extremo: uma surface de gpu nao tem texto

o plev leva esse problema ao limite. o showcase desenha a ui com wgpu sobre WASM. o
crawler que falha em ler um React SPA pelo menos encontra DOM depois que o JS roda
(se ele rodasse). um canvas de wgpu nao tem nem isso: e pixel numa surface de GPU,
sem texto, sem DOM semantico, sem heading. nenhum bot le a tela de um app que pinta a
si mesmo no cliente. a consequencia e que descoberta por ai, para qualquer app do
plev, nao pode depender do que esta na tela. precisa de uma representacao em texto
servida pelo backend, ou de um endpoint que o agente consulte direto (ver nlweb). o
mesmo raciocinio vale para app nativo swift ou kotlin sem versao web: nao existe
crawler lendo a tela de um app mobile.

## a solucao: render no servidor, e o caso rust

a regra e uma: conteudo e dados estruturados tem que estar no HTML que chega antes de
qualquer execucao de cliente. o nome muda com o stack, o principio nao:

- em meta-framework (next.js, nuxt, sveltekit, astro): renderize no servidor, SSR,
  SSG ou ISR, nunca monte o conteudo num `useEffect`.
- em SPA pura (vite, CRA): o pior caso. ponha pre-render na frente (prerender.io,
  rendertron) ou migre para um framework que renderiza no servidor.
- em backend de qualquer linguagem (rails, django, laravel, go, php, rust): monte o
  HTML completo no servidor e sirva. a linguagem nao muda nada, o que muda e o
  conteudo existir no HTML inicial.

o caranguejo vermelho ja vive do lado certo dessa linha para o blog. o Zola e um
gerador estatico em rust: ele resolve os templates Tera em build, e o `@graph` sai no
HTML estatico, antes de qualquer JS. e SSG na pratica, e e por isso que o blog nao
sofre do buraco do wasm mesmo sendo um projeto rust com peso de cliente.

o caso rust generalizado, para um backend dinamico (axum, actix), e o argumento de
fechamento: voce define o grafo como struct, deriva `Serialize`, e o `serde_json`
serializa o `@graph` no HTML servido. o tipo do rust vira a garantia. campo
obrigatorio que falta nao compila, `@id` e referencia ficam sob o checador, e o grafo
chega ao crawler ja montado. e a diferenca entre "o schema esta certo porque alguem
conferiu na mao" e "o schema esta certo porque o tipo nao deixa errar". para um app
plev que precisa de descoberta, o vetor nao e a surface de GPU, e um endpoint rust
que serializa a mesma data em JSON-LD e, no proximo degrau, em resposta agentica.

## nlweb e mcp: o no vira endpoint

com o grafo estavel, o degrau seguinte e deixar de ser so pagina e virar endpoint que
um agente consulta. o nlweb, da microsoft, faz isso: transforma um site em servidor
MCP. o anuncio saiu no microsoft source em 19 de maio de 2025 (validado), assinado por
r.v. guha, o mesmo de RSS, RDF e schema.org. cada instancia de nlweb e tambem um
servidor MCP, o que torna o conteudo do site consultavel por agente sem raspar HTML.
a implementacao de referencia e em python, aceita schema.org como formato de entrada e
saida, e suporta postgres com pgvector entre os vector stores.

o MCP que o nlweb fala e o Model Context Protocol, padrao aberto introduzido pela
anthropic em novembro de 2024 (validado, confirmado pela anthropic e pela wikipedia).
e o protocolo que padroniza como um modelo conecta a ferramenta e fonte de dado
externa. nlweb sobre MCP fecha a topologia: o JSON-LD limpo deixou de ser so comida de
crawler, virou o input que o nlweb consome e o formato que ele devolve. um `@graph`
bem ancorado vira api conversacional sem reescrever backend, e os melhores resultados
vem de sites ja estruturados como lista de item, evento ou produto. para o plev, e a
unica saida real de descoberta para um app sem versao web: expor `/ask` e `/mcp` sobre
a data que o app consome, no backend, e nao na ui de GPU.

divergencia que precisa ficar registrada, porque o briefing pediu validar
`github.com/microsoft/NLWeb` e o post P9 cita `github.com/nlweb-ai/NLWeb`: os dois
resolvem. o repo canonico hoje e `nlweb-ai/NLWeb`, e a url `microsoft/NLWeb`
redireciona para ele. o projeto migrou de owner. quem cita deve usar `nlweb-ai/NLWeb`,
com a nota de que a url antiga da microsoft ainda resolve por redirect. segundo ponto:
o post P9 diz que o nlweb foi anunciado "no build 2025". a data bate (o build 2025
correu de 19 a 22 de maio de 2025 e o anuncio e de 19 de maio), mas o artigo primario
do microsoft source nao nomeia o build, diz so "today". tratei a atribuicao ao build
como nao confirmada pela fonte primaria, ainda que a data seja consistente. os early
adopters que o post lista (shopify, snowflake, o'reilly, tripadvisor, eventbrite,
hearst) estao todos confirmados na fonte primaria, que ainda inclui chicago public
media, common sense media, ddm, inception labs, milvus e qdrant.

## llms.txt, robots e os user agents de ai

o `llms.txt` e um indice em markdown de urls importantes. real, mas secundario: nao e
fator de ranking nem de bloqueio. gary illyes disse no search central live de julho de
2025 que o google nao usa e nao planeja usar, e john mueller comparou o arquivo a
velha meta keywords tag, justo a que os buscadores ignoram ha mais de uma decada
porque o dono do site controla e portanto manipula
(searchenginejournal.com/google-says-llms-txt-comparable-to-keywords-meta-tag,
validado). divergencia explicita, e ela importa: carolyn shelby, no search engine
land de 9 de julho de 2025 (validado), discorda da analogia. o argumento dela e que
llms.txt cura urls reais cujo conteudo precisa existir, ao contrario da meta keywords
que deixava declarar qualquer coisa sem prova. nao consolido as duas: mueller acha que
e cloaking facil, shelby acha que e mapa controlado e legitimo. as duas posicoes ficam
na mesa, e a decisao pratica e tratar llms.txt como camada opcional, depois de HTML
semantico, dados estruturados e sitemap, nunca como prioridade.

robots e a porta do no. para ser citado, o agente precisa alcancar o conteudo. mantenha
destravados no robots.txt os user agents que importam. a frota cresceu desde a primeira
lista do post: cada fornecedor roda agora mais de um bot, separados por funcao
(treino, busca, fetch on-demand). a lista validada na consulta de jun/2026:

| fornecedor | user agents | funcao |
|------------|-------------|--------|
| openai | `GPTBot`, `OAI-SearchBot`, `ChatGPT-User` | treino, indice de busca, fetch sob demanda |
| anthropic | `ClaudeBot`, `Claude-SearchBot`, `Claude-User` | treino, indice de busca, fetch sob demanda |
| perplexity | `PerplexityBot`, `Perplexity-User` | indice, fetch sob demanda |
| google | `Google-Extended` | controle de uso no gemini |
| common crawl | `CCBot` | corpus aberto que alimenta varios modelos |

bloquear e decisao estrategica, faz sentido para conteudo pago, mas tira o site de um
canal de aquisicao em crescimento. e o detalhe que derruba site sem ninguem ver: erro
de servidor numa requisicao a arquivo incomum, ou bloqueio acidental por CDN ou WAF,
pode estar barrando esses bots sem aparecer em relatorio nenhum. cheque o log de
servidor, e o sinal mais direto do que o motor de fato ingere.

## fontes (status de validacao)

| item | fonte | status |
|------|-------|--------|
| faqpage deprecado, timeline mai/jun/ago 2026 | developers.google.com (primaria) + SEJ 574429 | validado |
| faqpage segue tipo schema.org valido, marcacao nao penaliza | SEJ 574429 (matt g. southern) | validado |
| "google ainda usa faqpage para entender a pagina" | nenhuma fonte primaria confirma | nao confirmado |
| crawlers de ai nao executam js | vercel + merj | validado |
| gptbot 569M req, claudebot 370M req no mes | vercel + merj | validado |
| gptbot baixa js 11,50%, claudebot 23,84%, nao executam | vercel + merj | validado |
| applebot renderiza js como googlebot | vercel + merj | validado |
| mcp introduzido pela anthropic em nov/2024 | anthropic + wikipedia | validado |
| nlweb anunciado 19 mai 2025 por r.v. guha | microsoft source (primaria) | validado |
| nlweb e servidor mcp, schema.org, /ask, postgres/pgvector | nlweb-ai/NLWeb (repo) + microsoft source | validado |
| repo canonico do nlweb e nlweb-ai/NLWeb | github (fetch dos dois owners) | validado |
| "nlweb anunciado no build 2025" | microsoft source diz "today", nao nomeia build | nao confirmado (data consistente) |
| early adopters shopify, snowflake, o'reilly, tripadvisor, eventbrite, hearst | microsoft source | validado |
| google nao usa llms.txt (illyes/mueller, jul 2025) | SEJ 544804 | validado |
| contraponto: llms.txt nao e meta keywords (shelby, jul 2025) | search engine land 458199 | validado |
| user agents de ai (frota por fornecedor) | nohacks/momentic/almcorp via busca jun/2026 | validado |
| schema-dts: tipos typescript do google para json-ld | github.com/google/schema-dts | validado |
| schema ajuda llms da microsoft (canel, smx munich mar 2025) | seroundtable 39093 | nao confirmado (nao re-fetchado) |

as estatisticas de mercado do post P9 maior (seer 61%, ahrefs 0,218 vs 0,664, muck
rack 94%/82%, moz 88%, geo-16 78%, averi 38 a 65%) estao fora do escopo deste
documento tecnico e nao foram re-validadas aqui. quem citar essas no capitulo
revalida na bibliografia do proprio post antes de publicar.

## divergencias entre fontes (nao consolidadas)

1. repo do nlweb: briefing pediu `microsoft/NLWeb`, post P9 usa `nlweb-ai/NLWeb`. os
   dois resolvem, o canonico e `nlweb-ai/NLWeb`, a url da microsoft redireciona. usar o
   canonico.
2. llms.txt como meta keywords: mueller e illyes (google) afirmam a analogia e o nao
   uso; shelby (search engine land) rejeita a analogia. as duas posicoes ficam
   explicitas, sem media.
3. nlweb e o build 2025: post diz "no build 2025", fonte primaria diz so "today" em 19
   de maio de 2025. data consistente, atribuicao ao evento nao confirmada.
4. faqpage "ainda usado para entender a pagina": circula em fonte secundaria, nao
   aparece em fonte primaria. confirmado so que o tipo segue valido e nao penaliza.
