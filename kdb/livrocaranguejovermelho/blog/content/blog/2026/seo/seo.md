+++
title = "busca em 2026: de índice de páginas a grafo de entidades"
date = 2026-01-01
draft = true
+++



# busca em 2026: de índice de páginas a grafo de entidades

> a busca parou de ordenar documentos e passou a resolver um grafo.
> a sua marca é um nó, quem fala de você é aresta, a resposta é o motor percorrendo isso e citando os nós em que confia. 
> esse é um estudo denso de seo, geo, crawler de ia e medição, no llm bullshit.

a estrutura search mudou, não a surface. 
a busca clássica era um índice: documento de um lado, query do outro, sai uma lista ordenada de dez links. essa logica continua existindo, mas deixou de ser o jogo único. o motor generativo não ordena lista, ele recupera passagem de várias fontes, sintetiza uma resposta e cita algumas, quem voce queira, ou não, todas fangs empurrando ai muitas vezes, a contra-gosto e atrapalhando a vida de quem só quer uma dentista no bairro... ranquear agora é ordenar uma lista. ser citado é existir no grafo certo. dois problemas diferentes, e quem resolve o segundo como se fosse o primeiro perde.

as pessoas não estao buscando menos, o volume cresceu, o google viu quase 50% mais impressões no último ano, mas o clickrate despencou. quase dois terços das buscas no google terminam sem clique, no ai mode 93%. a resposta acontece na página, e a influência acontece dentro dela, antes do clique e sem ele. ser uma das fontes citadas é o jogo novo.

vamos abordar a "topologia" desse grafo, como virar um nó que o motor reconhece, como criar as arestas de evidência que fazem ele te citar, como deixar o nó legível e acessível, e como medir quais nós são citados quando ninguém clica.

---

## o tamanho da virada, em números


ai overview na página derruba o ctr orgânico. a seer interactive mediu queda de 61%, de 1,76% pra 0,61%, em mais de três mil termos. a ahrefs mediu 58% menos clique no primeiro resultado em dezembro de 2025, contra 34,5% em abril. a curva acelera.

o referral que os llms mandam ainda é pequeno. o chatgpt lidera com uns 80% do referral de chatbot de ia, a perplexity uns 12%. perto do google é miudeza, relatório de publisher põe o chatgpt em torno de 0,02% do referral total, e a tollbit estima chatbot de ia mandando 95 a 96% menos tráfego que a busca. o clique de ia é baixo. a influência mora dentro da resposta, fora de qualquer dashboard.

a consequência estrutural: ranquear parou de prever ser citado. o overlap entre top 10 e o que os motores citam caiu de uns três quartos em meados de 2025 pra 17 a 38% no começo de 2026, depois do ai overviews subir pro gemini 3. a moz achou 88% das citações do ai mode fora do top 10 orgânico. dois grafos diferentes, interseção pequena.

o outro lado: marca citada no ai overview ganha mais clique, não menos. a alm corp mediu 35% mais clique orgânico e 91% mais clique pago pra quem é citado. o clique que sobra é de intenção alta, o usuário já leu o resumo. volume bruto de clique virou métrica enganosa, qualidade do referral é a régua.

---

## dois jogos, duas estruturas

jogo um, busca ranked, o de sempre. o google ainda indexa, ainda ordena dez links, ainda manda a maior fatia de tráfego do planeta. backlink continua fator de ranking, o vazamento de api de maio de 2024 confirmou apesar do discurso público. conteúdo, autoridade de domínio, higiene técnica, core web vitals, tudo vale aqui. nada morreu.

jogo dois, ser citado por motor generativo. o motor recupera passagem de várias fontes, sintetiza e dá crédito a algumas. quem entra depende de outros sinais: o quanto a passagem é fácil de extrair, o quanto terceiro confiável te cita, o quanto a informação é específica e verificável. autoridade de domínio, rainha do jogo um, correlaciona fraco com citação no jogo dois.

você joga os dois ao mesmo tempo, base comum e camada extra. a base é conteúdo bom, entidade clara, html semântico, autoridade real. a camada extra é tornar isso legível e extraível por máquina. ai seo não é trilha paralela, é reforço sobre o seo que você já deveria fazer. tratar como projeto separado custa o dobro e perde os dois.

