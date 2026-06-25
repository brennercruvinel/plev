---
title: livros de rust e wasm e o mercado editorial (refs para "por que este livro existe")
date: 2026-06-25
tags: [refs, livros, rust, wasm, mercado-editorial, caranguejovermelho, dimensao-books]
fontes:
  - https://doc.rust-lang.org/book/
  - https://nostarch.com/rust-programming-language-2nd-edition
  - https://nostarch.com/rust-programming-language-3e
  - https://www.amazon.com/Rust-Programming-Language-3rd/dp/1718504446
  - https://www.penguinrandomhouse.com/books/790517/the-rust-programming-language-3rd-edition-by-steve-klabnik-carol-nichols-and-chris-krycho-with-contributions-from-the-rust-community/
  - https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/
  - https://nostarch.com/rust-rustaceans
  - https://rust-for-rustaceans.com/
  - https://rustwasm.github.io/docs/book/
  - https://blog.rust-lang.org/inside-rust/2025/07/21/sunsetting-the-rustwasm-github-org/
  - https://pragprog.com/titles/khrust/programming-webassembly-with-rust/
  - https://www.packtpub.com/en-us/product/practical-webassembly-9781838828004
  - https://www.amazon.com/Practical-WebAssembly-Explore-fundamentals-programming/dp/1838828001
  - https://www.casadocodigo.com.br/products/livro-webassembly
  - https://novatec.com.br/livros/entendendo-algoritmos/
  - https://altabooks.com.br/produto/analise-pratica-de-series-temporais/
  - https://www.amazon.com.br/dp/8575225634
  - https://www.amazon.com.br/dp/132854639X
  - https://www.amazon.com.br/dp/B0DG37XVR6
  - https://www.amazon.com.br/dp/B0CM8TRWK3
  - https://www.amazon.com.br/dp/B0FG7NW67J
  - https://www.amazon.com.br/dp/8550815624
status_validacao: parcial (20 de 23 links retornam 200; 3 paginas de editora bloqueiam bot com 403 mas o conteudo foi confirmado por fetch direto e por listagens espelho; itens de metadado nao confirmados marcados no texto)
---

# livros de rust e wasm e o mercado editorial

material de referencia para o capitulo "por que este livro existe". a tese do
capitulo nao precisa de retorica: ela cai do inventario. existe uma prateleira
densa de livros de rust para engenheiro de sistemas, uma prateleira fina de
rust mais WASM, e um buraco quase total quando o leitor pretendido e um
adolescente de 13 a 17 anos lendo em portugues. o caranguejo vermelho mira esse
buraco. abaixo estao os livros, com fonte, mais a leitura honesta de para quem
cada um foi escrito.

metodo: cada item tem titulo, autoria, editora e dado de catalogo so quando a
fonte confirma. numero de copias, paginas ou data que a fonte nao mostra fica
marcado como nao confirmado. nao consolidei divergencia entre fontes; quando
duas fontes discordam, as duas aparecem.

## o canone do rust (a prateleira densa)

estes tres sao o que um dev serio compra. nenhum deles foi escrito pensando num
leitor de 14 anos, e isso e o ponto, nao uma critica.

### the rust programming language (klabnik, nichols, krycho)

o livro oficial. a 2a edicao tem copyright 2023, autoria de steve klabnik e
carol nichols, com contribuicoes creditadas a chris krycho e a comunidade rust,
ISBN 9781718503106, no starch press. o texto inteiro e gratuito em
`doc.rust-lang.org/book` e tambem sai impresso. o material de divulgacao da no
starch afirma "over 50,000 copies sold" (numero da editora, nao auditado por
terceiro: tratar como claim de marketing).

a 3a edicao muda a autoria: klabnik, nichols e krycho passam a constar como
coautores de pleno direito (ISBN 9781718504448, 624 paginas, no starch),
construida sobre a edicao 2024 do rust, com um capitulo novo de async e uso de
miri para analise de codigo unsafe. a penguin random house lista a data de
publicacao como marco de 2026; uma busca apontou 31 de marco de 2026, que trato
como nao confirmado por divergir de fonte primaria unica.

para quem e: dev que ja programa e quer aprender rust de verdade. ownership,
borrow, lifetimes, generics, traits, cargo. e a porta de entrada canonica, e e
densa. um iniciante absoluto, mais ainda um adolescente, esbarra em ownership e
lifetime na primeira semana sem andaime nenhum. e o oposto do que o caranguejo
vermelho propoe: aqui o leitor ja chega sabendo o que e um ponteiro.

### programming rust, 2a edicao (blandy, orendorff, tindall)

