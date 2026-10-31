---
id: task-63/fase-c
title: "task-63 fase c: arquivo de sessão, persistência e trecho"
project: plev
type: secao
parent: task-63
status: referencia
date: 2026-09-23
date_basis: texto
topics: [gravacao, formatos]
relations: [adr-0037, task-61, task-65, task-66b]
evidence: [c62a05d]
source: docs/log/steps/task-63-flight-recorder.md
source_revision: ec0a224c
revised: 2026-10-30
review: revisado
---

# fase c fechada (2026-09-23)

Registra a persistência da sessão em arquivo: o formato `MONS`, com cabeçalho de identidade do build, seções `S`, `I`, `C` e `A` com sha256 por seção, e um leitor estrito que devolve erro tipado em arquivo malformado. Traz `Recorder::persist`, `export` de um trecho que começa no keyframe, `install_panic_hook` e a recusa de sessão no `monster_player`. Integrada na `main` em `c62a05d` com gate verde de 1855 testes; medido, uma janela de 20,6 s deu um arquivo de 76 KB.

integrada na `main` em `c62a05d` (seis commits), rebaseada sobre a 60 e a
58b, gate verde num diretório isolado (1855 testes). arquivo de sessão
`.monster` com magic `MONS`: cabeçalho com identidade de build e seções
`S` (JSON lines), `I`, `C` (os bytes do `monster::encode`, verbatim) e `A`,
sha256 por seção; `Recorder::persist` e `export` (trecho a partir do
keyframe que governa `t0`); `Session::open` estrito (erro tipado em todo
arquivo malformado, 2000 danos aleatórios sem panic; animação recusada
pelo nome); `install_panic_hook`; `monster_player` recusa sessão com
mensagem. medido: a janela de 20,6 s do todo dá um arquivo de 76 KB
(3,7 KB/s), escrito em 1 a 1,5 ms e lido em 0,5 ms.

o agente corrigiu uma premissa do orquestrador: o acervo de imagens não
guarda pixels depois do primeiro frame (o `pixels` de `image.rs:65` é do
upload pendente, drenado pela GPU). decisão do orquestrador: retenção de
pixels opt-in, ligada pelo gravador, compartilhando o mesmo `Arc` com o
upload (sem cópia dupla; o perfil de memória não muda quando não se
grava). isso e a ligação no shell do showcase (atalho de debug e panic
hook) são a fase d, que entra depois da task-61 porque as duas mexem no
shell.