---

## como o motor escolhe o que citar

já tem existem papers medimdo  isso, e é o que separa quem sabe de quem chuta. o paper geo de 2024, aggarwal e colegas, kdd. formaliza o motor generativo e testa o que aumenta visibilidade na resposta. dois achados se sustentaram em replicação. citar outras fontes dentro do seu conteúdo aumenta a sua chance de ser citado, porque sinaliza rigor, é o quotation addition. e estatística específica com atribuição bate afirmação vaga com folga, é o statistics addition, ganho em torno de 40% de visibilidade. repara: o paper que todo mundo invoca em discussão de schema não testou schema, testou conteúdo.

o geo-16, kumar e palkhouski, setembro de 2025, foi mais fino. 16 pilares, 1.702 citações em três motores, brave, ai overviews, perplexity. os três pilares no topo da correlação: metadata e frescor, html semântico, dados estruturados. página com nota alta e ao menos doze pilares batidos chegou a 78% de citação cross-engine, odds ratio 4,2. em geo competitivo, quando duas fontes disputam, relevância tópica e posição na lista decidem quem vem primeiro.

o que ganha citação, destilado: resposta na frente, a afirmação principal num bloco extraível nos primeiros parágrafos. estrutura clara, um h1, hierarquia de h2 e h3, sem pular nível. dado específico e verificável com fonte. fonte primária citada no corpo. e o sinal mais difícil de fingir, experiência de primeira mão, o "testei isso por sessenta dias", "rodei em quinze clientes", que separa humano do mar de texto que qualquer llm gera. marque afirmação versus opinião, mostre método, reconheça o limite. modelo penaliza confiança sem lastro, não importa a autoridade do domínio.

---

## entidade primeiro, a marca não é um site

as engines não "raciocinam" sobre página. o reasoning é sobre entidade. site é coleção de urls, marca é um nó único num knowledge graph, ligado a conceito, pessoa, lugar, outras marcas. o knowledge graph do google guarda mais de 500 bilhões de fatos sobre 5 bilhões de entidades, e o gemini treina em cima dele. se a sua empresa não é um nó reconhecível ali, você não é mal ranqueado, você é ignorado em favor de quem é.

o raciocínio da engine é empresa, entidade, evidência externa, citação. não site, artigo, ranking. o trabalho deixa de ser publicar página e passa a ser existir como nó que o grafo resolve sem ambiguidade. o sinal mais barato e mais subusado pra isso é o `sameAs` no `Organization` schema, que o google diz usar: cada `sameAs` declara que aquele perfil de terceiro é a mesma entidade do seu schema, e o motor funde os nós com confiança. você empresta a autoridade dos perfis que aponta.

quais perfis, e o peso de cada um. wikidata é a fonte canônica de base (apesar de nenhuma fang adminitir isso publicamenre), a truth node, sem exigência de notabilidade, qualquer organização tem um qid. wikipedia é o sinal único mais forte, mas exige notabilidade, só com cobertura real de terceiro. 


crunchbase e cb insights carregam fundação e funding, usados pesado por ia pra confirmar que a empresa existe. linkedin é a entidade profissional de maior peso pra b2b e saas. g2 e product hunt ancoram produto e software com prova social. github ancora projeto e quem desenvolve, peso pra devtools e open source. semantic scholar e orcid ancoram autor e pesquisa, o caminho de autoridade pra quem produz conhecimento técnico ou acadêmico.

dois detalhes fecham o node. consistência: nome, descrição, logo e atributos idênticos no site, no google business profile, no linkedin, no crunchbase, na imprensa, porque inconsistência cria ambiguidade e atrasa a resolução. e entidade de autor: `sameAs` no `Person`, credencial, byline, orcid, porque o modelo dá confiança a nó de autor individual, não só à marca. é o ponto onde entidade e earned media se cruzam: menção de terceiro confiável é a aresta que confirma o nó, e é por isso que menção de marca correlaciona com visibilidade em ia bem mais que backlink.

---

## earned media, a aresta que virou rei

a peça que mais subiu na hierarquia. 

citação por ia é dominada por earned media, conteúdo de terceiro falando de você, não conteúdo no seu domínio. 


