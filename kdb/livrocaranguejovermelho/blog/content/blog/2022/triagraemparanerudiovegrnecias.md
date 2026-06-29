+++
authors = ["Brenner Cruvinel"]
title = "Sistema de triagem de neurodivergências do Hoff Health"
description = "Princípios e arquitetura de um sistema de triagem de neurodivergências baseado em correlação com padrões, não diagnóstico, para o Hoff Health."
date = 2022-05-10
[taxonomies]
tags = ["Neurodivergência", "Saúde Digital", "Hoff Health"]
+++


## Princípios Fundamentais

- **Não-diagnóstico**: Apresentação como "correlação com padrões" em vez de diagnóstico
- **Baseado em dados**: Utiliza múltiplos pontos de dados coletados ao longo do tempo
- **Threshold conservador**: Alta especificidade para minimizar falsos positivos
- **Abordagem capacitadora**: Foco em forças e estratégias, não em déficits
- **Linguagem inclusiva**: Termos como "padrões", "características" ou "tendências"

## Perfis de Triagem

### TDAH (Transtorno de Déficit de Atenção e Hiperatividade)

#### Marcadores Emocionais e Comportamentais

- Flutuações rápidas de estados emocionais
- Hiperfoco em tópicos de interesse seguido de desinteresse
- Interrupções frequentes do próprio fluxo de pensamento
- Expressão de frustração relacionada à organização e conclusão de tarefas
- Padrões de fala mais rápidos e animados em tópicos de interesse

#### Algoritmo de Detecção

1. Monitoramento de variabilidade emocional (desvio padrão de estados emocionais)
2. Análise de prosódia para detectar mudanças rápidas de tópico
3. Identificação de padrões de hiperconcentração vs. distração
4. Correlação com menções a esquecimentos, atrasos ou dificuldade em concluir tarefas

#### Estratégias de Suporte

- Micro-intervenções para gestão de tempo e organização
- Técnicas de mindfulness adaptadas para TDAH
- Abordagens para capitalizar em períodos de hiperfoco
- Estratégias para reduzir estímulos digitais dispersivos

### TEA (Transtorno do Espectro Autista)

#### Marcadores Emocionais e Comportamentais

- Dificuldade em expressar nuances emocionais (menor variabilidade de prosódia)
- Reações intensas a situações específicas (sobrecarga sensorial)
- Preferência por rotinas e previsibilidade
- Interesses específicos com profundo conhecimento
- Padrões de fala mais analíticos e detalhados

#### Algoritmo de Detecção

1. Análise da complexidade emocional expressa (variedade de emoções mencionadas)
2. Medição de reatividade emocional a mudanças ou eventos imprevistos
3. Detecção de padrões de fala sistemáticos e estruturados
4. Identificação de mencões a desafios em contextos sociais específicos

#### Estratégias de Suporte

- Ferramentas para gestão de energia social
- Técnicas para lidar com sobrecarga sensorial
- Abordagens para capitalizar em interesses específicos
- Estratégias para criar previsibilidade em ambiente digital

### Ansiedade

#### Marcadores Emocionais e Comportamentais

- Preocupação constante com eventos futuros
- Ruminação sobre interações passadas
- Busca de reasseguramento frequente
- Padrões de evitação de situações específicas
- Prosódia com tensão vocal e respiração alterada

#### Algoritmo de Detecção

1. Monitoramento de keywords relacionadas a preocupação e ruminiação
2. Análise de prosódia para detectar tensão vocal e alterações respiratórias
3. Medição de proporção de conteúdo negativo antecipado vs. experiências positivas
4. Identificação de padrões cíclicos de preocupação-alívio temporário-preocupação

#### Estratégias de Suporte

- Técnicas de respiração e grounding para momentos de ansiedade
- Journaling estruturado para desafiar preocupações excessivas
- Limites saudáveis para consumo de notícias e mídias sociais
- Exposição gradual a situações evitadas

### Depressão

#### Marcadores Emocionais e Comportamentais

- Persistência de emoções negativas (tristeza, vazio, desesperança)
- Baixa variabilidade emocional (afeto embotado)
- Diminuição de interesse em atividades anteriormente prazerosas
- Alterações em padrões de sono e energia
- Prosódia com tom monótono, volume baixo, ritmo lento

