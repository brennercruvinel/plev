+++
authors = ["Brenner Cruvinel"]
title = "Otimização de prompts para o Hoff Health"
description = "Modelo de dados, processamento de vetores PAD e estratégia de otimização de prompts para o sistema emocional do Hoff Health."
date = 2022-08-12
[taxonomies]
tags = ["Prompt Engineering", "IA", "Hoff Health"]
+++



### Modelo de Dados

- Implementação das interfaces de EmotionalSeed
- Desenvolvimento do sistema de processamento de vetores PAD
- Criação da camada de persistência com criptografia

### Algoritmos Fundamentais

- Implementação do cálculo de similaridade emocional
- Desenvolvimento do decaimento temporal
- Criação do framework bayesiano simplificado

### Sentimentree Básica

- Estrutura de grafo para conexões emocionais
- Sistema de peso para conexões entre nós
- Algoritmo para detecção de padrões simples

## Fase 5: Interface Principal e Visualizações (Semanas 9-10)

### Dashboard Principal

- Design e implementação do dashboard home
- Integração com a Roda da Vida
- Desenvolvimento do contador para próxima sessão

### Componentes de Visualização

- Implementação dos componentes
    - Streamgraph para fluxo temporal de emoções
    - Calendário emocional
    - Radar chart para relações
    - Lollipop chart para predominâncias semanais

### Bottom Sheets e Interatividade

- Sistema de bottom sheets para detalhamento
- Interações entre visualizações diferentes
- Filtros e controles para personalização da experiência

## Fase 6: Sessão Diária Principal (Semanas 11-12)

### Interface de Sessão

- Desenvolvimento da interface minimalista para sessão de 5 minutos
- Implementação do timer regressivo
- Integração com feedback visual de emoções

### Extração e Processamento

- Sistema para extração de dados emocionais da Hoff
- Processamento em background para criar Seeds
- Atualização das visualizações com novos dados

### Microbriefing

- Algoritmo para geração de insights relevantes
- Desenvolvimento da interface de plano de ação
- Sistema de acompanhamento de progresso


### Otimização de Performance

- Melhoria do uso da API da Hume para redução de custos
- Otimização de renderização de visualizações
- Ajustes para performance em dispositivos mais antigos

### Polimento da UX

- Refinamento de animações e transições
- Consistência visual em todos os componentes
- Ajustes de acessibilidade

## 

## Sistema de Triagem de Neurodivergências

## Abordagem Geral

O sistema de triagem de neurodivergências do Hoff Health utiliza análise longitudinal de padrões emocionais e comportamentais detectados via API da Hoff, oferecendo uma abordagem não-diagnóstica e não-estigmatizante.

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
- "Explorar com um profissional..." em v


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

## Implementação Firebase e Segurança Zero-Knowledge

##Apenas exemplos bem basico de  Arquitetura do Banco de Dados

### Coleções do dados de exmeplos basiuco 

### Users

```jsx
{
  uid: string,                 // ID de autenticação do Firebase
  displayName: string,         // Nome de exibição
  email: string,               // Email (criptografado)
  createdAt: timestamp,        // Data de criação
  lastActive: timestamp,       // Último acesso
  streak: number,              // Dias consecutivos de uso
  totalSessions: number,       // Total de sessões
  publicKey: string,           // Chave pública para criptografia
  settings: {                  // Configurações do usuário
    notifications: boolean,    // Notificações ativadas
    theme: 'light' | 'dark',   // Tema preferido
    language: string,          // Idioma preferido
    privacyLevel: number       // Nível de privacidade (1-3)
  },
  metadata: {                  // Metadados não sensíveis
    deviceType: string,        // Tipo de dispositivo
    timezone: string,          // Fuso horário
    appVersion: string         // Versão do app
  }
}

```

### Sessions

```jsx
{
  id: string,                  // ID da sessão
  userId: string,              // ID do usuário
  startTime: timestamp,        // Início da sessão
  endTime: timestamp,          // Fim da sessão
  duration: number,            // Duração em segundos
  transcriptHash: string,      // Hash da transcrição (verificação)
  metadata: {                  // Metadados não sensíveis
    deviceType: string,        // Tipo de dispositivo
    connectionQuality: string, // Qualidade da conexão
    completionType: string     // Como a sessão terminou
  },
  // Dados encriptados no dispositivo
  encryptedData: {
    iv: string,                // Vetor de inicialização
    data: string,              // Dados encriptados (JSON em string)
    version: number,           // Versão do algoritmo
    authTag: string            // Tag de autenticação
  }
}

```