a muck rack analisou mais de um milhão de links citados por chatgpt, claude, gemini e perplexity entre julho e dezembro de 2025: 94% das citações vieram de fonte não paga, earned media sozinho 82%, jornalismo 20 a 30%. a universidade de toronto, treze indústrias, chegou a 92% de earned media.

o contraste com o seo clássico está medido. a ahrefs achou backlink correlacionando só 0,218 com visibilidade em ia, contra 0,664 de menção de marca na web. a autoridade de domínio com citação caiu pra perto de 0,18. marca é cerca de 6,5 vezes mais citável via fonte de terceiro que via domínio próprio, e um estudo controlado mediu lift mediano de 239% ao distribuir conteúdo por veículo de terceiro.

no grafo é simples:  as engines confiam no nó que muitos nós confiáveis apontam.
digital pr, ser mencionado e citado por publicação confiável de terceiro, é a alavanca mais alta pra citação por ia hoje. não anula o backlink, que ainda conta no ranking, mas reposiciona: no jogo de ser citado, aresta de terceiro pesa mais que qualquer coisa que você publique em casa. caveat de sempre: é correlação de estudo de mercado, não lei física, varia por vertical. mas a direção é a mesma em todo estudo sério.

---

## conteúdo, dois leitores ao mesmo tempo

conteúdo é o que está dentro do nó, e nunca deixou de ser o trabalho pesado. o que mudou é que ele tem dois leitores agora, a pessoa e o modelo, e os dois recompensam quase as mesmas coisas com ênfase diferente.

lidere com a resposta. tl;dr ou bloco de takeaways no topo, autocontido, nos primeiros 150 palavras. o modelo extrai esse bloco direto, a pessoa decide em segundos se continua. escreva por tópico, não por palavra-chave: palavra-chave responde uma query, tópico cobre o tema inteiro e ganha citação por ele. parágrafo compacto, heading descritivo, lista onde a estrutura ajuda.

e-e-a-t deixou de ser jargão de auditoria e virou diferencial. experiência demonstrável, terminologia certa, referência a pesquisa primária, byline e credencial nomeadas, schema de autor. confiança é o mais crítico, e se constrói com precisão, método exposto, limite reconhecido, link pra fonte primária. é a mesma honestidade epistêmica que os papers de geo mediram funcionando: citar fonte e mostrar número específico ganha mais citação que afirmar com confiança e sem lastro.

a heurística repetida nos guias sérios é 80% em conteúdo, autoridade e e-e-a-t, 20% em técnico. heurística, não proporção universal: ela quebra quando o piso está podre. num site que é react spa sem ssr, sem sitemap, sem canonical, sem schema e bloqueando o gptbot, o gargalo é 100% técnico, e nenhum conteúdo compensa crawler que lê html vazio. aí o técnico é o trabalho inteiro até o piso existir. o 80/20 só vale depois que o básico está de pé.

---

## o fosso do dado proprietário

a maior vantagem competitiva dos próximos anos, e a única alavanca de domínio próprio que sobrevive num grafo dominado por earned media. os motores estão saturados de genérico: guia genérico, lista genérica, explicação genérica. quando o modelo sintetiza, citar mais um resumo do que já existe não acrescenta nada. ele cita o que adiciona algo único: benchmark próprio, pesquisa própria, dataset próprio, experimento próprio.

os números batem entre estudos. pesquisa original e benchmark com dado próprio são citados a 3 a 10 vezes a taxa de um blog post padrão. medição de mercado põe a citação de pesquisa original e dado proprietário em 38 a 65%, contra 6 a 15% de blog comum e 3 a 8% de página de produto. dado original em conteúdo que já existe levanta a citação em 55 a 120%, estatística com fonte sozinha em 40 a 70%. site rico em dado é citado várias vezes mais por url que diretório.

o argumento mais forte não é o número, é a estrutura: dado original não existe em fonte de terceiro. quando o assunto aparece, o motor tem duas opções, citar o seu domínio ou descartar a evidência. é por isso que dado proprietário fura a regra anterior, onde o domínio próprio perde pra earned media. aqui você vence porque ninguém mais tem a fonte. o motor trata dado primário como evidência, não como marketing. e o fosso é durável: você é citado por anos, e o competidor não replica a fonte.

