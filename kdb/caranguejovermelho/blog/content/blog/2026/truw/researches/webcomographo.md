+++
title = "webcomographo"
date = 2026-01-01
draft = true
+++

Camada 1: O Web como grafo

  A web nao e uma lista de paginas. E um grafo direcionado, nos
  (paginas) e arestas (links). Isso muda tudo.

  O Google nao "procura" noticias. Ele caminha pelo grafo. O Googlebot e
  um programa que faz BFS (Breadth-First Search), o mesmo algoritmo que
   voce aprendeu em estrutura de dados. So que distribuido em milhares de
   maquinas.

  A sacada do Larry Page (literalmente, o sobrenome dele e Page, e o
  algoritmo e PageRank) foi: a importancia de um no e funcao da
  importancia dos nos que apontam para ele. Isso e um problema de
  autovalor:

  r = M * r

  Onde M e a matriz de adjacencia normalizada do grafo web, e r e o vetor
   de rankings. Voce resolve por iteracao (power method). Nao tem
  if/else. E algebra linear.

  Camada 2: Crawling nao e scraping

  Scraping e: "vou naquela pagina, pego o HTML, extraio o texto". Fragil,
   ponto-a-ponto, quebra toda semana.

  Crawling em escala planetaria e outra coisa. Os principios:

  Frontier queue, uma fila de prioridade de URLs a visitar, ordenada
  por: freshness esperada, importancia do dominio, tempo desde ultima
  visita. Matematicamente, e um problema de scheduling com restricoes:
  - Politeness: nao bater no mesmo dominio mais que X vezes/segundo
  - Freshness: estimar a taxa de mudanca de cada pagina (modelo
  probabilistico)
  - Budget: N crawlers, M bilhoes de paginas, como alocar?

  Consistencia de deduplicacao, como saber se voce ja viu essa pagina?
  Com bilhoes de URLs, voce nao pode fazer if url in set. Usam Bloom
  filters, uma estrutura probabilistica que responde "definitivamente
  nao" ou "provavelmente sim" usando k funcoes hash. Usa memoria
  constante. Falsos positivos aceitaveis, falsos negativos zero.

  Near-duplicate detection, 50 outlets publicam a mesma materia da
  Reuters com titulos diferentes. Como detectar? SimHash/MinHash, voce
  projeta o documento em um fingerprint de 64 bits tal que documentos
  similares produzem hashes similares. A probabilidade de colisao e
  proporcional a similaridade de Jaccard. Nao e if/else. E hashing
  sensivel a localidade (LSH).

  Camada 3: O que o Google News faz que parece magia

  1. Clustering, agrupa artigos sobre o mesmo evento. Nao por keywords
  (isso e anos 2000). Por embeddings em espaco vetorial. Artigos sobre o
  mesmo fato caem no mesmo cluster mesmo com palavras completamente
  diferentes. K-means ou HDBSCAN no espaco de embeddings.
  2. Entity linking, "Lula" no texto vira a entidade Q37181 no
  Knowledge Graph (Wikidata). "O presidente" tambem vira Q37181 pelo
  contexto. Isso e NER (Named Entity Recognition) + disambiguation. Nao e
   regex. Sao modelos de linguagem.
  3. Freshness scoring, cada pagina tem uma taxa de mudanca estimada.
  Um portal de noticias muda a cada minuto. Uma pagina de Wikipedia, a
  cada semana. O crawler aloca budget proporcionalmente. Modelo: processo
   de Poisson.
  4. Quality signals, E-E-A-T (Experience, Expertise,
  Authoritativeness, Trustworthiness). Nao e um checklist. E um ensemble
  de sinais: historico do dominio, backlinks de autoridades, consistencia
   com Knowledge Graph, user engagement signals. Cada sinal e um feature.
   O ranker e um modelo treinado (LambdaMART, agora neural).

  Camada 4: O que a Perplexity faz

  Perplexity e mais simples do que parece. O pipeline:

  Query do usuario
        |
        v
  Search API (Bing/Google), eles NAO crawleam tudo
        |
        v
  Top N resultados (10-20 URLs)
        |
        v
  Fetch + parse do conteudo (isso sim e scraping pontual)
        |
        v
  Chunking + reranking (qual trecho e mais relevante?)
        |
        v
  LLM sintetiza resposta com citacoes

  O "segredo" da Perplexity nao e o crawling. E o reranking, dado o
  contexto da query, qual trecho de qual fonte e mais informativo? Usam
  modelos como ColBERT ou cross-encoders. E busca vetorial com
  refinamento.

  Camada 5: O que o GDELT faz (e por que importa pro Truw)

  GDELT e o mais proximo do que voce quer. Kalev Leetaru construiu isso
  no Google Jigsaw. O pipeline:

  1. Ingestao, monitora ~300k outlets via RSS/Atom feeds + sitemap.xml.
   Nao crawlea o HTML inteiro. Le o feed estruturado. Isso e ordens de
  magnitude mais eficiente.
  2. CAMEO coding, cada evento e codificado numa taxonomia de 300+
  tipos (CAMEO = Conflict and Mediation Event Observations). "Presidente
  assina lei" = tipo 0331. Isso comprime linguagem natural em codigos de
  4 digitos. Exatamente o principio dos seus claim tuples.
  3. GKG (Global Knowledge Graph), entidades, temas, tons emocionais,
  localizacoes, todos extraidos automaticamente e interligados. Atualiza
  a cada 15 minutos. Eles processam ~3/4 bilhao de artigos no historico.
  4. Armazenamento, BigQuery (colunar, comprimido). Voce faz queries
  SQL sobre toda a historia da midia global.

  A pergunta que importa pro Truw

  A engenharia por tras nao e if/else. E:

  - Algebra linear (rankings, embeddings, similaridade)
  - Teoria de grafos (crawling, propagacao, Knowledge Graph)
  - Probabilidade (deduplicacao, freshness, scheduling)
  - Teoria da informacao (compressao, delta encoding, exatamente o que
  voce esta pesquisando)
  - Sistemas distribuidos (filas, backpressure, consistencia eventual)

  Mas nenhum deles comecou com tudo. O Google comecou crawleando
  Stanford. O GDELT comecou com feeds RSS. A Perplexity comecou usando a
  API do Bing.

  A questao real para o Truw nao e "como crawlear tudo", e qual e a
  representacao minima de um fato verificavel que permite compressao
  temporal maxima. E isso voce ja esta respondendo com os claim tuples de
   24 bytes e o delta encoding.

  O GDELT comprime em CAMEO codes. Voce comprime em claim tuples. A
  diferenca e que o GDELT registra eventos e o Truw quer registrar
  veracidade. Sao grafos diferentes sobre os mesmos dados

 Agregadores e bases abertas (gratis, sem limite)

  GDELT Project, BigQuery free tier, 300k+ outlets, atualiza a cada
  15min, historico desde 1979
  Media Cloud, open source, 60k+ fontes, API gratuita, export CSV
  Common Crawl, arquivo completo da web, petabytes, hospedado na AWS
  Open Data
  Internet Archive / Wayback Machine, arquivo historico, API gratuita
  Wikipedia Current Events Portal, curadoria humana diaria de eventos
  globais
  Wikimedia Event Stream, stream em tempo real de edits (proxy para
  breaking news)
  EventRegistry, 150k+ fontes, free tier 500 queries/mes
  Open PageRank, ranking de dominios, gratuito, CSV download

  APIs com free tier

  NewsData.io, 79k fontes, 206 paises, 200 requests/dia gratis
  GNews.io, 60k+ fontes, 100 requests/dia gratis
  TheNewsAPI, 80k+ fontes, 150 requests/dia gratis
  Currents API, 30k+ fontes, 600 requests/dia gratis
  Spaceflight News API, niche, mas 100% gratuito e sem limite
  MediaStack, 7.500 fontes, 100 requests/mes gratis
  NewsAPI.org, 80k+ fontes, 100 requests/dia gratis (dev only,
  production paga)
  WorldNewsAPI, 50k+ fontes, 100 requests/dia gratis

  RSS/Atom (100% gratis, sem limites, sem API key)

  OPML directories, listas curadas de feeds RSS de milhares de outlets
  Google News RSS, qualquer topico/pais via URL parametrizada, sem key
  Reddit RSS, qualquer subreddit como feed (/r/worldnews/.rss)
  Hacker News API, gratuita, sem autenticacao
  Feedspot, diretorio de 200k+ feeds RSS categorizados
  AllTop, agregador de RSS por topico
  Planet, agregadores de feeds por comunidade

  Datasets academicos (download unico, gratis)

  NELA-GT (Harvard), 1.8M artigos de 500+ fontes, com labels de
  credibilidade
  FakeNewsNet (ASU), PolitiFact + GossipCop, claims com labels
  LIAR dataset, 12.8k claims politicas com 6 labels de veracidade
  FEVER (Fact Extraction and Verification), 185k claims contra
  Wikipedia
  MultiFC, 34k claims de 26 fact-checking sites
  CT-FAN, dataset multilingual de claims COVID
  PHEME, rumores no Twitter com annotations
  RumourEval, propagacao de rumores em redes sociais
  CredBank, 60M tweets, 1k+ eventos, scores de credibilidade
  ISOT Fake News, 44k artigos (21k fake, 23k real)
  Kaggle Fake News, varios datasets abertos
  BuzzFace, 2.2k artigos do BuzzFeed com labels

  Fact-checkers abertos (RSS + API + structured data)

  ClaimReview schema (Google), padrao aberto, todos fact-checkers que
  usam ficam indexaveis
  Google Fact Check API, gratuita, indexa todos ClaimReview do planeta
  DataCommons, Google, dados estruturados de verificacoes
  Full Fact API, gratuita, fact-checks UK
  Snopes RSS, gratuito
  PolitiFact RSS, gratuito
  AFP Checamos RSS, gratuito, PT-BR
  Lupa RSS, gratuito, PT-BR
  Aos Fatos RSS, gratuito, PT-BR
  Agencia Publica RSS, gratuito, PT-BR
  Fato ou Fake (G1) RSS, gratuito, PT-BR
  Chequeado (Argentina) RSS, gratuito, ES
  Maldita (Espanha) RSS, gratuito, ES
  Africa Check RSS, gratuito, EN
  Vishvas News (India) RSS, gratuito, HI/EN
  BOOM Live (India) RSS, gratuito
  Taiwan FactCheck Center RSS, gratuito

  Governo e diarios oficiais (gratis, dados primarios)

  DOU (Diario Oficial da Uniao), API aberta, todos atos oficiais do
  Brasil
  Dados Abertos Brasil (dados.gov.br), portal federal
  Camara dos Deputados API, votacoes, projetos, discursos
  Senado Federal API, votacoes, senadores, materias
  TSE Dados Abertos, eleicoes, candidatos, prestacao de contas
  US Federal Register API, gratuita
  EU Open Data Portal, regulacoes, datasets
  UK Parliament API, votacoes, debates, Hansard
  Congress.gov API, gratuita

  Ferramentas de crawling (gratis, self-hosted)

  Scrapy, Python, open source, o padrao da industria
  Colly, Go, rapido, leve
  Newspaper3k / Newspaper4k, Python, extrai artigo limpo de qualquer
  URL
  Trafilatura, Python, extracao de texto de alta qualidade
  readability (Mozilla), JS, o mesmo que o Firefox Reader Mode usa
  RSS-Bridge, self-hosted, gera RSS de sites que nao tem
  Miniflux, leitor RSS self-hosted, API, PostgreSQL
  FreshRSS, leitor RSS self-hosted, gratuito

  Resumo pratico: com GDELT (BigQuery free tier) + Google Fact Check API
  - RSS dos principais outlets + datasets academicos, voce cobre o
  planeta inteiro sem pagar nada. O gargalo nao e acesso, e
  processamento. E exatamente ai que a compressao delta entra.

  Camada 1 ◉ Interface civica
  Um feed de noticias com gamificacao (leaderboard,
  achievements, calibrador) que ensina o usuario a pensar
   criticamente. O espectro politico posiciona o usuario
  antes de mostrar conteudo, criando consciencia do
  proprio vies.

  Camada 2 ◉ Motor de verificacao
  Scoring composto que cruzafontes com ratings
  externos (NewsGuard, Ad Fontes, MBFC, IFCN), pesquisa
  automatica, scraping de portais, e analise espectral.
  Nao e "verdadeiro ou falso" binario. E um gradiente de
  credibilidade com 8 status possiveis.

  Camada 3 ◉ Knowledge Graph
  Grafo de desinformacao com entidades tipadas, claims,
  narrativas, spreaders, e arestas polimorficas. Rastreia
   como informacao se propaga, nao apenas o que diz.

  Camada 4 ◉ NTA (Narrative Trajectory Analysis)
  A camada que diferencia de tudo que existe. A topologia
   de propagacao como sinal diagnostico. Verdades
  propagam com deltas pequenos e fontes diversas.
  Falsidades propagam com deltas grandes e clusters
  homogeneos. Isso e verificacao por fisica da
  informacao, nao por autoridade editorial.

  Camada 5 ◉ Compressao e soberania
  Tuple de 24 bytes, delta encoding, embeddings
  vetoriais. A verificacao pertence ao usuario, nao a
  plataforma. O dado e comprimido, portavel, e auditavel.

  Tipo de aplicacao: infraestrutura epistemica. Mais
  proximo de um protocolo (como TCP/IP e para rede, Truw
  seria para confianca) do que de um app consumer
  tradicional. O app mobile e o cavalo de Troia para
  adocao. O protocolo e o produto real.
