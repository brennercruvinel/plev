---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-11
domain: geometry
---

# reference analysis: math, physics e geometry

## escopo

analise factual de quatro bibliotecas rust relevantes para um motor de composicao GPU-first (φ): matematica linear com SIMD, fisica 2d/3d, algebra linear completa, e tesselacao de paths para GPU. foco em arquitetura interna, compatibilidade WASM/multi-plataforma, e viabilidade de integracao com pipeline wgpu existente.

dados coletados em marco de 2026 via github, crates.io, documentacao oficial e benchmarks publicados.

---

## repositorios analisados

### glam (bitshifter/glam-rs), ~1.9k stars, v0.32.1

**o que e:** biblioteca de algebra linear otimizada para games e graficos. tipos fundamentais: vec2, vec3, vec3a (16-byte aligned), vec4, mat2, mat3, mat3a, mat4, quat, affine2, affine3a. variantes f32, f64, i32, u32, i64, u64, bool. licenca MIT/apache-2.0.

**arquitetura:**
- tipos SIMD-first: vec3a, vec4, mat4, quat armazenam dados internamente como `__m128` (sse2), `float32x4_t` (NEON/aarch64), ou `v128` (WASM simd128). vec2 e vec3 usam escalares (sem padding).
- sem traits genericos por design, cada tipo e concreto. isso permite inlining agressivo e otimizacao SIMD sem indirection.
- `no_std` suportado (`--no-default-features --features libm`).
- features opcionais: `bytemuck` (cast para `&[u8]`, essencial para upload GPU), `serde`, `mint` (interop com outras libs math), `encase` (uniform buffer layout para wgpu).
- WASM: simd128 ativado via `RUSTFLAGS="-C target-feature=+simd128"`. fallback escalar automatico se nao disponivel.
- 38.6m+ downloads totais no crates.io (81 versoes). MSRV 1.68.2.

**relevancia para φ:**
- φ ja usa `[f32; 2]` e `[f32; 4]` crus para posicoes e cores nos vertex buffers. glam substituiria com vec2/vec4 tipados, ganhando operacoes SIMD (lerp, normalize, transform) sem overhead.
- `bytemuck::Pod` implementado para todos os tipos glam, upload direto para GPU via `bytemuck::cast_slice()`.
- mat4 com projecao ortografica (y-down, como φ usa) disponivel via `Mat4::orthographic_rh`.
- bevy, wgpu-examples, e a maioria do ecossistema rust game-dev usa glam. interop sem friccao.

**insight principal:** glam nao usa generics/traits para tipos matematicos. consequencia: mat4::inverse() retorna mat4 (nao option<mat4>), assumindo que a matriz e invertivel. se nao for, o resultado contem nan. isso e uma decisao de performance, evita branch no hot path. para φ (projecao ortografica bem-comportada), isso e seguro.

**limitacao:** sem decomposicoes matriciais (svd, qr, lu). sem matrizes de dimensao arbitraria. projetado exclusivamente para 2d/3d/4d, nao e algebra linear geral.

---

### rapier (dimforge/rapier), ~5.1k stars, v0.25.x

**o que e:** motor de fisica 2d e 3d para games, animacao e robotica. quatro crates: rapier2d, rapier3d, rapier2d-f64, rapier3d-f64. licenca apache-2.0.

**arquitetura:**
- pipeline modular: physicspipeline (step), querypipeline (raycasts, intersections), islandsolver (sleeping/waking).
- dependencia forte: nalgebra (algebra linear) + parry2d/parry3d (deteccao de colisao, BVH).
- sem generics nas structs de dados, compilacao incremental mais rapida que nphysics (predecessor generico).
- broad-phase: dynamic BVH com rebalancing automatico e travessia SIMD.
- narrow-phase: GJK + EPA, suporte a convex hulls, trimesh, heightfields, voxels (novo em 0.25).
- determinismo cross-platform: build separado (`rapier2d-deterministic`) garante ieee 754-2008 compliance. build padrao prioriza performance.
- WASM: compilavel para wasm32 com bindings JS oficiais (`rapier.js`). em 2025, versoes npm com SIMD WASM ficaram 2-5x mais rapidas que v0.24.
- serializacao de estado: snapshot + restore do estado completo da simulacao (serde).
- 700k+ downloads (rapier2d), 1.1m+ (rapier3d) no crates.io.

