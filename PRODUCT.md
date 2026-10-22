# Mirante — produto

**Status:** MVP de referência implementado. Marca, design system e app (Rust + WebAssembly) documentados no repositório.
**Versão:** 0.1 · outubro de 2026

## 1. O que é

Mirante é uma plataforma aberta para entender a cobertura de uma notícia. O leitor escolhe um tema e recebe:

1. um **relance** curto, em linguagem editorial, sobre o que foi publicado;
2. a **distribuição da cobertura** por espectro (esquerda, centro, direita, sem avaliação);
3. a **factualidade** das fontes e a **posição** de cada veículo;
4. a lista de **artigos e veículos** que sustentam cada afirmação, com link para o original.

Não é um leitor de feed, nem um verificador de fatos, nem um “placar da verdade”. É uma ferramenta de leitura da cobertura: mostra o enquadramento, a amostra e o que ficou de fora.

## 2. Público e trabalho

**Público inicial:** leitores curiosos, jornalistas, pesquisadores, professores e equipes de comunicação que precisam entender rapidamente como um assunto está sendo enquadrado — e que desconfiam de resumos sem fonte.

**Trabalho principal:** “Quando um assunto importante aparece, quero ver como veículos de posições diferentes o cobriram, com evidência suficiente para eu tirar minhas próprias conclusões.”

**Trabalho secundário:** acompanhar temas ao longo do tempo; comparar a cobertura de um veículo; salvar um relance para ler depois.

## 3. Escopo do MVP

### Dentro

- Busca por tema em uma janela de 7–30 dias.
- 200–500 fontes curadas, um país/idioma.
- Agrupamento de artigos por evento.
- Relance com citações por afirmação.
- Quatro visualizações: espectro, factualidade, dispersão de fontes e volume no tempo.
- Lista de fontes com viés, factualidade e “sem avaliação”.
- Cache por tema; estados de vazio, carregando, erro e amostra pequena.
- Auto-hospedagem via Docker Compose.

### Fora do MVP

- 50 mil fontes; feed personalizado; contas e recomendações.
- Ownership detalhado (pesquisa manual intensa; v2).
- App mobile nativo; alertas em tempo real.
- “Blindspot” sofisticado; primeiro apenas um indicador simples de assimetria, com N mínimo.
- Ratings proprietários empacotados no repositório.

## 4. Princípios de produto

1. **Cobertura, não veredito.** O produto descreve o que foi publicado; não decide o que é verdade.
2. **Toda afirmação tem fonte.** O relance cita artigos; os números são calculados em código, não pelo modelo.
3. **N e incerteza visíveis.** Nenhuma porcentagem sem base; “sem avaliação” é um estado de primeira classe.
4. **Calma por padrão.** Sem urgência artificial, sem feed infinito, sem caixa alta.
5. **Aberto por desenho.** Código, método e limites visíveis; dados de terceiros são plugáveis.

## 5. Modelo de conteúdo

```
Source        id, nome, domínio, país, idioma, ownership, rating_ids
Rating        fonte (AllSides/Ad Fontes/MBFC/own), viés, factualidade, confiança, data
Article       id, source_id, url canônica, título, trecho, publicado_em, idioma, hash
StoryCluster  id, título de trabalho, janela, embedding, status
ClusterArticle cluster_id, article_id, similaridade, papel (lead/cobertura)
Report        cluster_id, versão, resumo, seções, limitações, gerado_em
Claim         report_id, texto, article_ids, tipo (fato/enquadramento/ausência), confiança
ChartSnapshot cluster_id, métrica, valores, n, período
```

## 6. Métricas de sucesso

- **Qualidade do cluster:** pureza ≥ 85% em um conjunto humano de 100–200 histórias; taxa de histórias divididas < 10%.
- **Citação:** ≥ 95% das afirmações factuais do relance com pelo menos um artigo de suporte; zero citações inventadas em auditoria.
- **Cobertura de rating:** ≥ 80% dos artigos da amostra com viés conhecido para a lista curada; “sem avaliação” sempre visível.
- **Utilidade:** usuários conseguem explicar, depois de ler, ao menos um enquadramento e um ângulo ausente.
- **Custo:** ≤ US$ 0,10 por relance em modelo pequeno; cache hit ≥ 60%.

## 7. Restrições técnicas

- Backend em **Rust**; frontend em **WebAssembly**.
- Tokens e componentes deste design system são a fonte da verdade visual.
- Sem dependência de Google Fonts em produção; fontes self-hosted.
- O código é licenciado sob MIT OR Apache-2.0; os dados de rating de terceiros têm licença própria e não são empacotados.
- Nenhum texto completo de artigo é republicado; título, trecho curto e link.

## 8. Não objetivos

- Substituir a leitura das reportagens originais.
- Produzir uma pontuação de “verdade” por artigo ou por veículo.
- Competir com o Ground News em número de fontes no dia um.
- Ser uma rede social de notícias.
