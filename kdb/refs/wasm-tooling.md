---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: wasm
---

# analise de referencia: WASM tooling ecosystem

## escopo

levantamento factual de sete ferramentas do ecossistema webassembly relevantes para o φ, engine de composicao GPU-first em rust que ja utiliza trunk para builds WASM/webgpu. o objetivo e documentar estado atual, arquitetura e implicacoes praticas, nao recomendar adocao.

data da pesquisa: 2026-03-11.

---

## repositorios analisados

### 1. trunk (trunk-rs/trunk), 4.207 stars, v0.21.14

**o que e:** bundler dedicado a aplicacoes rust/WASM para web. usa um arquivo HTML como ponto de entrada, compila o crate para `wasm32-unknown-unknown`, gera bindings via wasm-bindgen, e serve com dev-server integrado.

**arquitetura:** pipeline baseado em assets: o HTML fonte referencia o crate rust, trunk resolve dependencias (WASM, CSS, scss, JS snippets, imagens), invoca `cargo build --target wasm32-unknown-unknown`, executa `wasm-bindgen`, empacota tudo em `dist/`. dev-server com hot-reload via websocket e proxy http configuraveis.

**relevancia para φ:** ferramenta atual do projeto. `index.html` com `data-target-name="φ"` gera o artefato WASM. o entry point e `#[wasm_bindgen(start)] fn wasm_main()` em `lib.rs`. o binario se chama `φ-app` (nao `φ`) justamente para evitar colisao de nomes com o artefato WASM gerado pelo trunk.

**insight principal:** trunk e opinativo, funciona bem para spas rust-puras onde o HTML e minimo e o rust controla tudo via wasm-bindgen. esse modelo encaixa perfeitamente no φ, que renderiza via webgpu canvas sem DOM.

**limitacao:** 127 issues abertas. ultima release (v0.21.14) em maio de 2025, gap de ~10 meses sem release nova ate a data desta pesquisa. nao suporta webassembly component model. nao gera pacotes npm (o artefato e um bundle auto-contido, nao uma lib JS reutilizavel). nao tem suporte nativo a code splitting ou lazy loading de modulos WASM.

---

### 2. wasm-pack (rustwasm/wasm-pack), 7.115 stars, v0.14.0

**o que e:** cli que compila crates rust em pacotes npm-compativeis, gerando bindings JS/TS via wasm-bindgen e empacotando com `package.json` pronto para publicacao no npm.

**arquitetura:** pipeline: `cargo build --target wasm32-unknown-unknown` -> `wasm-bindgen` (gera glue JS + `.wasm`) -> `wasm-opt` (otimizacao opcional) -> gera `package.json` com metadados do `Cargo.toml`. suporta targets: `bundler` (webpack/rollup), `web` (esm nativo), `nodejs`, `no-modules`. inclui subcomandos `test` (wasm-pack test via headless browser), `pack`, `publish`.

**relevancia para φ:** cenario futuro, se φ expuser uma API JS para embedding em aplicacoes web existentes (ex: um componente react que renderiza via φ internamente), wasm-pack seria o caminho para distribuir via npm. atualmente irrelevante porque φ e auto-contido (nao e uma lib JS).

**insight principal:** wasm-pack e complementar ao trunk, nao substituto. trunk serve aplicacoes completas; wasm-pack empacota bibliotecas para consumo JS. um projeto pode usar ambos: wasm-pack para o core como pacote npm, trunk para a demo/showcase.

**limitacao:** 391 issues abertas. foco exclusivo em `wasm32-unknown-unknown` (nao suporta WASI ou component model). nao inclui dev-server. o glue JS gerado assume ambiente browser ou node, nao ha suporte direto a runtimes como deno ou bun sem ajustes.

---

### 3. extism (extism/extism), 5.474 stars, v1.13.0

**o que e:** framework leve para execucao de plugins webassembly. permite que aplicacoes host carreguem e executem codigo WASM de terceiros com isolamento, controle de recursos e comunicacao bidirecional.

**arquitetura:** modelo host/plugin com duas camadas:
- **host sdks** (16 linguagens: rust, JS, go, python, java, .net, etc.): integram o runtime wasm na aplicacao host, gerenciam ciclo de vida dos plugins.
- **plugin pdks** (9 linguagens: rust, JS, go, python, c, c++, etc.): libs que o autor do plugin usa para interagir com o host (ler input, retornar dados, acessar config, fazer http quando permitido).

memoria persistente entre chamadas ao mesmo plugin. http controlado pelo host (sem depender de WASI). limitadores de tempo e memoria por plugin.

**relevancia para φ:** alta para arquitetura futura. se φ evoluir para permitir plugins de terceiros (ex: shaders customizados, widgets, efeitos visuais, geradores de cena), o modelo extism resolve o problema de executar codigo nao-confiavel com isolamento. o host (engine φ) controlaria exatamente quais recursos cada plugin acessa. cada plugin seria um `.wasm` que implementa uma interface definida pelo φ.