e o efeito é composto: dado proprietário realimenta o grafo. pesquisa original gera cobertura, que gera menção, que gera busca de marca, que reforça o nó da entidade, que torna a marca mais segura de citar, que gera mais citação. quem trata pesquisa original como ativo único perde o ciclo, quem publica de forma consistente vira fonte de referência e acumula vantagem. na prática: pesquise os seus clientes, analise os seus dados internos, responda com dado real a pergunta que ninguém na sua indústria respondeu com rigor.

---

## o html que o crawler lê

pra ser citado, o nó precisa ser legível. e o erro mais comum aqui é binário: ou você acerta ou você é invisível. a maioria dos crawlers de ia não executa javascript. se o conteúdo só aparece depois do js rodar no cliente, o crawler chega, lê html vazio e vai embora. não tem meio-termo.

a regra é uma: conteúdo e dados estruturados têm que estar no html que chega antes de qualquer execução de cliente. server-side, sempre. em meta-framework, next.js, nuxt, sveltekit, astro, é renderizar no servidor, ssr, ssg ou isr, nunca montar conteúdo num useEffect. em spa pura, vite ou cra, o pior caso, você põe pré-renderização na frente, prerender.io ou rendertron, ou migra pra um framework com renderização no servidor. em backend de qualquer linguagem, rails, django, laravel, go, php, idêntico, monte o html completo no servidor e sirva. a linguagem não muda nada, o que muda é o conteúdo existir no html inicial.

o html em si, semântico de verdade: article, section, nav, aside, heading na hierarquia certa. não é estética, o geo-16 colocou html semântico entre os três pilares no topo da correlação com citação. e a regra que amarra tudo, a concordância tripla: texto visível, html semântico e dados estruturados dizem a mesma coisa. schema que descreve o que não está na página é, na melhor hipótese, ignorado, na pior, sinal de spam. higiene técnica clássica, core web vitals, https, sitemap, é o piso sobre o qual o resto se apoia, vale pro ranking e não atrapalha a citação.

---

## dados estruturados, declarando o nó

schema.org é como você diz pro server, explícito, o que a página é e como as coisas se conectam. é onde você declara o nó e as arestas internas dele. a diferença entre amador e estado da arte não é quantidade de schema, é topologia. os modelos de llm orquestrada por leigos, geralmente jogam vários blocos isolados de json-ld na página para entregar rapido, cada plugin cuspindo o seu, dois `Organization` divergentes brigando. estado da arte é um grafo só, um `@graph`, cada entidade declarada uma vez com `@id` estável, todo o resto apontando por `{ "@id": "..." }` em vez de duplicar o objeto. o `@id` é, literalmente, o endereço estável do nó dentro do grafo.

a regra de ancoragem, onde os guias sérios convergem: entidade persistente do site, que vale em todas as páginas, ancora na raiz do domínio com fragmento, `Organization` em `dominio.com/#organization`, `WebSite` em `dominio.com/#website`, `Person` dos autores em `dominio.com/#nome`. entidade de página ancora na url da própria página mais fragmento, `Article` em `dominio.com/artigo#article`. a entidade de página referencia as persistentes por `@id`, nunca duplica. e declare as persistentes antes das de página, é higiene pra que um parser que lê em ordem resolva os nós certos.



```json
{
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "Organization",
      "@id": "https://exemplo.com/#organization",
      "name": "Exemplo",
      "url": "https://exemplo.com",
      "sameAs": ["https://www.wikidata.org/wiki/Q000000"],
      "knowsAbout": ["generative engine optimization", "schema.org"]
    },
    {
      "@type": "Person",
      "@id": "https://exemplo.com/#autor",
      "name": "Autor",
      "sameAs": ["https://www.wikidata.org/wiki/Q000001"]
    },
    {
      "@type": "Article",
      "@id": "https://exemplo.com/artigo#article",
      "headline": "titulo",
      "author": { "@id": "https://exemplo.com/#autor" },
      "publisher": { "@id": "https://exemplo.com/#organization" },
      "datePublished": "2026-06-15",
      "dateModified": "2026-06-15",
      "speakable": { "@type": "SpeakableSpecification", "cssSelector": [".resumo", "h2"] }
    }
  ]
}
```

