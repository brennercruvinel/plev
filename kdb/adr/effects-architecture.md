---
project: plev
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-08
domain: effects
---

# effects architecture (task-08)

## pipeline de efeitos (fragment shader only)

### blur (separável, 2 passes)
1. render source texture
2. h-pass: source -> temp_a (blur horizontal, 13-tap)
3. v-pass: temp_a -> temp_b (blur vertical, 13-tap)
4. result: temp_b

### shadow (3 passes + blur)
1. shadow extraction: source.alpha × shadow_color -> silhouette
2. blur silhouette (2 passes como acima)
3. result: blurred shadow texture

### composite
- full-screen triangle (3 verts, sem vbo, `@builtin(vertex_index)`)
- premultiplied alpha blending: `One / OneMinusSrcAlpha`
- opacity via uniform scalar multiplicando todos os canais

## texturepool
- keyed por `(width, height, format)`
- `acquire()` retorna texturehandle com view clonado (evita borrow conflict)
- `release()` marca como disponível para reuso
- grow-only: nunca destrói texturas
- `invalidate_size()` no resize: remove texturas de tamanho antigo

## gaussian weights
- 13-tap (center + 6 symmetric offsets)
- pesos precomputados no CPU via `gaussian_weights(sigma)`
- normalizados para somar 1.0
- layout: `[f32; 16]`, 13 weights + 3 padding (vec4 alignment no WGSL)

## bind groups
- group 0: texture + sampler (compartilhado por blur/shadow/composite)
- group 1: uniforms específicos do efeito (bluruniforms/shadowuniforms/compositeuniforms)
- max 2 bind groups, dentro do limite WASM de 4

## cuidados
- textureview::clone() é safe em wgpu 28 (reference-counted)
- blur uniform buffer é compartilhado entre h e v pass via write_buffer entre passes
- texturas do pool precisam de `RENDER_ATTACHMENT | TEXTURE_BINDING`