### EmotionalData

```jsx
{
  id: string,                  // ID dos dados emocionais
  userId: string,              // ID do usuário
  sessionId: string,           // ID da sessão (opcional)
  timestamp: timestamp,        // Momento da coleta
  dataType: string,            // Tipo de dado (seed, agregação, etc)
  // Dados encriptados no dispositivo
  encryptedData: {
    iv: string,                // Vetor de inicialização
    data: string,              // Dados encriptados (JSON em string)
    version: number,           // Versão do algoritmo
    authTag: string            // Tag de autenticação
  }
}

```

### Metrics (Dados Anônimos para Analytics)

```jsx
{
  id: string,                  // ID da métrica
  timestamp: timestamp,        // Momento da coleta
  sessionCount: number,        // Número de sessões (anônimo)
  avgDuration: number,         // Duração média (anônimo)
  userCount: number,           // Contagem de usuários (anônimo)
  retentionRate: number,       // Taxa de retenção (anônimo)
  errorRates: {                // Taxas de erro (anônimo)
    apiErrors: number,
    clientErrors: number,
    connectionErrors: number
  }
}

```

## Arquitetura de Segurança Zero-Knowledge

### Implementação de Criptografia Client-Side

### Geração de Chaves

```jsx
// Pseudocódigo para geração de chaves
const generateKeys = async () => {
  // Gera um par de chaves RSA para criptografia assimétrica
  const keyPair = await window.crypto.subtle.generateKey(
    {
      name: "RSA-OAEP",
      modulusLength: 4096,
      publicExponent: new Uint8Array([1, 0, 1]),
      hash: "SHA-256",
    },
    true,
    ["encrypt", "decrypt"]
  );

  // Gera uma chave AES para criptografia simétrica rápida de dados
  const aesKey = await window.crypto.subtle.generateKey(
    {
      name: "AES-GCM",
      length: 256,
    },
    true,
    ["encrypt", "decrypt"]
  );

  // Exporta as chaves para armazenamento seguro
  const publicKeyBuffer = await window.crypto.subtle.exportKey(
    "spki",
    keyPair.publicKey
  );

  // A chave pública pode ser compartilhada e armazenada no servidor
  const publicKeyString = arrayBufferToBase64(publicKeyBuffer);

  // Chaves privadas nunca saem do dispositivo
  // São armazenadas no SecureStore/Keychain/Keystore
  await secureStore.set('privateKey', keyPair.privateKey);
  await secureStore.set('aesKey', aesKey);

  return {
    publicKey: publicKeyString
  };
};

```

### Criptografia de Dados

```jsx
// Pseudocódigo para criptografia de dados
const encryptData = async (data) => {
  // Recupera a chave AES do armazenamento seguro
  const aesKey = await secureStore.get('aesKey');

  // Gera um vetor de inicialização aleatório para cada operação
  const iv = window.crypto.getRandomValues(new Uint8Array(12));

  // Converte os dados para formato adequado para criptografia
  const encodedData = new TextEncoder().encode(JSON.stringify(data));

  // Criptografa os dados com AES-GCM
  const encryptedBuffer = await window.crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv,
      tagLength: 128
    },
    aesKey,
    encodedData
  );

  // Formata para armazenamento
  const result = {
    iv: arrayBufferToBase64(iv),
    data: arrayBufferToBase64(encryptedBuffer),
    version: 1,
    authTag: "" // O authTag está incluído no final do buffer encriptado para AES-GCM
  };

  return result;
};

```

### Descriptografia de Dados

```jsx
// Pseudocódigo para descriptografia de dados
const decryptData = async (encryptedData) => {
  // Recupera a chave AES do armazenamento seguro
  const aesKey = await secureStore.get('aesKey');

  // Converte dados de base64 para ArrayBuffer
  const iv = base64ToArrayBuffer(encryptedData.iv);
  const data = base64ToArrayBuffer(encryptedData.data);

  // Descriptografa os dados
  try {
    const decryptedBuffer = await window.crypto.subtle.decrypt(
      {
        name: "AES-GCM",
        iv,
        tagLength: 128
      },
      aesKey,
      data
    );

    // Converte o resultado para um objeto JSON
    const decodedData = new TextDecoder().decode(decryptedBuffer);
    const jsonData = JSON.parse(decodedData);

    return jsonData;
  } catch (error) {
    console.error("Erro ao descriptografar: Dados possivelmente corrompidos ou chave incorreta");
    throw new Error("Erro na descriptografia");
  }
};

```