#### Algoritmo de Detecção

1. Monitoramento da proporção de emoções negativas vs. positivas ao longo do tempo
2. Análise de prosódia para detectar monotonia vocal e energia reduzida
3. Identificação de menções a dificuldades com sono, energia e motivação
4. Cálculo da trajetória de variabilidade emocional (declínio significativo)

#### Estratégias de Suporte

- Micro-atividades comportamentais para reativação
- Técnicas para interromper espirais negativas de pensamento
- Estratégias para manter conexão social mínima
- Sugestões para estruturação de rotinas básicas

### TOC (Transtorno Obsessivo-Compulsivo)

#### Marcadores Emocionais e Comportamentais

- Padrões recorrentes de pensamentos intrusivos
- Comportamentos repetitivos para aliviar ansiedade
- Rigidez em rotinas específicas
- Expressão de culpa ou vergonha relacionada a pensamentos
- Prosódia com tensão e alívio cíclicos

#### Algoritmo de Detecção

1. Identificação de menções a pensamentos intrusivos e desconforto associado
2. Detecção de padrões de verificação ou comportamentos repetitivos
3. Análise de ciclos de ansiedade-alívio temporário-retorno de ansiedade
4. Menções a perfeccionismo extremo ou preocupação com erros

#### Estratégias de Suporte

- Técnicas de conscientização e aceitação de pensamentos
- Estratégias de exposição e prevenção de resposta simplificadas
- Abordagens para reduzir verificação digital
- Ferramentas para flexibilidade cognitiva

## Sistema de Pontuação Probabilística

### Metodologia de Cálculo

- **Base de pontuação**: 0-100 para cada perfil
- **Threshold para recomendação**: 75+ pontos (alta especificidade)
- **Período mínimo**: 2 semanas de dados (mínimo 5 sessões)
- **Pesos adaptativos**: Maior peso para marcadores mais recentes

### Fatores de Pontuação

- Frequência de marcadores específicos (40%)
- Intensidade de marcadores quando presentes (30%)
- Consistência ao longo do tempo (20%)
- Impacto auto-reportado na vida (10%)

### Ajustes de Confiabilidade

- Aumento de confiabilidade com mais sessões
- Redução de pontuação para padrões inconsistentes
- Ajuste para expressões situacionais vs. traços estáveis
- Correlação cruzada entre diferentes perfis (comorbidades)

## Interface de Apresentação

### Elementos Visuais

- Gráficos de radar para diferentes dimensões
- Visualização de timeline para evolução de padrões
- Código de cores neutro (não clínico)
- Ênfase visual em forças e estratégias, não em déficits

### Linguagem Recomendada

- "Padrões consistentes com..." em vez de "indicadores de..."
- "Estratégias que podem ser úteis..." em vez de "tratamentos para..."
- "Características comuns em pessoas com..." em vez de "sintomas de..."
- "Explorar com um profissional..." em vez de "buscar diagnóstico para..."

### Próximos Passos Sugeridos

- Recursos educacionais específicos
- Comunidades de apoio (quando aplicável)
- Ferramentas de autogestão dentro do app
- Opções para buscar avaliação profissional quando indicado

## Ética e Privacidade

### Considerações Éticas

- Explicação clara de limitações da triagem
- Evitar uso de termos diagnósticos em notificações e interfaces
- Revisão periódica dos algoritmos para prevenir vieses
- Monitoramento de impacto emocional da apresentação de resultados

### Proteções de Privacidade

- Criptografia de dados de triagem em repouso
- Opção de excluir dados de triagem separadamente
- Consentimento específico para coleta e análise
- Transparência sobre como os dados são utilizados

## Implementação de Detecção

### Abordagem Integrada

A triagem de neurodivergências do Hoff Health utiliza análise longitudinal de padrões emocionais e comportamentais detectados via API da Hoff, oferecendo uma abordagem não-diagnóstica e não-estigmatizante.

A detecção acontece durante conversas regulares, sem questionários diretos ou interrupções.

### Dados Utilizados