**relevancia para φ:**
- φ e um motor de composicao, nao game engine. fisica completa (rigid bodies, joints, CCD) e excessiva para o escopo atual.
- caso de uso potencial futuro: scroll deceleration com atrito fisico, spring animations com integracao verlet, deteccao de colisao para drag-and-drop complexo.
- querypipeline isoladamente poderia ser util para spatial queries (point-in-shape, ray casting), mas φ ja tem hit-testing linear reverso que e suficiente para UI.
- peso: rapier2d traz nalgebra + parry2d + simba como dependencias transitivas, impacto significativo em compile time e binary size.

**insight principal:** o novo BVH dinamico com SIMD do rapier e estado-da-arte para broad-phase. se φ eventualmente precisar de spatial indexing para milhares de elementos interativos, a arquitetura do parry (que pode ser usada independente do rapier) seria a referencia.

**limitacao:** rapier nao e modular no sentido de "use so o solver de colisao sem o pipeline de fisica". usar parry2d diretamente e possivel mas a API e de nivel mais baixo. compile time significativo (~30s clean build adicionais estimado).

---

### nalgebra (dimforge/nalgebra), ~4.2k stars, v0.34.1

**o que e:** biblioteca de algebra linear geral. tipo unico parametrizavel `Matrix<T, R, C, S>` para vetores, matrizes, e transformacoes de qualquer dimensao. licenca apache-2.0.

**arquitetura:**
- core type: `Matrix<T, R, C, S>`, t (escalar), r (rows), c (cols), s (storage). dimensoes podem ser const-generics (compile-time) ou dynamic (heap-allocated).
- const-generics desde v0.26: armazenamento `[[T; R]; C]` para matrizes estaticas. antes usava genericarray (typenum).
- aliases: vector1..vector6, matrix1x1..matrix6x6, dvector, dmatrix (dynamic).
- decomposicoes: cholesky, qr, lu, fullpivlu, svd, schur, hessenberg, symmetriceigen.
- sub-crates: nalgebra-glm (API estilo glm/c++), nalgebra-lapack (lapack bindings), nalgebra-sparse (sparse matrices).
- `no_std` suportado. WASM compativel.
- 55m+ downloads totais no crates.io (121 versoes).
- usado por rapier, parry, e todo ecossistema dimforge.

**relevancia para φ:**
- φ nao precisa de decomposicoes matriciais, matrizes esparsas, ou dimensoes dinamicas. o escopo de nalgebra excede vastamente as necessidades de um compositor GPU 2d.
- nalgebra-glm poderia ser uma alternativa a glam, mas benchmarks (mathbench-rs) mostram glam 1.5-3x mais rapido em operacoes comuns (mat4 mul, inverse, transform) devido a SIMD nativo e inlining agressivo.
- se φ adotar rapier/parry no futuro, nalgebra entra como dependencia transitiva automaticamente, nao precisa ser adotada diretamente.

**insight principal:** nalgebra usa generics pesados (matrix<t,r,c,s>) que impactam compile time e dificultam inlining em hot paths. glam evita isso por design. para operacoes de graficos 2d/3d (que sao 99% do uso em φ), glam e objetivamente superior em ergonomia e performance.

**limitacao:** performance inferior a glam em operacoes 3d/4d comuns. curva de aprendizado maior por causa do sistema de tipos generico. `Matrix::inverse()` retorna `Option<Matrix>` (correto matematicamente, mas branch no hot path).

---

### lyon (nical/lyon), ~2.4k stars, v1.0.16

**o que e:** biblioteca de tesselacao de paths 2d para renderizacao GPU. converte curvas bezier, arcos, e paths SVG-compliant em triangulos (vertex + index buffers). licenca MIT/apache-2.0.