### Sistema de Recuperação Segura

### Backup Seguro de Chaves

```jsx
// Pseudocódigo para backup seguro de chaves
const createSecureBackup = async (password) => {
  // Recupera as chaves do armazenamento seguro
  const privateKey = await secureStore.get('privateKey');
  const aesKey = await secureStore.get('aesKey');

  // Exporta as chaves para formato raw
  const privateKeyBuffer = await window.crypto.subtle.exportKey(
    "pkcs8",
    privateKey
  );
  const aesKeyBuffer = await window.crypto.subtle.exportKey(
    "raw",
    aesKey
  );

  // Combina as chaves em um único objeto
  const keysBundle = {
    privateKey: arrayBufferToBase64(privateKeyBuffer),
    aesKey: arrayBufferToBase64(aesKeyBuffer),
    version: 1,
    timestamp: Date.now()
  };

  // Deriva uma chave de criptografia a partir da senha do usuário
  const passwordKey = await deriveKeyFromPassword(password);

  // Criptografa o bundle de chaves com a chave derivada da senha
  const iv = window.crypto.getRandomValues(new Uint8Array(12));
  const encodedBundle = new TextEncoder().encode(JSON.stringify(keysBundle));

  const encryptedBundle = await window.crypto.subtle.encrypt(
    {
      name: "AES-GCM",
      iv,
      tagLength: 128
    },
    passwordKey,
    encodedBundle
  );

  // Formata para armazenamento
  const backupData = {
    iv: arrayBufferToBase64(iv),
    data: arrayBufferToBase64(encryptedBundle),
    salt: arrayBufferToBase64(passwordSalt), // Sal usado na derivação da chave
    version: 1
  };

  return backupData;
};

```

### Recuperação por Biometria Multi-fator

```jsx
// Pseudocódigo para recuperação de chaves com biometria
const recoverKeysWithBiometrics = async (encryptedBackup) => {
  // Verificação biométrica (Face ID, Touch ID, etc)
  const biometricResult = await authenticateWithBiometrics("Recuperar suas chaves");

  if (!biometricResult.success) {
    throw new Error("Falha na autenticação biométrica");
  }

  // Se a autenticação biométrica for bem-sucedida, solicitar senha
  const password = await promptSecurePassword();

  // Deriva a mesma chave de criptografia da senha
  const salt = base64ToArrayBuffer(encryptedBackup.salt);
  const passwordKey = await deriveKeyFromPassword(password, salt);

  // Descriptografa o bundle de chaves
  const iv = base64ToArrayBuffer(encryptedBackup.iv);
  const data = base64ToArrayBuffer(encryptedBackup.data);

  try {
    const decryptedBuffer = await window.crypto.subtle.decrypt(
      {
        name: "AES-GCM",
        iv,
        tagLength: 128
      },
      passwordKey,
      data
    );

    // Converte o resultado para objeto JSON
    const decodedBundle = new TextDecoder().decode(decryptedBuffer);
    const keysBundle = JSON.parse(decodedBundle);

    // Importa as chaves restauradas
    const privateKey = await window.crypto.subtle.importKey(
      "pkcs8",
      base64ToArrayBuffer(keysBundle.privateKey),
      {
        name: "RSA-OAEP",
        hash: "SHA-256"
      },
      true,
      ["decrypt"]
    );

    const aesKey = await window.crypto.subtle.importKey(
      "raw",
      base64ToArrayBuffer(keysBundle.aesKey),
      {
        name: "AES-GCM",
        length: 256
      },
      true,
      ["encrypt", "decrypt"]
    );

    // Armazena as chaves restauradas no armazenamento seguro
    await secureStore.set('privateKey', privateKey);
    await secureStore.set('aesKey', aesKey);

    return true;
  } catch (error) {
    console.error("Erro ao recuperar chaves", error);
    throw new Error("Falha na recuperação de chaves");
  }
};

```

## Regras de Segurança do 