jim blandy, jason orendorff e leonora f. s. tindall. o'reilly media, junho de
2021, 738 paginas, cobre a edicao 2021 da linguagem. nivel declarado pela
propria o'reilly: intermediario a avancado, programacao de sistemas.

para quem e: quem vem de C ou C++ e quer o modelo de memoria do rust com rigor.
738 paginas dizem tudo sobre a audiencia. e um livro de referencia de
engenheiro, nao um primeiro contato.

### rust for rustaceans (jon gjengset)

jon gjengset, no starch press, copyright 2022, ISBN 9781718501850. o subtitulo
e "idiomatic programming for experienced developers". a propria pagina do livro
se posiciona como continuacao de onde "the rust programming language" para:
unsafe, sistema de traits, macros, async, testes. o numero exato de paginas nao
aparece de forma limpa nas fontes (uma listagem cita indice ate ~245): nao
confirmado.

para quem e: quem ja sabe rust e quer subir de nivel. uma resenha citada na
propria pagina e direta: "if you don't know rust it will be pretty much
inaccessible". e o teto da prateleira. menciono aqui justamente para mostrar a
distancia entre o teto e o chao onde o caranguejo vermelho comeca.

## rust mais wasm (a prateleira fina)

aqui a oferta ja rareia, e o item de referencia da comunidade acabou de ser
aposentado.

### the rust and webassembly book (rustwasm, arquivado em 2025)

o guia da working group rust + WASM, em `rustwasm.github.io/docs/book`, com o
tutorial classico do game of life compilado de rust para WASM, publicacao em
npm, debug, otimizacao de tamanho e deploy. nao cita versao de wasm-pack nem de
wasm-bindgen no corpo.

o site carrega hoje um aviso de que o projeto "is no longer maintained", com
link para o post oficial de 21 de julho de 2025 que encerra a organizacao
rustwasm no github (`blog.rust-lang.org/inside-rust/2025/07/21/...`). detalhe de
honestidade: esse post fala da organizacao e dos repositorios (wasm-bindgen
transferido para outra org; wasm-pack, gloo, twiggy, walrus e weedle
arquivados; org totalmente arquivada em setembro de 2025) e nao nomeia o livro.
a working group ja tinha sido arquivada em 2024 apos cerca de 5 anos inativa. ou
seja: "arquivado em 2025" se confirma pelo banner do proprio site do livro mais
pelo sunset da org que o hospeda, nao por uma frase que diga "o livro foi
arquivado". registro a nuance em vez de consolidar.

o que isso significa para o capitulo: o material gratuito de referencia de rust
mais WASM agora aponta para si mesmo como nao mantido. e exatamente o vacuo que
um livro novo, vivo e em portugues pode ocupar.

### programming webassembly with rust (kevin hoffman)

kevin hoffman, the pragmatic bookshelf (pragprog), marco de 2019, 238 paginas,
ISBN 9781680506365, subtitulo "unified development for web, mobile, and embedded
applications". a pagina diz que guia a instalacao das ferramentas por capitulo
mas nao fixa versao de toolchain, o que importa: WASM e o tooling rust mudaram
muito desde 2019. e um livro de 2019 num ecossistema que se reescreveu.

### practical webassembly (sendil kumar nellaiyapen)

sendil kumar nellaiyapen, packt publishing, 2 de maio de 2022, 232 paginas, ISBN
9781838828004, subtitulo "explore the fundamentals of webassembly programming
using rust". cobre os blocos do WASM, o tooling rust mais WASM e introduz rust
para construir aplicacoes WASM. nivel introdutorio a intermediario, em ingles.

## brasil (o buraco)

uma constatacao que sustenta o capitulo inteiro: em portugues, sobre WASM mais
rust de forma especifica, ha essencialmente um livro.

### desmistificando webassembly (raphael amorim)

raphael amorim, casa do codigo (editora ligada a alura), titulo completo
"desmistificando webassembly: alta performance, portabilidade e seguranca", ISBN
978-85-5519-346-0, 311 paginas, 2023. cobre fundamentos de WASM, seguranca,
performance e portabilidade, com enfase em rust mas tratando WASM como alvo
multilinguagem. a data exata de publicacao (a pagina da editora nao expoe dia e
mes de forma limpa; uma fonte cita setembro de 2023) fica como nao confirmada;
o ano 2023 e consistente entre fontes.

e um bom livro e e o unico do seu nicho em portugues. mas o leitor pretendido e
um desenvolvedor adulto. nao existe equivalente para adolescente, nem com a
ponte didatica que parte do zero. esse e o vao que o caranguejo vermelho mira:
nao competir com o amorim, e sim cobrir o leitor que ainda nao consegue ler o
amorim.

## best sellers fracos (o contraste do brenner)