os tipos que mais importam, por ordem de prioridade: `Organization` com `sameAs` pro wikidata primeiro, é a fundação, o que conecta a sua marca ao knowledge graph. depois `Article` ou `BlogPosting` pro conteúdo, `BreadcrumbList` pra arquitetura, `Product` mais `Offer` pra e-commerce. dois campos subusados que pesam muito: `knowsAbout` em `Organization` e `Person`, com os tópicos reais de autoridade, e `sameAs` em `Person` dos autores, ancorando a entidade do autor. tipos emergentes que valem o esforço: `Dataset` pra qualquer conteúdo com tabela ou métrica, com `license` e `temporalCoverage`, porque alimenta pipeline de rag direto. `DefinedTerm` e `DefinedTermSet` pra vocabulário de nicho que não existe no wikidata. `SpeakableSpecification` apontando, via `cssSelector`, o trecho mais extraível.

os três bugs que toda auditoria pega, todos evitáveis com `@id` disciplinado: colisão, dois nós com o mesmo `@id` que o parser funde numa coisa só. referência órfã, um `@id` apontando pra entidade que não existe. e duplicação por cms, plugins como yoast e rankmath injetando cada um o seu bloco sem saber do outro. é o argumento mais forte pra centralizar a geração de schema numa camada única, em vez de delegar a plugin. valide no schema markup validator e no rich results test, com lint no ci/cd, porque schema quebra em silêncio: um `@id` corrompido só aparece semanas depois, quando a citação some.

e o caveat que muito guia esconde. a evidência de que schema, sozinho, melhora citação por ia é mista, não unânime. a única confirmação first-party de uma plataforma grande é do bing: fabrice canel disse no smx munich de março de 2025 que schema ajuda os llms da microsoft a entender conteúdo, e isso vale pro copilot. o google deu um aceno mais vago, dizendo que muitos sistemas rodam melhor com dados estruturados. openai, anthropic e perplexity não confirmaram nada. e tem contra-evidência: um estudo do search atlas não achou correlação entre cobertura de schema e taxa de citação. schema remove fricção pra quem já parseia html, é fundação pra entidade e knowledge graph, e o geo-16 o coloca entre os pilares, mas não é botão mágico. construa o grafo porque ele é contrato de dados limpo e vetor pra agente, não porque alguém prometeu citação garantida.

---

## faqpage, e o que o google aposentou em 2026

aviso prático pra não perseguir ornamento morto. em 7 de maio de 2026 o google parou de exibir os rich results de faq na busca, aquele sanfonado de pergunta e resposta embaixo do link. o filtro e o relatório no search console saem em junho, o suporte na api em agosto. nenhum blog post, nenhuma explicação, só um aviso pequeno no topo da documentação. foi o fim de um ciclo que começou em 2023, quando o google já tinha restringido o recurso a sites de governo e saúde depois do abuso da marcação pra inflar espaço na serp.

mas `FAQPage` continua sendo um tipo schema.org válido. o google depreciou a exibição, não a marcação, e disse que vai continuar usando o `FAQPage` pra entender a página. a marcação segue rastreável por bingbot, perplexitybot e pelos crawlers de rag. a regra prática: mantenha `FAQPage` onde a pergunta e a resposta são reais, úteis e visíveis na página, porque conteúdo liderado por pergunta é forte pra citação. remova só onde a seção era magra e existia pro enfeite de serp. o schema nunca fez o trabalho, o conteúdo sempre fez.

---

## crawlers de ia, a porta do nó

pra ser citado, os agentes de llm precisam alcançar o nó. mantenha destravados no robots.txt os user agents que importam: `GPTBot` e `OAI-SearchBot` da openai, `PerplexityBot` da perplexity, `ClaudeBot` da anthropic, `Google-Extended` que controla uso no gemini, `CCBot` do commoncrawl.

```
User-agent: GPTBot
Allow: /
User-agent: OAI-SearchBot
Allow: /
User-agent: PerplexityBot
Allow: /
User-agent: ClaudeBot
Allow: /
User-agent: Google-Extended
Allow: /
User-agent: CCBot
Allow: /
```

