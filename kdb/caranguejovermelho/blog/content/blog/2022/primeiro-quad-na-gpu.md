+++
authors = ["Brenner Cruvinel"]
title = "o primeiro quad na gpu"
description = "o primeiro retangulo com alpha blending na gpu do plev, premultiplied alpha em todo o pipeline, e os dois bugs de gamma que vieram junto."
# data provisoria. o ano da pasta (2022) sera ajustado pela timeline real depois.
date = 2022-01-01
path = "blog/primeiro-quad-na-gpu"
[taxonomies]
tags = ["building plev", "Rust", "GPU", "wgpu", "WebGPU"]
+++

quando o primeiro retangulo colorido apareceu na tela, com a borda meio transparente deixando o fundo passar, eu fiquei olhando pra aquilo por um tempo bom. um quad. quatro vertices, uma cor, um canal de alpha. e mesmo assim levei dias pra ele aparecer, e mais umas semanas pra entender que ele nunca tinha ficado certo de verdade.

esse e o primeiro post da serie building plev, onde eu conto como a engine foi construida por dentro. nao e manual de api, e a jornada com os bugs que paguei caro e os adr que sobraram deles. o plev e um compositing engine gpu-first em rust, a ideia e ser pro rust mais ou menos o que o skia e pro mundo c++. mas a parte que importa hoje e bem menor: como desenhar um unico retangulo direito.

## o quad e o atomo da engine

no fim, quase tudo que a engine desenha vira quad. um retangulo solido e um quad. um glifo de texto e um quad com uma textura de atlas por cima. uma sombra e um quad com um shader que sabe borrar a borda. entao o primeiro pipeline que escrevi foi o de quad, e ele carrega uma decisao que ficou em todo o resto: premultiplied alpha.

a ideia do premultiplied e facil de falar e chata de internalizar. em vez de guardar a cor e a transparencia separadas e multiplicar so na hora de compor, voce ja guarda a cor multiplicada pelo alpha. o blend fica associativo, camada sobre camada compoe sem halo escuro na borda, sem aquela franja cinza que aparece quando voce mistura alpha do jeito ingenuo.

o fragment shader do quad e quase nada:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Linearize, then premultiplied alpha output.
    let rgb = srgb_to_linear(in.color.rgb);
    return vec4<f32>(rgb * in.color.a, in.color.a);
}
```

duas linhas que fazem trabalho. primeiro lineariza a cor, depois multiplica pelo alpha. a ordem importa, e foi exatamente a ordem que eu errei.

## a cor entra uma vez, sai uma vez

a regra que eu devia ter sabido desde o comeco, e que virou invariante da engine: a cor e linearizada uma vez so, no ponto em que ela entra na memoria da gpu. decode uma vez na entrada, encode uma vez na saida.

o problema e que por meses eu nao fiz isso. os tokens de tema, os valores de css, os literais hex, todos vivem em sRGB. e a surface da janela no desktop e um formato sRGB, o que significa que a gpu aplica uma conversao de linear pra sRGB em toda escrita. eu entregava valor sRGB pra gpu como se fosse linear. a gpu reencodava. cada meio-tom subia mais ou menos 2.5 vezes.

o sintoma: o fundo da pagina, um #303030, valor 48, media 118 na tela. cinza lavado, sem contraste. eu reportei isso pra mim mesmo umas cinco vezes em semanas diferentes, sempre achando que era escolha de token. cheguei a escurecer os tokens pra compensar. funcionou numa tela e deixou todas as proximas cores erradas, que e a assinatura do band-aid: conserta o sintoma local, espalha a causa.

a correcao foi uma fronteira so. cor de vertice lineariza dentro do shader, o `srgb_to_linear` que ja apareceu ali em cima. clear color e uniform montado na cpu passam por `Color::to_linear_array()`, com o alpha intocado. textura que ja chega decodificada (atlas de imagem, backdrop, composite, blur) nao converte de novo, senao voce lineariza duas vezes e cai no buraco oposto.

depois disso o mesmo #303030 mediu 50 contra um token de 48. tem um teste que prende isso pra sempre, `to_linear_array_darkens_srgb_midtones`, que fixa #303030 em 0.0296. quem quiser o diff, e o commit 69013d1.

### medir o pixel, nao olhar pra tela

a licao que eu mais levo desse periodo nao e sobre gamma. e sobre validacao. eu passei semanas olhando pra tela e achando que sabia o que via. o olho e um amperimetro pessimo. o que destravou foi medir o pixel: ler o valor, 48 ou 118 ou 50, em vez de julgar "ta meio claro". se um token de 48 renderiza 118, o bug esta na transferencia, nao no token. parece obvio escrito assim. nao foi, no calor.

## o mesmo bug, espelhado, no navegador

quando o build de WebGPU comecou a rodar, o fundo veio errado de novo, so que pro outro lado. mediu (8,8,8) onde devia ser (48,48,48). escuro demais, exatamente o inverso do bug do desktop, vindo da mesma raiz.

o motivo: a api de canvas do WebGPU so aceita formato de surface nao-sRGB, bgra8unorm ou rgba8unorm. no desktop a surface e sRGB e o encode na escrita acontece sozinho. na web esse encode era silenciosamente pulado. eu escrevia o valor ja linearizado, cru, na tela, sem ninguem reencodar na saida.

a saida foi configurar a surface com o formato base mais a variante sRGB no `view_formats`, via `TextureFormat::add_srgb_suffix()`, e mandar todo render pass mirar numa view criada com o formato sRGB. tem um unico jeito sancionado de criar esse alvo, `GpuContext::surface_render_view`. um `create_view(&Default::default())` pelado herda o formato nao-sRGB da textura e pula o gamma de novo, calado. e o pior e que no desktop isso passa, porque la `add_srgb_suffix()` e identidade. o bug fica invisivel na maquina de quem revisa e catastrofico na web, o que e o tipo de defeito que sobrevive ao code review.

migrei os sete pontos de render pra `surface_render_view` e o fundo da web mediu (48,48,48), identico ao desktop. um caminho de codigo, todo target encodando igual (commit 2a33933). sinceramente, essa foi a parte que mais me deu gosto. a diferenca entre os targets fica expressa uma vez, na configuracao, e some. nada de um `if` "conserta a cor na web" espalhado pelo codigo.

## o que esse retangulo me ensinou

eu comecei achando que o trabalho dificil da engine seria a parte chique: layout, texto, animacao. o primeiro quad me ensinou que o chao tinha mais armadilha que o teto. gamma e correcao de cor sao conhecimento velho, todo motor grafico serio ja pagou esse pedagio, o flutter com o impeller, o que a zed fez no gpui. eu nao inventei nada aqui. so descobri, do meu jeito e com minhas semanas perdidas, por que o conselho de linearizar uma vez existe.

o quad continua sendo quatro vertices e uma cor. a diferenca e que agora eu sei onde a cor entra e onde ela sai, e tenho um teste que grita se alguem mexer nisso. aquele mesmo `srgb_to_linear` do shader do quad reaparece no rect_sdf, no texto, nas duas sombras. uma fronteira de cor, repetida de proposito em cada shader que toca pixel. no proximo da serie eu puxo esse fio e mostro como o texto entra nesse mesmo desenho sem virar um caso especial.
