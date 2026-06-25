+++
authors = ["Brenner Cruvinel"]
title = "o solver analitico que matou o jitter dependente de frame-rate"
description = "building plev: troquei o forward euler do spring por um solver analitico de oscilador harmonico. tres regimes, coeficientes pre-computados, e o mesmo resultado a 30, 60 e 120 FPS."
# data provisoria; o ano da pasta sera ajustado pela timeline real do livro depois
date = 2024-09-01
# path fixo (ADR 0006): a pasta do ano nao vaza na URL, da pra remanejar sem quebrar link
path = "blog/spring-solver-analitico"
[taxonomies]
tags = ["building plev", "Rust", "Animação", "Física", "Engine"]
+++

tinha uma animação no `message_dock` que parecia certa. o dock abrindo, os avatares deslizando pra dentro, tudo macio no meu mac a 60fps. ai liguei o profiler num frame mais pesado, o dt subiu, e a mesma animação ficou outra. mais lenta, com um tranco no fim. nada quebrou, nenhum panic. só que o movimento dependia de quantas vezes por segundo eu chamava o tick, e isso é bug de corretude.

## o jeito antigo era honesto sobre a própria limitação

antes do sistema de animação formal, eu animava com lerp exponencial por frame. uma função de três linhas:

```rust
fn smooth(current: f32, target: f32, speed: f32) -> f32 {
    let diff = target - current;
    if diff.abs() < 0.5 { target } else { current + diff * speed }
}
```

cada frame: `current = smooth(current, target, 0.12)`. dá um easing exponencial natural, rápido no começo, desacelerando no fim. funciona sem nenhuma infra de tempo, e foi o que segurou o `message_dock` por um tempo.

o problema mora no `speed`. 0.12 por frame só quer dizer alguma coisa se você souber quantos frames vêm por segundo. a 60fps fica bom, a 30fps o mesmo 0.12 converge na metade da velocidade. eu até deixei isso documentado na época como limitação conhecida, então não foi surpresa. foi dívida.

## o spring trocou o problema, não tirou ele

a task-27 trouxe um `Spring<T>` de verdade: massa, rigidez, amortecimento, a física que dá aquele overshoot gostoso quando o amortecimento é baixo. mas a primeira versão integrava por forward euler, somando aceleração vezes dt a cada passo. euler é simpático até você botar um spring rígido ou um dt grande. ai ele diverge. o estado a 30fps não bate com o estado a 60fps, e com stiffness alta o valor escapa pro infinito em vez de assentar no alvo.

então eu tinha o mesmo bug de antes, agora com mais casas decimais. um spring que devolve resultado diferente conforme o frame-rate vira gerador de jitter parametrizado.

## a sacada: esse spring tem solução fechada

um spring amortecido é um oscilador harmônico. a equação é a de qualquer livro de física, `m*x'' + c*x' + k*x = 0`, e essa equação tem solução analítica. eu não preciso integrar passo a passo torcendo pra não acumular erro. dá pra avaliar a solução exata no instante dt, qualquer dt.

foi o que a task-35 fez. peguei o padrão da natura (crate de spring physics em rust, o solver fechado dela é o que cataloguei na pesquisa de referência como pattern c1) e reconstrui dentro do `Spring<T>`, mantendo a API genérica sobre `f32` e `[f32; N]` e a config que vem do tema. create, never copy: a matemática é a mesma do oscilador, o resto é meu.

a primeira coisa que o `tick` faz é decidir o regime:

```rust
let omega_0_sq = self.stiffness / self.mass;
let gamma = self.damping / (2.0 * self.mass);
let disc = gamma * gamma - omega_0_sq;
```

o sinal de `disc` é o oscilador inteiro. negativo, o spring oscila antes de assentar, o sub-amortecido com overshoot. zero, ele chega no alvo o mais rápido possível sem passar, o criticamente amortecido. positivo, ele chega devagar e nunca passa, o super-amortecido. cada regime é um par de exponenciais decaindo, e cada um vira quatro coeficientes:

```rust
let (pos_pos, pos_vel, vel_pos, vel_vel) = if disc < -1e-6 {
    // sub-amortecido: exp * (cos, sin)
    let omega_d = (-disc).sqrt();
    let exp = (-gamma * dt).exp();
    let cos_wd = (omega_d * dt).cos();
    let sin_wd = (omega_d * dt).sin();
    let g_over_wd = gamma / omega_d;
    (
        exp * (cos_wd + g_over_wd * sin_wd),
        exp * sin_wd / omega_d,
        -exp * omega_0_sq * sin_wd / omega_d,
        exp * (cos_wd - g_over_wd * sin_wd),
    )
} else if disc > 1e-6 {
    // super-amortecido: duas exponenciais reais
    // ...
} else {
    // criticamente amortecido: exp * polinomio
    // ...
};
```

esses quatro números são uma matriz de transição de estado 2x2. dado o deslocamento atual (valor menos alvo) e a velocidade atual, eles dizem qual vai ser o deslocamento e a velocidade depois de dt segundos. o `tick` então deixa de ser um loop de integração e vira um multiply-add:

```rust
let new_disp = displacement.scale(pos_pos).add(&self.velocity.scale(pos_vel));
let new_vel  = displacement.scale(vel_pos).add(&self.velocity.scale(vel_vel));
self.value = self.target.add(&new_disp);
self.velocity = new_vel;
```

## por que isso mata o jitter

porque a solução fechada é exata pra qualquer dt. não tem erro de integração pra acumular. dois ticks de `1.0/30.0` e quatro ticks de `1.0/60.0` caem no mesmo lugar, porque ambos avaliam a mesma curva contínua, só que em pontos diferentes dela.

os testes cobram exatamente isso. `spring_frame_rate_independence` roda o mesmo spring a 30, 60 e 120fps e exige que os três terminem dentro de 0.5 do alvo um do outro. `spring_high_stiffness_stable` bota `stiffness` em 10000, roda 600 ticks, e checa que o valor continua finito o caminho todo. o euler antigo estouraria nesse segundo teste. o analítico assenta.

de quebra ganhei um `damping_ratio()` quase de graça, porque uma vez escrito o discriminante, o zeta `= c / (2*sqrt(k*m))` já está ali na frente. dá pra perguntar pro spring se ele é sub, crítico ou super-amortecido sem rodar um frame sequer.

## o que continua exigindo cuidado

o solver tira o jitter da matemática, mas a tela tem os dela. coordenada animada ainda passa por `round()` antes de ir pro compositor, senão a borda fica oscilando entre duas linhas de pixel. o `font_size` fica fixo durante a animação, porque escalar o tamanho da fonte muda a chave de shaping e re-shapa o glifo a cada frame. e o relógio é `web_time::Instant`, não `std::time::Instant`, que dá panic no WASM. três detalhes que não têm nada a ver com spring e tudo a ver com rodar igual no mac, no browser e no celular.

o resultado é chato de tão simples. o tick virou quatro multiplicações e duas somas com coeficientes pré-computados, e a animação que eu via diferente entre 30 e 60fps agora cai dentro de meio pixel nos dois.

## rastros

- código: `crates/engine/src/animation/spring.rs`, `tick()` em l96-160, `damping_ratio()` em l87-94, trait `SpringInterpolate` e impls em l3-38
- testes: `crates/engine/src/animation/tests_spring.rs`, `spring_frame_rate_independence` l69-105, `spring_high_stiffness_stable` l107-124
- task: `kdb/mission/steps/checked/task-35-spring-analytical-solver-done.md`
- o estado anterior: `kdb/adr/animation-pattern-lerp.md` (lerp exponencial por frame, com a limitação de frame-rate já anotada)
- regra de animação: `kdb/mission/rules.md`, seção animação (solver analítico, 3 regimes, frame-rate independent, incondicionalmente estável)
- fundação creditada: pattern c1 (natura, closed-form spring) em `kdb/adr/extracted-patterns.md`