- Padrões de variabilidade emocional ao longo do tempo
- Análise de prosódia (tom, ritmo, energia)
- Menções contextuais e situacionais
- Padrões de mudança rápida de tópicos
- Expressões de dificuldade ou frustração
- Padrões de interesse intenso e persistente

### Output de Resultados

Quando padrões significativos são identificados:

1. Notificação discreta ao usuário (sem alarmes)
2. Apresentação de características observadas (sem diagnóstico)
3. Sugestão de estratégias aplicáveis dentro do app
4. Recurso opcional: sugestão de avaliação profissional
5. Rastreamento de progressão de padrões ao longo do tempo

## Integração com Sessões Diárias

A triagem acontece naturalmente durante as sessões de 5 minutos do Hoff, sem adicionar carga ou ansiedade ao usuário. Os dados coletados alimentam continuamente o sistema de pontuação probabilística, criando um perfil cada vez mais preciso ao longo das semanas.

O microbriefing final de cada sessão pode incluir insights sobre padrões neurodivergentes observados, sempre com linguagem capacitadora e não-estigmatizante.



### Princípios Fundamentais

- **Não-diagnóstico**: Apresentação como "correlação com padrões" em vez de diagnóstico
- **Baseado em dados**: Utiliza múltiplos pontos de dados coletados ao longo do tempo
- **Threshold conservador**: Alta especificidade para minimizar falsos positivos
- **Abordagem capacitadora**: Foco em forças e estratégias, não em déficits
- **Linguagem inclusiva**: Termos como "padrões", "características" ou "tendências"

## Perfis de Triagem

### TDAH (Transtorno de Déficit de Atenção e Hiperatividade)

### Marcadores Emocionais e Comportamentais

- Flutuações rápidas de estados emocionais
- Hiperfoco em tópicos de interesse seguido de desinteresse
- Interrupções frequentes do próprio fluxo de pensamento
- Expressão de frustração relacionada à organização e conclusão de tarefas
- Padrões de fala mais rápidos e animados em tópicos de interesse

### Algoritmo de Detecção

1. Monitoramento de variabilidade emocional (desvio padrão de estados emocionais)
2. Análise de prosódia para detectar mudanças rápidas de tópico
3. Identificação de padrões de hiperconcentração vs. distração
4. Correlação com menções a esquecimentos, atrasos ou dificuldade em concluir tarefas

### Estratégias de Suporte

- Micro-intervenções para gestão de tempo e organização
- Técnicas de mindfulness adaptadas para TDAH
- Abordagens para capitalizar em períodos de hiperfoco
- Estratégias para reduzir estímulos digitais dispersivos

### TEA (Transtorno do Espectro Autista)

### Marcadores Emocionais e Comportamentais

- Dificuldade em expressar nuances emocionais (menor variabilidade de prosódia)
- Reações intensas a situações específicas (sobrecarga sensorial)
- Preferência por rotinas e previsibilidade
- Interesses específicos com profundo conhecimento
- Padrões de fala mais analíticos e detalhados

### Algoritmo de Detecção

1. Análise da complexidade emocional expressa (variedade de emoções mencionadas)
2. Medição de reatividade emocional a mudanças ou eventos imprevistos
3. Detecção de padrões de fala sistemáticos e estruturados
4. Identificação de mencões a desafios em contextos sociais específicos

### Estratégias de Suporte

- Ferramentas para gestão de energia social
- Técnicas para lidar com sobrecarga sensorial
- Abordagens para capitalizar em interesses específicos
- Estratégias para criar previsibilidade em ambiente digital

### Ansiedade

### Marcadores Emocionais e Comportamentais

- Preocupação constante com eventos futuros
- Ruminação sobre interações passadas
- Busca de reasseguramento frequente
- Padrões de evitação de situações específicas
- Prosódia com tensão vocal e respiração alterada

### Algoritmo de Detecção

1. Monitoramento de keywords relacionadas a preocupação e ruminiação
2. Análise de prosódia para detectar tensão vocal e alterações respiratórias
3. Medição de proporção de conteúdo negativo antecipado vs. experiências positivas
4. Identificação de padrões cíclicos de preocupação-alívio temporário-preocupação

### Estratégias de Suporte

