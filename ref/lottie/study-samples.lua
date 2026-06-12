return {
  id  = "stu-lottie-samples",
  typ = "study",
  sts = "ativo",
  dom = "anim",
  dat = "2026-06-11",
  ttl = "amostras lottie/dotlottie/webm reais: baselines bytes/s e o modelo de interatividade declarativa a superar",
  lnk = { "ref-study", "monster-formato" },
  txt = [=[
diretiva do brenner: nao e para entender lottie, e para fazer a NOSSA
engine. leitura obrigatoria antes de tocar no anm ou em demo de
animacao; duplicar capacidade existente do engine repete o defeito
mais caro do projeto (kdb/how-to/code-against-the-plev-engine.md).

o que extrair:
1. interatividade declarativa (MONEY/interactivebototnbar, 36.8KB):
   dotlottie v2 embarca state machines (s/*.json, 9.2KB + 13.5KB cru,
   ~4KB comprimido): inputs tipados (Event, String com default),
   estados PlaybackState (segment nomeado + autoplay + speed) e
   GlobalState, transicoes com guards (Equal sobre inputs),
   interactions Click/PointerDown amarradas a NOME DE LAYER (a propria
   layer e a hit area) e OnComplete encadeando segmentos. markers do
   json (cm/tm/dr) = segmentos nomeados da timeline. comportamento de
   jogo num button bar por ~22KB de sm. licao para o anm: tier 1 de
   interatividade = track declarativa de state machine sobre NodeId +
   hit regions reais (render_interactive) + signal/; rhai e tier 2,
   so quando a sm nao bastar. o seek O(1) dos nossos keyframes torna
   segment/goto trivial (no swf isso custava replay desde o frame 1).
2. baselines medidos (bytes por segundo de animacao; medir, nunca
   estimar):
   - explosion 1.7s: json 411KB/s, gzip9 122KB/s, .lottie 128KB/s,
     webm 22.7KB/s (efeito denso: video esmaga vetor 5.6x)
   - girl 1.58s: json 469KB/s, .lottie 69.2KB/s, webm 70.3KB/s
     (vetor flat empata com video e ainda escala e scruba)
   - swf (ref/anim/swf-samples, estudo stu-ruffle): arquivo completo
     7.5-16KB/s, delta puro de movimento ~1.7KB/s. o alvo do anm e a
     faixa do swf, com keyframes O(1) que o swf nao tinha
   - cards/motions/bar/01.lottie: componente animado inteiro em 7.8KB
3. licao do "advanced optimizer" da lottiefiles: 0% (explosion) e
   2.5% (girl) de reducao pre-compressao; o ganho do .lottie e so
   deflate do zip. otimizacao de verdade acontece no encoder (modo B:
   delta descoberto entre snapshots, reducao de keyframes por
   tolerancia, colapso de track estatica, exclude-defaults via
   presence flags), nunca como pos-processamento do texto.

o que NAO copiar: o json do after effects (inchado, expressions), o
player deles, qualquer dependencia lottie dentro do engine. webm fica
como baseline honesto na suite de benchmark e como hipotese de asset
hibrido para efeitos densos.
consome: ws-anim
]=],
}