```jsx
// Regras do Firestore para segurança
rules_version = '2';
service cloud.datastore {
  match /databases/{database}/documents {
    // Função de validação de usuário autenticado
    function isAuthenticated() {
      return request.auth != null;
    }

    // Função para verificar se o usuário é dono do recurso
    function isOwner(userId) {
      return request.auth.uid == userId;
    }

    // Função para verificar campos obrigatórios na criptografia
    function hasValidEncryption(data) {
      return data.encryptedData.iv != null &&
             data.encryptedData.data != null &&
             data.encryptedData.version != null;
    }

    // Regras para coleção de usuários
    match /users/{userId} {
      allow read: if isAuthenticated() && isOwner(userId);
      allow create: if isAuthenticated() && isOwner(userId);
      allow update: if isAuthenticated() && isOwner(userId);
      allow delete: if false; // Não permitir exclusão de contas
    }

    // Regras para sessões
    match /sessions/{sessionId} {
      allow read: if isAuthenticated() && isOwner(resource.data.userId);
      allow create: if isAuthenticated() && isOwner(request.resource.data.userId) &&
                     hasValidEncryption(request.resource.data);
      allow update: if isAuthenticated() && isOwner(resource.data.userId) &&
                     hasValidEncryption(request.resource.data);
      allow delete: if isAuthenticated() && isOwner(resource.data.userId);
    }

    // Regras para dados emocionais
    match /emotionalData/{dataId} {
      allow read: if isAuthenticated() && isOwner(resource.data.userId);
      allow create: if isAuthenticated() && isOwner(request.resource.data.userId) &&
                     hasValidEncryption(request.resource.data);
      allow update: if isAuthenticated() && isOwner(resource.data.userId) &&
                     hasValidEncryption(request.resource.data);
      allow delete: if isAuthenticated() && isOwner(resource.data.userId);
    }

    // Regras para métricas anônimas
    match /metrics/{metricId} {
      allow read: if false; // Somente acesso pelo backend
      allow write: if false; // Somente acesso pelo backend
    }
  }
}

```

## Cloud Functions para Análise Segura

```jsx
// Pseudocódigo de Cloud Function para processamento seguro
exports.processAnonymizedMetrics = functions.datastore
  .document('sessions/{sessionId}')
  .onCreate(async (snapshot, context) => {
    // Extrai apenas metadados não sensíveis para análise
    const session = snapshot.data();
    const { userId, startTime, endTime, duration, metadata } = session;

    // Não acessa dados criptografados, apenas metadados

    // Atualiza métricas anônimas agregadas
    const metricsRef = admin.datastore().collection('metrics').doc('daily');
    await metricsRef.set({
      sessionCount: admin.datastore.FieldValue.increment(1),
      totalDuration: admin.datastore.FieldValue.increment(duration),
      // Outros campos agregados...
    }, { merge: true });

    // Opcionalmente, atualiza streak do usuário sem acessar dados sensíveis
    const userRef = admin.datastore().collection('users').doc(userId);
    await userRef.update({
      streak: admin.datastore.FieldValue.increment(1),
      totalSessions: admin.datastore.FieldValue.increment(1),
      lastActive: admin.datastore.Timestamp.now()
    });

    return null;
  });

```

## Monitoramento e Auditoria

```jsx
// Pseudocódigo para monitoramento de segurança
exports.monitorSecurityEvents = functions.datastore
  .document('users/{userId}')
  .onUpdate(async (change, context) => {
    const before = change.before.data();
    const after = change.after.data();
    const userId = context.params.userId;

    // Verifica mudanças críticas que podem indicar comprometimento
    if (before.publicKey !== after.publicKey) {
      // Registra evento de mudança de chave pública
      await admin.datastore().collection('securityEvents').add({
        userId,
        eventType: 'PUBLIC_KEY_CHANGE',
        timestamp: admin.datastore.Timestamp.now(),
        metadata: {
          ip: context.ip,
          userAgent: context.userAgent
        }
      });

      // Notifica usuário sobre mudança de chave
      await sendSecurityAlert(userId, 'PUBLIC_KEY_CHANGE');
    }

    return null;
  });

```

## Considerações Sobre Recuperação de Conta

Para implementar um sistema de recuperação de conta que mantenha a arquitetura zero-knowledge, é preciso considerar:

1. **Múltiplos Fatores de Recuperação**:
    - Senha forte + biometria + código de recuperação
    - Fragmentação de chave (Shamir Secret Sharing)
    - Backup em dispositivo secundário confiável
2. **Trade-offs de Segurança vs. Usabilidade**:
    - Recuperação 100% segura pode significar impossibilidade de recuperação se todos os fatores forem perdidos
    - Uma opção é permitir recuperação parcial (dados recentes) com menos fatores
3. **Implementação de Melhor Esforço**:
    - Backup criptografado de chaves no dispositivo
    - Sincronização segura entre dispositivos autorizados
    - Conjunto limitado de dados recuperáveis sem todas as chaves

A estratégia recomendada equilibra segurança com usabilidade, priorizando a proteção dos dados mais sensíveis enquanto oferece opções de recuperação para o caso de perda de dispositivo ou chaves.