bloquear é decisão estratégica, faz sentido pra conteúdo pago, mas tira você de um canal de aquisição em crescimento. e um detalhe que derruba site sem ninguém ver: erro de servidor numa requisição a arquivo incomum, ou bloqueio acidental por cdn ou waf, pode estar barrando esses bots sem aparecer em relatório nenhum. cheque o log.

o `llms.txt`: índice em markdown de urls importantes, real, mas não é mecanismo de ranking nem de bloqueio. o google não usa e disse que não planeja, gary illyes confirmou no search central live de julho de 2025, e john mueller comparou o arquivo ao velho meta keywords tag, justo o que os buscadores ignoram há mais de uma década porque o dono do site controla e portanto manipula. log em centenas de milhares de domínios mostra que os crawlers que importam quase não tocam no arquivo. camada opcional, depois de html semântico, dados estruturados e sitemap. nunca prioridade.

---

## nlweb e mcp, o nó vira endpoint

o degrau seguinte, com o grafo estável: deixar de ser só página e virar endpoint que um agente consulta. o nlweb, da microsoft no build 2025, criado por r.v. guha, o mesmo do rss, rdf e schema.org, transforma um site em servidor mcp, o protocolo que a anthropic criou em novembro de 2024. expõe dois endpoints: `/ask`, que aceita pergunta em linguagem natural e devolve json estruturado em schema.org, e `/mcp`, agent-to-agent, que torna o site chamável por agentes como o operator da openai ou o project mariner do google, sem raspar html. early adopters: shopify, snowflake, o'reilly, tripadvisor, eventbrite, hearst.

isso fecha a topologia do guia: json-ld limpo deixou de ser só comida de crawler, virou o input que o nlweb consome e o formato que devolve. um `@graph` bem construído, com `@id` estável, vira api conversacional sem reescrever backend, e os melhores resultados vêm de sites já estruturados como lista de itens, receita, evento, produto. o repositório de referência está em `github.com/nlweb-ai/NLWeb`, em python, e aceita postgresql com pgvector entre os vector stores. e é a única saída pra app nativo sem versão web: não existe crawler lendo a tela de um app swift ou kotlin, então o vetor de descoberta por agente é expor `/ask` e `/mcp` sobre os dados, no backend que o app consome, não na ui.

---

## medir o que importa, do ranking à citação

a métrica mudou de eixo. posição numa lista de dez links era a régua antiga. a régua de hoje é share of citation, com que frequência a sua marca aparece nas respostas, e referral quality, o valor do clique que sobra, não o volume que minguou. medir certo é metade do trabalho, e é a metade que mais gente pula.

as ferramentas padrão são parcialmente cegas. o ga4 captura referral de quem clica num link dentro do chatgpt, da perplexity ou do claude, filtrando por `chatgpt.com`, `perplexity.ai`, `claude.ai`. mas não vê as 80% e mais de interações de ia que terminam sem clique, e parte do tráfego de ia perde o referrer e cai como "direct", inflando essa caixa e escondendo a origem. pior, o google não separa ai mode e ai overviews, os dois vêm embalados em `google / organic` com a busca tradicional. não há, hoje, jeito limpo de isolar isso no ga4.

então você triangula. o search console traz impressão e ctr e tem filtro de aparição em ai overviews. o log de servidor mostra quais páginas os bots de ia rastreiam de fato, o sinal mais direto do que o motor ingere. e o teste manual, que ferramenta nenhuma substitui: rode os prompts da sua categoria no chatgpt e na perplexity e veja se a marca aparece, e em que contexto. é onde a perda invisível fica visível. existem ferramentas de mercado pra share of citation e share of model, otterly.ai, profound e parecidas, categoria emergente que muda rápido, valide antes de adotar e não terceirize o julgamento. o tráfego de ia é difícil de atribuir por desenho, a influência mora dentro da resposta, e quem só olha o gráfico de tráfego mede a sombra, não o objeto.

---

## o que não muda, e os mitos que custam caro