o brenner marcou seis ASINs da amazon.com.br como best sellers tecnicos fracos,
para contrastar com a tese de "por que este livro existe". resolvi cada ASIN.
primeira divergencia honesta antes de qualquer analise: a etiqueta "best sellers
tecnicos (O'Reilly e afins)" nao bate com a lista. so dois dos seis sao livros
tecnicos, e nenhum dos dois e O'Reilly de origem. os outros quatro sao pop tech
ou pop ciencia sobre AI e sociedade, alguns em ingles, varios em audiobook. nao
forcei a etiqueta sobre o dado. abaixo estao identidade com fonte, por que vende
e onde e fraco. "por que vende" sai de posicionamento e marca, que sao fato.
"onde e fraco" e juizo editorial meu, marcado como analise, nao como dado.

### 1. entendendo algoritmos (ASIN 8575225634)

"entendendo algoritmos: um guia ilustrado para programadores e outros curiosos",
aditya y. bhargava. editora novatec no brasil, 264 paginas, 2017, ISBN
978-8575225639 (confirmado em novatec.com.br e na pagina amazon). nota de
origem: o original em ingles e "grokking algorithms", da manning, nao da
O'Reilly. a etiqueta "O'Reilly e afins" nao se aplica aqui.

por que vende: e ilustrado, acessivel, barato como porta de entrada, e cobre o
basico que cai em entrevista. o desenho carrega o leitor por recursao, busca
binaria, tabelas hash, grafos e programacao dinamica sem assustar.

onde e fraco (analise): e introdutorio de proposito. cobre um conjunto pequeno
de algoritmos e para onde o trabalho de verdade comeca. nao e referencia, nao
traz rigor nem prova, e o pseudocodigo em python nao vira pratica de engenharia
sozinho. vende porque e a primeira escada, e e fraco pela mesma razao: e so a
primeira escada.

### 2. ai superpowers (ASIN 132854639X)

"ai superpowers: china, silicon valley, and the new world order", kai-fu lee.
edicao em ingles vendida na amazon.com.br. a pagina nao expoe editora, ano nem
paginas: editora e data ficam nao confirmadas pela fonte consultada (de
conhecimento geral, houghton mifflin harcourt, 2018, mas nao confirmo isso pela
pagina, entao marco como nao confirmado).

por que vende: autoridade do autor (ex-chefe do google na china, depois
investidor) mais a narrativa china versus EUA na corrida de AI. nome forte,
tese vendavel.

onde e fraco (analise): e geopolitica e memoria, nao engenharia. e de 2018, ou
seja, anterior a onda de LLM que redefiniu o campo; boa parte das previsoes foi
atropelada pelos transformers e pelo que veio depois. para quem quer aprender a
construir, entrega quase nada tecnico.

### 3. nexus (ASIN B0DG37XVR6)

"nexus: uma breve historia das redes de informacao, da idade da pedra a
inteligencia artificial", yuval noah harari. edicao em audiobook, companhia das
letras, narracao de camilo schaden, traducao de berilo vargas e denise bottmann
(confirmado na pagina). ano nao exposto na pagina: nao confirmado pela fonte.

por que vende: marca harari, o halo de "sapiens", mais o hype de AI e a maquina
de distribuicao da companhia das letras. vende pelo nome na capa.

onde e fraco (analise): e historia pop generalista, larga e rasa no tecnico de
AI. nao e livro de tecnologia e nao se propoe a ser. entra na lista do brenner
como exemplo de best seller que toca o tema sem ensinar nada construivel.

### 4. co-intelligence (ASIN B0CM8TRWK3)

"co-intelligence: living and working with ai", ethan mollick. edicao em ingles,
ebook kindle (confirmado na pagina). editora e ano nao expostos: nao confirmado
pela fonte (de conhecimento geral, portfolio/penguin, 2024, mas nao confirmo
pela pagina).

por que vende: timing da onda chatgpt mais o alcance do mollick (professor da
wharton, muito lido nas redes) e o enquadramento pratico de "como trabalhar com
AI". util e oportuno.

onde e fraco (analise): envelhece rapido porque amarra o conteudo ao estado dos
modelos de 2023 e 2024. e mais regra de bolso e anedota do que engenharia
duravel. ajuda a usar a ferramenta, nao a entender ou construir nada por baixo.

### 5. a maquina do caos (ASIN B0FG7NW67J)

"a maquina do caos: como as redes sociais reprogramaram nossa mente e nosso
mundo", max fisher. edicao em audiobook, editora todavia, traducao de marcelo
levy e erico assis (confirmado na pagina). ano nao exposto na pagina: nao
confirmado pela fonte.

por que vende: narrativa de jornalista do new york times mais o zeitgeist do
dano das redes sociais, com a chancela da todavia. e reportagem bem contada.

onde e fraco (analise): nao e tecnico em nenhum ponto. e critica de midia e
sociedade, com uma historia causal de um lado so. nada de como os sistemas
funcionam por dentro. na lista, e o caso mais distante de "livro tecnico".

### 6. analise pratica de series temporais (ASIN 8550815624)

"analise pratica de series temporais: predicao com estatistica e aprendizado de
maquina", aileen nielsen. editora alta books no brasil, 1a edicao em portugues,
2021, 480 paginas, ISBN 9788550815626 (confirmado em altabooks.com.br e em
listagens de catalogo). o original e da O'Reilly; a alta books licencia titulos
O'Reilly no brasil, entao aqui sim a etiqueta "O'Reilly e afins" se sustenta.

por que vende: marca O'Reilly, series temporais como tema quente em ML e dados,
e cobertura larga (estatistica classica mais ML, com exemplos em R e python).

onde e fraco (analise): a propria largura e a fraqueza. trata muitos metodos de
forma rasa, e dividir entre R e python dilui os dois. ha relato de leitores
sobre exemplos de codigo que nem sempre rodam e sobre errata, mas nao confirmo
reclamacao individual com fonte linkada, entao marco essa parte como nao
confirmada. o que confirmo e o padrao editorial: amplitude que privilegia
cobertura sobre profundidade.

## tabela de validacao

| item | identidade confirmada | link 200 | status |
|------|----------------------|----------|--------|
| the rust programming language 2e | sim (no starch, copyright 2023, ISBN 9781718503106) | doc.rust-lang.org/book 200; nostarch 2e 403 (bot block) | validado (50k copias = claim de editora, nao auditado) |
| the rust programming language 3e | sim (klabnik, nichols, krycho; ISBN 9781718504448; 624p) | nostarch 3e 200; amazon 200; penguin 200 | validado; data exata "31 mar 2026" nao confirmada |
| programming rust 2e | sim (blandy, orendorff, tindall; o'reilly; jun 2021; 738p) | o'reilly 403 (bot), confirmado por webfetch | validado |
| rust for rustaceans | sim (gjengset; no starch; copyright 2022; ISBN 9781718501850) | rust-for-rustaceans.com 200; nostarch 200 | validado; paginas nao confirmadas |
| the rust and webassembly book | sim (rustwasm, banner "no longer maintained") | book 200; sunset post 200 | validado; "arquivado" via banner + sunset da org, post nao nomeia o livro |
| programming webassembly with rust | sim (hoffman; pragprog; mar 2019; 238p; ISBN 9781680506365) | pragprog 200 | validado |
| practical webassembly | sim (nellaiyapen; packt; mai 2022; 232p; ISBN 9781838828004) | packtpub 403 (bot), amazon 200 | validado |
| desmistificando webassembly | sim (raphael amorim; casa do codigo; 311p; ISBN 978-85-5519-346-0; 2023) | casadocodigo 200 | validado; dia/mes de 2023 nao confirmado |
| entendendo algoritmos (8575225634) | sim (bhargava; novatec; 264p; 2017; ISBN 978-8575225639) | amazon.com.br 200; novatec 200 | validado; origem manning, nao O'Reilly |
| ai superpowers (132854639X) | parcial (kai-fu lee; ingles) | amazon.com.br 200 | editora e ano nao confirmados pela pagina |
| nexus (B0DG37XVR6) | sim (harari; companhia das letras; audiobook) | amazon.com.br 200 | validado; ano nao confirmado pela pagina |
| co-intelligence (B0CM8TRWK3) | parcial (ethan mollick; ingles; kindle) | amazon.com.br 200 | editora e ano nao confirmados pela pagina |
| a maquina do caos (B0FG7NW67J) | sim (max fisher; todavia; audiobook) | amazon.com.br 200 | validado; ano nao confirmado pela pagina |
| analise pratica de series temporais (8550815624) | sim (nielsen; alta books; 480p; 2021; ISBN 9788550815626) | amazon.com.br 200; altabooks 200 | validado; reclamacoes de leitores nao confirmadas |

## o que o inventario prova

o canone do rust comeca para quem ja programa. o material gratuito de rust mais
WASM foi para o arquivo morto em 2025. em portugues existe um livro de WASM, bom
e para adulto. e os best sellers que tocam o tema, quando voce os resolve um a
um, ou sao introducao deliberadamente rasa ou nem sao tecnicos. nenhuma dessas
prateleiras fala com um adolescente brasileiro que quer compilar rust para a web
e ver rodar. essa cadeira esta vazia. e a cadeira do caranguejo vermelho.