**insight principal:** o modelo de host functions do extism e o padrao mais maduro para plugin systems em WASM. a separacao host SDK / plugin pdk permite que autores de plugins usem qualquer linguagem que compile para WASM, nao apenas rust.

**limitacao:** android nao suportado nativamente (recomendam chicory, um runtime java). performance nao documentada com benchmarks publicos. o runtime embarcado adiciona overhead de inicializacao por plugin. 51 issues abertas.

---

### 4. serde-wasm-bindgen (rreverser/serde-wasm-bindgen), 608 stars, v0.6.5

**o que e:** integracao nativa entre serde e wasm-bindgen que converte tipos rust diretamente em objetos javascript nativos (map, array, typedarray, etc.), sem passar por JSON como intermediario.

**arquitetura:** duas funcoes principais:
- `serde_wasm_bindgen::to_value(&rust_struct)` -> `JsValue` (serializa rust -> JS nativo)
- `serde_wasm_bindgen::from_value(js_value)` -> `RustStruct` (deserializa JS nativo -> rust)

internamente, caminha a arvore serde e constroi objetos JS via chamadas wasm-bindgen diretas, evitando o round-trip `Rust -> JSON string -> JS parse` que `serde-json` + `JsValue::from_serde()` faria.

**relevancia para φ:** relevante se φ implementar comunicacao bidirecional com JS no target WASM, ex: receber configuracao de cena como objeto JS, enviar metricas de performance, ou expor API de criacao de nodes via JS. a diferenca de performance vs JSON e significativa para objetos grandes ou chamadas frequentes.

**insight principal:** o readme afirma "much smaller code size overhead than JSON, and, in most common cases, much faster serialization/deserialization." a vantagem nao e so velocidade, e tamanho do binario WASM, porque nao precisa incluir o parser JSON completo do serde_json.

**limitacao:** inteiros de 64 bits (u64, i64) requerem tratamento especial (bigint). nao gera tipos typescript automaticamente (precisa do crate `tsify` separado). 5 issues abertas. nao tem releases tagadas no github (publicacao apenas via crates.io).

---

### 5. spin (fermyon/spin), 6.332 stars, v3.6.2

**o que e:** framework open-source para construir e executar microservicos serverless baseados em webassembly. desenvolvido pela fermyon, usa o webassembly component model e o runtime wasmtime.

**arquitetura:** aplicacoes spin sao conjuntos de componentes WASM definidos em `spin.toml`. cada componente responde a triggers (http, redis, timer, custom). o runtime spin instancia o componente sob demanda, executa, e descarta, modelo serverless classico. suporta storage embutido (key-value, sqlite), outbound http, e variaveis de configuracao.

sdks oficiais: rust, javascript, python, go. sdks comunitarios: zig, moonbit.

**relevancia para φ:** baixa para o core. spin e voltado para backend serverless, nao rendering. porem, relevante se φ tiver um servico backend (ex: asset server, collaboration server, render farm), componentes spin poderiam processar requests com latencia sub-milissegundo de cold start (vantagem WASM vs containers).

**insight principal:** o webassembly component model que spin adota e o futuro da interoperabilidade WASM. componentes podem se compor e comunicar via interfaces tipadas (wit, webassembly interface types). se esse modelo amadurecer, φ poderia definir interfaces wit para plugins, tornando-os interoperaveis entre runtimes.