**arquitetura:**

- **pipeline de tesselacao:**
  1. `Path` construido via `PathBuilder` (API estilo SVG: `move_to`, `line_to`, `quadratic_bezier_to`, `cubic_bezier_to`, `arc`, `close`)
  2. curvas achatadas (flattened) em segmentos de linha com tolerancia configuravel (`FillOptions::tolerance(0.01)` = max 0.01px entre curva e aproximacao)
  3. algoritmo de decomposicao monotona (single-pass, nao three-pass como implementacoes classicas)
  4. output: vertices + indices via trait `GeometryBuilder`

- **dois tessellators principais:**
  - `FillTessellator`: preenche o interior de paths complexos (mesmo com self-intersections, winding rules)
  - `StrokeTessellator`: extruda strip de triangulos ao longo do path (line caps, joins, dash patterns, miter limit)

- **tessellators especializados:** circulos, retangulos arredondados, poligonos convexos, polylines, evitam overhead do tessellator geral para formas comuns.

- **sub-crates:**
  - `lyon_tessellation`: core (fill + stroke)
  - `lyon_path`: construcao e iteracao de paths
  - `lyon_geom`: matematica de bezier (subdivide, flatten, intersect, bounding box)
  - `lyon`: re-export unificado

- **geometrybuilder trait:** output desacoplado do tessellator. `VertexBuffers<V, I>` e a implementacao padrao (vec<v> + vec<u16/u32>), mas qualquer struct pode implementar o trait. `BuffersBuilder` mapeia atributos do tessellator (posicao, normal) para vertex type customizado via closure.

- **performance:**
  - ~2x mais rapido que libtess2 (referencia c) em benchmarks com rust logo e ghostscript tiger.
  - attributes (cor, normal) lazy-computed via `FillVertex::interpolated_attributes`, zero overhead se nao usados.
  - tolerancia controla tradeoff qualidade/vertices: 0.01 = alta qualidade (mais triangulos), 1.0 = baixa qualidade (menos triangulos). para UI a 2x DPI, 0.25 e tipicamente suficiente.

- 3.3m+ downloads totais no crates.io (62 versoes). MSRV nao documentado. estavel desde 1.0 (jan 2021, ultimo breaking change).

**relevancia para φ (critica):**

φ atualmente renderiza apenas quads (retangulos axis-aligned) via instanced rendering. lyon e a ponte para formas arbitrarias:

1. **rounded rectangles:** lyon tem tessellator especializado (`lyon::tessellation::basic_shapes::fill_rounded_rectangle`). alternativa ao approach atual de φ que faz rounding no fragment shader (SDF). tradeoff: tessellacao gera mais vertices mas elimina branch no shader.

2. **custom shapes para icones:** paths SVG tesselados uma vez no init, geometry mantida na GPU. zero work per-frame para icones estaticos.

3. **charts e graficos:** linhas curvas (bezier), areas preenchidas, pie charts, tudo via path tessellation.

4. **integracao com pipeline wgpu existente:**
   - lyon gera `VertexBuffers { vertices: Vec<GpuVertex>, indices: Vec<u32> }` onde `GpuVertex` e definido pelo usuario.
   - para φ: definir `GpuVertex` com `position: [f32; 2]` + `color: [f32; 4]` (mesmo layout do quad pipeline atual).
   - upload para `wgpu::Buffer` via `bytemuck::cast_slice()`.
   - renderizar com `draw_indexed()` no mesmo render pass dos quads.
   - pode compartilhar o mesmo shader (quad.wgsl) se o vertex layout for identico, ou usar shader dedicado para shapes (com antialiasing via SDF no edge).

5. **tessellacao e offline:** acontece no CPU, tipicamente uma vez (shapes estaticas) ou quando path muda (animacao de morph). nao e per-frame para UI estatica. custo: ~1ms para paths complexos (tiger SVG completo), microsegundos para formas simples.