- Técnicas de respiração e grounding para momentos de ansiedade
- Journaling estruturado para desafiar preocupações excessivas
- Limites saudáveis para consumo de notícias e mídias sociais
- Exposição gradual a situações evitadas

### Depressão

### Marcadores Emocionais e Comportamentais

- Persistência de emoções negativas (tristeza, vazio, desesperança)
- Baixa variabilidade emocional (afeto embotado)
- Diminuição de interesse em atividades anteriormente prazerosas
- Alterações em padrões de sono e energia
- Prosódia com tom monótono, volume baixo, ritmo lento

### Algoritmo de Detecção

1. Monitoramento da proporção de emoções negativas vs. positivas ao longo do tempo
2. Análise de prosódia para detectar monotonia vocal e energia reduzida
3. Identificação de menções a dificuldades com sono, energia e motivação
4. Cálculo da trajetória de variabilidade emocional (declínio significativo)

### Estratégias de Suporte

- Micro-atividades comportamentais para reativação
- Técnicas para interromper espirais negativas de pensamento
- Estratégias para manter conexão social mínima
- Sugestões para estruturação de rotinas básicas

### TOC (Transtorno Obsessivo-Compulsivo)

### Marcadores Emocionais e Comportamentais

- Padrões recorrentes de pensamentos intrusivos
- Comportamentos repetitivos para aliviar ansiedade
- Rigidez em rotinas específicas
- Expressão de culpa ou vergonha relacionada a pensamentos
- Prosódia com tensão e alívio cíclicos

### Algoritmo de Detecção

1. Identificação de menções a pensamentos intrusivos e desconforto associado
2. Detecção de padrões de verificação ou comportamentos repetitivos
3. Análise de ciclos de ansiedade-alívio temporário-retorno de ansiedade
4. Menções a perfeccionismo extremo ou preocupação com erros

### Estratégias de Suporte

- Técnicas de conscientização e aceitação de pensamentos
- Estratégias de exposição e prevenção de resposta simplificadas
- Abordagens para reduzir verificação digital
- Ferramentas para flexibilidade cognitiva

## Sistema de Pontuação Probabilística

### Metodologia de Cálculo

- **Base de pontuação**: 0-100 para cada perfil
- **Threshold para recomendação**: 75+ pontos (alta especificidade)
- **Período mínimo**: 2 semanas de dados (mínimo 5 sessões)
- **Pesos adaptativos**: Maior peso para marcadores mais recentes

### Fatores de Pontuação

- Frequência de marcadores específicos (40%)
- Intensidade de marcadores quando presentes (30%)
- Consistência ao longo do tempo (20%)
- Impacto auto-reportado na vida (10%)

### Ajustes de Confiabilidade

- Aumento de confiabilidade com mais sessões
- Redução de pontuação para padrões inconsistentes
- Ajuste para expressões situacionais vs. traços estáveis
- Correlação cruzada entre diferentes perfis (comorbidades)

## Interface de Apresentação

### Elementos Visuais

- Gráficos de radar para diferentes dimensões
- Visualização de timeline para evolução de padrões
- Código de cores neutro (não clínico)
- Ênfase visual em forças e estratégias, não em déficits

### Linguagem Recomendada

- "Padrões consistentes com..." em vez de "indicadores de..."
- "Estratégias que podem ser úteis..." em vez de "tratamentos para..."
- "Características comuns em pessoas com..." em vez de "sintomas de..."
- "Explorar com um profissional..." em vez de "buscar diagnóstico para..."

### Próximos Passos Sugeridos

- Recursos educacionais específicos
- Comunidades de apoio (quando aplicável)
- Ferramentas de autogestão dentro do app
- Opções para buscar avaliação profissional quando indicado

## Ética e Privacidade

### Considerações Éticas

- Explicação clara de limitações da triagem
- Evitar uso de termos diagnósticos em notificações e interfaces
- Revisão periódica dos algoritmos para prevenir vieses
- Monitoramento de impacto emocional da apresentação de resultados

### Proteções de Privacidade

- Criptografia de dados de triagem em repouso
- Opção de excluir dados de triagem separadamente
- Consentimento específico para coleta e análise
- Transparência sobre como os dados são utilizados