**limitacao:** foco exclusivo em servidor (sem suporte a browser/client-side). custom triggers so em rust. algumas apis nao disponiveis em todos os sdks (ex: mysql nao disponivel em python, redis trigger nao disponivel em c#). plataforma fermyon cloud e comercial.

---

### 6. lunatic (lunatic-solutions/lunatic), 4.845 stars, v0.13.2

**o que e:** runtime webassembly inspirado na beam (erlang/otp). executa modulos WASM como processos leves com isolamento, message passing e supervisao, conceitos diretamente mapeados da filosofia erlang.

**arquitetura:** cada modulo WASM roda como um processo leve com stack, heap e syscalls proprios. scheduler preemptivo com work-stealing. comunicacao via canais (channels), nao memoria compartilhada. permissoes granulares por processo (filesystem, memoria, rede). suporte a distribuicao entre nodes.

**relevancia para φ:** conceitual. o modelo de processos isolados com supervisao e relevante se φ precisar de concorrencia robusta no lado servidor (ex: multiple render contexts, hot-reload de plugins sem derrubar o host). o padrao "let it crash" do erlang, aplicado via WASM isolation, e elegante para fault tolerance.

**insight principal:** lunatic demonstra que WASM pode servir como mecanismo de isolamento de processos alem de sandbox de seguranca. cada processo WASM e mais leve que uma thread OS e mais isolado que uma green thread.

**limitacao:** ultimo release (v0.13.2) em maio de 2023. ultimo commit em marco de 2025 (fix de build). gap de ~3 anos sem release. apenas rust e assemblyscript como linguagens suportadas. hot reloading incompleto. compatibilidade WASI planejada mas nao concluida. o projeto aparenta estar em estado de baixa atividade ou abandono parcial.

---

### 7. workers-rs (cloudflare/workers-rs), 3.374 stars, v0.7.5

**o que e:** SDK rust para cloudflare workers, permite escrever workers serverless inteiramente em rust, compilados para WASM e executados na edge network da cloudflare.

**arquitetura:** o crate `worker` fornece bindings rust para as apis da plataforma workers (kv, durable objects, r2, d1, queues). compila para `wasm32-unknown-unknown`. deploy via wrangler cli. suporta integracao com frameworks http rust (ex: axum) via feature flag `http`.

macro `#[event(fetch)]` define o handler principal. request/response seguem a API fetch padrao (ou `http` crate com flag).

**relevancia para φ:** especifica para deploy na cloudflare. relevante se φ tiver servicos edge (ex: asset cdn inteligente, pre-processamento de cenas, API gateway). nao relevante para o engine em si.

**insight principal:** demonstra o padrao "rust -> WASM -> edge" em producao. a integracao com axum via feature flag e um bom exemplo de como manter compatibilidade entre runtime nativo e WASM com o mesmo codigo http.

**limitacao:** marcado explicitamente como "work-in-progress." 165 issues abertas. fortemente acoplado a plataforma cloudflare (nao portavel para outros provedores). breaking changes frequentes entre versoes da feature `http`.

---

## padroes cross-cutting

1. **dois mundos, dois toolchains:** trunk e wasm-pack servem propositos diferentes e complementares. trunk = aplicacao completa. wasm-pack = biblioteca npm. o ecossistema nao convergiu para uma ferramenta unica.

2. **component model como futuro:** spin e extism apostam no webassembly component model (wit interfaces, composabilidade). trunk e wasm-pack ainda operam no modelo "modulo monolitico." a transicao sera gradual e impactara como plugins e libs sao distribuidos.

3. **plugin isolation via WASM:** extism e lunatic demonstram que WASM e viavel como mecanismo de isolamento para codigo de terceiros. o overhead de instanciacao e aceitavel para plugins que persistem entre chamadas (extism) ou processos de vida longa (lunatic).

4. **serialization boundary:** serde-wasm-bindgen resolve o gargalo de comunicacao rust<->JS com zero JSON. qualquer projeto que cruze a fronteira WASM/JS com frequencia se beneficia, a alternativa (serde_json + jsvalue::from_serde) adiciona tanto latencia quanto tamanho de binario.

5. **serverless WASM maturo para backend:** spin e workers-rs demonstram que WASM serverless ja e producao (fermyon cloud, cloudflare). cold start sub-milissegundo e a vantagem competitiva vs containers.

6. **manutencao desigual:** lunatic parece em declinio (3 anos sem release). trunk tem gap de 10 meses. wasm-pack, spin, extism e workers-rs mantem cadencia de releases. avaliar saude do projeto e tao importante quanto avaliar features.

---

## implicacoes para φ

### curto prazo (manter)
- **trunk** continua adequado para o build WASM do φ. o modelo spa sem DOM e exatamente o caso de uso. monitorar o gap de releases, se trunk for abandonado, a alternativa mais proxima e configurar `wasm-pack` com target `web` + dev-server externo (ex: vite).

### medio prazo (considerar)
- **serde-wasm-bindgen**, se φ expuser API JS no target WASM (configuracao de cena, eventos, metricas), usar serde-wasm-bindgen em vez de serde_json para a fronteira rust<->JS. ganho duplo: performance e tamanho do binario.
- **wasm-pack**, se φ precisar ser distribuido como pacote npm (embedding em apps react/vue/svelte), wasm-pack gera o pacote. pode coexistir com trunk.

### longo prazo (inspiracao arquitetural)
- **extism**, modelo de referencia se φ implementar plugin system. host functions definidas pelo engine, plugins como `.wasm` com pdk. isolamento gratuito. suporte multi-linguagem para autores de plugins.
- **webassembly component model (via spin)**, definir interfaces wit para plugins φ tornaria os plugins interoperaveis e versionaveis. depende da maturidade do toolchain (wit-bindgen, wasm-tools).
- **lunatic**, o conceito de processos isolados com supervisao e relevante para fault tolerance em render pipelines complexos, mas o projeto em si nao e recomendavel dado o estado de manutencao.

### nao relevante
- **spin** e **workers-rs**, voltados para backend serverless. so se tornam relevantes se φ tiver servicos de infraestrutura (asset server, collaboration, render-as-a-service).