**insight principal:** o exemplo oficial `lyon/examples/wgpu/` demonstra o padrao completo: define `GpuVertex` customizado, tessela no init, faz upload unico para GPU, renderiza com `draw_indexed()`. a geometria fica persistente na GPU, exatamente o pattern que φ ja usa com gpuvec (grow-only). a integracao seria natural: lyon gera a geometria, gpuvec armazena, o render pass desenha.

**limitacao:** lyon faz tesselacao no CPU, para paths muito complexos com animacao per-frame (morphing SVG), o custo pode ser relevante. nao faz antialiasing (responsabilidade do shader/MSAA). nao renderiza texto (φ ja tem sistema proprio com cosmic-text). nao tem suporte a gradientes ou texturas no tessellator, isso e responsabilidade do shader.

---

## padroes cross-cutting

### 1. SIMD multi-plataforma e realidade
glam, rapier, e lyon todos lidam com SIMD de formas diferentes. glam abstrai sse2/NEON/simd128 internamente. rapier usa nalgebra+simba para SIMD. lyon opera no nivel escalar (tesselacao e CPU-bound, nao SIMD). para φ, o unico SIMD relevante no hot path e o de glam (transformacoes de vertices).

### 2. bytemuck como ponte cpu-gpu
glam + bytemuck e o padrao de facto para upload de dados para wgpu. lyon + bytemuck (via vertex type customizado com `#[repr(C)]` + `Pod + Zeroable`) completa o pipeline. nalgebra nao tem suporte bytemuck nativo, mais um motivo para preferir glam.

### 3. compilacao e tamanho de binario
- glam: impacto minimo (~5s compile, binario negligivel). zero dependencias obrigatorias.
- lyon: impacto moderado (~10s compile). dependencias leves (lyon_geom usa apenas tipos primitivos).
- nalgebra: impacto significativo (~15-20s compile). generics pesados, monomorphization extensiva.
- rapier2d: impacto alto (~30s+ compile). traz nalgebra + parry + simba + bitflags + crossbeam.

### 4. `no_std` e WASM
todos suportam WASM. glam e lyon sao `no_std` friendly. nalgebra suporta `no_std` com feature flag. rapier funciona em WASM mas requer std.

### 5. estabilidade da API
- glam: pre-1.0 (v0.32), mas mudancas sao tipicamente aditivas. bevy depende dele, breaking changes sao raros.
- lyon: 1.0 estavel desde 2021. API congelada para os tipos core.
- nalgebra: v0.34, mudancas ocasionais mas bem documentadas.
- rapier: v0.25, evolui mais rapido. voxels shape adicionado recentemente.

---

## implicacoes para φ

### adocao imediata recomendada

**glam**, substituir `[f32; 2]` / `[f32; 4]` crus por `Vec2` / `Vec4` nos vertex buffers e transformacoes. adicionar `Mat4` para projecao (substituir calculo manual atual). feature `bytemuck` para upload GPU. feature `serde` se serialization de scene for necessario. impacto: melhoria de ergonomia e performance SIMD com zero risco de regressao.

**lyon**, adicionar como dependencia para shapes customizadas quando φ expandir alem de quads. integracao natural com gpuvec existente. nao precisa ser imediata, quando o primeiro caso de uso aparecer (rounded rect via tessellacao, icone SVG, chart), lyon e a escolha obvia e bem-testada. considerar `lyon_tessellation` isoladamente (sem `lyon_path`) se quiser minimizar dependencias.

### monitorar

**rapier / parry**, nao adotar agora. φ nao precisa de fisica. se spring animations ou scroll physics forem implementadas, considerar: (a) implementar verlet/spring manualmente (< 50 linhas), ou (b) usar parry2d isoladamente para spatial queries se o numero de elementos interativos justificar BVH. rapier completo e overkill.

### nao adotar

**nalgebra**, nao ha caso de uso que justifique nalgebra sobre glam para φ. se rapier/parry for adotado no futuro, nalgebra entra como dependencia transitiva e fica encapsulada, φ nao deve usar nalgebra diretamente na API publica.