por baixo de cada ciclo de pânico, os fundamentos continuam. a metodologia de 2023, entidade forte, conteúdo estruturado, internal linking, cobertura ganha, higiene técnica, autoridade de marca, vale em 2026, com legibilidade pra llm e citação por cima. o que muda devagar e compõe ao longo de anos é a camada estratégica, entidade, estrutura, autoridade. a maioria dos "x morreu" é ruído da camada comercial que nem chega na estratégica. mudança mecânica sem impacto comercial pede investigação, não ação.

os mitos têm a mesma raiz, confundir marcação com trabalho. schema não é botão de citação, é fundação e remoção de fricção, o conteúdo faz o trabalho. llms.txt não é fator de ranking nem de citação, é discoverability opcional. seo não morreu, o clique mudou de lugar, o volume de busca até cresceu. "faq schema importa mais que nunca pra ia" é exagero, o tipo continua válido, mas o valor sempre esteve na pergunta e resposta reais, não na tag. e o técnico é o piso: piso bom não te faz subir, só evita que você caia. com o piso de pé, a maior parte do retorno vem de conteúdo, autoridade e entidade. sem o piso, ele é o trabalho inteiro. o 80/20 é mnemônico, não proporção fixa, e stack quebrado deveria ignorá-lo e consertar a base antes de tudo.

---

## checklist

1. trate os dois jogos juntos: ranquear na busca clássica e ser citado por motor generativo, com base comum e camada extra de legibilidade pra máquina.
2. seja um nó, não um site: `sameAs` no `Organization` apontando pra wikidata, wikipedia, crunchbase, linkedin, github, g2, product hunt e orcid quando couber, com nome, descrição e logo consistentes em tudo. `sameAs` no `Person` dos autores.
3. lidere com a resposta, bloco autocontido nos primeiros 150 palavras, escrita por tópico e não por palavra-chave.
4. dado específico com fonte e citação de fonte primária no corpo, é o que os papers de geo mediram ganhando citação.
5. experiência de primeira mão visível, byline e credencial, método exposto, limite reconhecido. e-e-a-t como diferencial, não enfeite.
6. publique dado proprietário: pesquisa, benchmark, dataset ou experimento próprio. é a única alavanca de domínio próprio que o motor é forçado a citar, porque ninguém mais tem a fonte, e ela realimenta o nó da entidade e a aresta de earned media.
7. invista em earned media e digital pr, a aresta mais alta pra citação por ia, dominante sobre backlink no jogo de ser citado.
8. renderize no servidor, ssr, ssg ou isr, nunca conteúdo só no cliente, porque crawler de ia não executa js.
9. html semântico de verdade e concordância tripla: texto visível, html e json-ld dizendo a mesma coisa.
10. um único `@graph` por página, entidades persistentes com `@id` ancorado na raiz, de página ancorado na url, referência por `@id` e nunca duplicação.
11. `Organization` com `sameAs` pro wikidata primeiro, `knowsAbout` com tópicos reais, `Person` dos autores com `sameAs`, `Dataset` pra dado tabular.
12. centralize a geração de schema numa camada única, mate a duplicação de plugin, valide no ci/cd.
13. mantenha `FAQPage` só onde a pergunta e resposta são reais e visíveis, sem perseguir rich result morto.
14. destrave `GPTBot`, `OAI-SearchBot`, `PerplexityBot`, `ClaudeBot`, `Google-Extended`, `CCBot` no robots.txt, e cheque o log por bloqueio acidental.
15. nlweb `/ask` e `/mcp` como próximo passo depois do grafo estável, vetor obrigatório pra app nativo sem versão web.
16. llms.txt como reforço secundário, por último.
17. meça share of citation e referral quality, não posição e volume bruto. triangule ga4, search console, log de servidor e teste manual de prompt.
18. separe ruído de camada comercial de mudança de camada estratégica antes de reagir. o 80/20 entre conteúdo e técnico é heurística, não proporção fixa: stack quebrado torna o técnico o trabalho inteiro até o piso existir.

---

## bibliografia

eixo a, papers acadêmicos de geo:

1. aggarwal, murahari, rajpurohit, kalyan, narasimhan, deshpande. geo: generative engine optimization. kdd '24, pp. 5 a 16. doi 10.1145/3637528.3671900. arxiv 2311.09735.
2. kumar, palkhouski. ai answer engine citation behavior: an empirical analysis of the geo-16 framework. arxiv 2509.10762 (set 2025).
3. chen, wang, chen, koudas. generative engine optimization: how to dominate ai search. arxiv 2509.08919 (set 2025).
4. yu, yang, ding, sato. structural feature engineering for generative engine optimization. arxiv 2603.29979 (mar 2026).
5. what gets cited: competitive geo in ai answer engines. arxiv 2605.25517 (2026).

eixo b, tráfego, zero-click e ctr (2025 a 2026):

6. seer interactive. aio impact on google ctr, atualização de setembro 2025. queda de 61% no ctr orgânico com ai overview.
7. ahrefs. análise de cliques no primeiro resultado com ai overviews, dezembro 2025.
8. brightedge. dados de share de busca, ai overviews e crescimento de impressões.
9. press gazette. dados de referral de chatgpt e perplexity pra publishers.
10. sparktoro e similarweb. zero-click search study. rand fishkin, traffic is a terrible goal.
11. sq magazine. ai overviews statistics 2026. sqmagazine.co.uk/ai-overviews-statistics/
12. digitalapplied. zero-click search statistics 2026.

eixo c, earned media e citação por ia:

13. muck rack. generative pulse, análise de mais de 1 milhão de links citados por chatgpt, claude, gemini e perplexity (jul a dez 2025). earned media em 82%, fontes não pagas em 94%.
14. university of toronto. ai citation study, setembro 2025, 13 indústrias. earned media em 92%.
15. ahrefs. correlação de backlink (0,218) versus menção de marca (0,664) com visibilidade em ia.
16. moz. 88% das citações do google ai mode fora do top 10 orgânico.
17. agility pr / bulldog reporter. earned media as the new ai seo.

eixo d, schema, structured data e a posição de google e bing:

18. google search central. mark up faqs with structured data, com o aviso de deprecação de 7 de maio de 2026. developers.google.com/search/docs/appearance/structured-data/faqpage
19. schema.org. vocabulário, statement e claim. schema.org/version/latest, schema.org/Statement, schema.org/Claim
20. search engine roundtable. schema helps microsoft's llms (copilot). fabrice canel no smx munich, março 2025. seroundtable.com/schema-llms-copilot-bing-microsoft-39093.html
21. search atlas. estudo de cobertura de schema versus taxa de citação por llm (sem correlação).

eixo e, nlweb, mcp e a web agêntica:

22. microsoft source. introducing nlweb. news.microsoft.com/source/features/company-news/introducing-nlweb-bringing-conversational-interfaces-directly-to-the-web/
23. nlweb-ai/nlweb. implementação de referência. github.com/nlweb-ai/NLWeb
24. microsoft tech community. optimize your site for agents, e nlweb com postgresql e pgvector.

eixo f, llms.txt e crawlers:

25. limy.ai. llms.txt in 2026: the full guide. limy.ai/blog/llms.txt-in-2026-the-full-guide
26. search engine journal. llm guidance doesn't transfer the way seo guidance did. searchenginejournal.com/llm-guidance-doesnt-transfer-the-way-seo-guidance-did/575077/

eixo g, implementação técnica:

27. google/schema-dts. tipos typescript pra schema.org. github.com/google/schema-dts

eixo h, entidade e knowledge graph:

28. digitalapplied. entity seo & knowledge graph optimization guide 2026. knowledge graph do google com mais de 500 bilhões de fatos, sameAs oficialmente usado, menção de marca correlacionando 0,664 contra 0,218 de backlink.
29. clickrank. how to get your brand into google & openai knowledge graph 2026. truth nodes, wikidata e schema.org, entity signals.

eixo i, dado proprietário e o que ganha citação:

30. averi.ai. ai search citation benchmarks 2026. pesquisa original e dado proprietário citados a 38 a 65%, contra 6 a 15% de blog post; dado original levanta citação em 55 a 120%.
31. ziptie.dev. why original research wins ai citations. 41% de ganho de visibilidade com estatística (princeton e georgia tech, kdd 2024), e o ciclo de pesquisa, cobertura, menção e entidade.
32. andy crestodina, orbit media. pesquisa de aeo e geo: dado original, sinal de credencial e resposta autocontida como os três atributos que ganham citação.
