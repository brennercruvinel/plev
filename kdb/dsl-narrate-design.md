---
project: phi
audience: [ai-agents, contributors]
status: reference
last-updated: 2026-03-08
domain: dsl
---

# DSL narrate, decisões de design

## gramática

hibrida estruturada-verbal: elementos são keywords (substantivos), modifiers são pares chave-valor, chaves para filhos, keywords verbais para ações (`show`, `on`, `when`, `each`, `bind`).

### disambiguação no parser

| situação | regra |
|----------|-------|
| modifier vs elemento seguinte | sets disjuntos: modifier keys ≠ element keywords |
| `{expr}` vs `{children}` | estado do parser: esperando valor (após modifier key) = expr; após modifiers = block |
| flag vs value-required modifier | tabela estática em `ModifierKey::is_flag()` |
| commas entre modifiers | opcionais, consumidas silenciosamente |

### modifier categories

**flags (sem valor):** flex, center, centered, bold, italic, wrap
**value-required (todos os outros):** gap, p, px, py, pt, pb, pl, pr, m, mx, my, w, h, min_w, min_h, max_w, max_h, grow, shrink, basis, align_items, justify, bg, text_color, rounded, shadow, opacity, border, font_size

decisão: sem categoria "optional value", simplifica o parser e elimina ambiguidade `rounded div` (rounded flag? ou rounded com valor "div"?). o usuário escreve `rounded "md"` explicitamente.

### valores de modifiers

apenas literais (string, int, float) e `{expr}`. bare idents como valores não suportados, evita ambiguidade `align_items center` (onde `center` poderia ser modifier key ou valor).

## codegen

### mapeamentos

| DSL element | builder constructor |
|-------------|-------------------|
| row | `div().flex().row()` |
| col | `div().flex().col()` |
| div | `div()` |
| text | `text()` |
| button | `button()` |
| spacer | `spacer()` |
| pascalcase | `Name::view()` |

### show + format interpolation

`show "Count: {count}"` -> `.child(format!("Count: {}", count))`

interpolação implementada como scanner de `{...}` no string literal:
- `{{` = escaped `{` (literal)
- `{expr}` = interpolação rust
- nested braces trackadas por depth counter

### geração de código

output wrappado em bloco com `use ::phi_narrate::builder::*;`, paths qualificados via crate name, import isolado no bloco (sem leak para scope externo).

## stubs (temporários até task-05)

builder stubs em `phi_narrate::builder` usam generics (`<V>`) para aceitar qualquer tipo. sem validação semântica, erros de tipo serão pegos pela builder API real quando mergeada.

## crates

- `phi_narrate_macro`, proc-macro crate (syn/quote/proc-macro2)
- `phi_narrate`, re-export + stubs (sem dependência em φ por enquanto)
