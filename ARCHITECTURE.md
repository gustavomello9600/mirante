# Mirante — arquitetura do MVP

**Status:** MVP de referência.  
**Deploy:** build nativo + Docker Compose + Nginx Proxy Manager, com frontend estático opcional.

## 1. Visão geral

```
navegador
  │  HTTPS (Nginx Proxy Manager, Let's Encrypt)
  ▼
[ mirante-web :8091 ]
  ├── Axum / Rust
  │    ├── /api/jobs      → job de análise com progresso real (polling)
  │    ├── /api/jobs/{id}
  │    ├── /api/search   → busca síncrona, clustering, relatório, cache
  │    ├── /api/topics
  │    ├── /api/sources
  │    ├── /api/health
  │    └── /*            → assets do Yew/WASM
  ├── SQLite (/data/mirante.db)
  └── fontes externas: Google News RSS + Bing News RSS + GDELT + feeds curados
```

O frontend é um SPA em **Yew 0.23**, compilado para WebAssembly com `wasm-pack`, servido pelo mesmo binário Rust que expõe a API. Não há Node em produção.

## 2. Backend (Rust/Axum)

| Módulo | Papel |
| --- | --- |
| `main.rs` | configuração, bind, shutdown gracioso |
| `api.rs` | rotas, estado, cache, erros, assets |
| `fetch.rs` | Google News RSS (busca ampla + `site:` por veículo), Bing News RSS com URLs diretas, GDELT, dedup |
| `catalog.rs` | catálogo de 205 veículos com viés/factualidade de demonstração |
| `text.rs` | normalização, stopwords PT/EN, stemming Snowball, TF-IDF, cosseno |
| `cluster.rs` | agrupamento aglomerativo por centróide, threshold 0.22 |
| `report.rs` | relance extrativo com citações, grupos, ausências e limitações |
| `llm.rs` | adaptador OpenAI-compatible (OpenRouter/OpenAI/mock), validação de citações |
| `analysis.rs` | orquestra ratings, clusters, leitura profunda, stats e relance |
| `fulltext.rs` | seleção por relevância, fetch concorrente limitado e extração de trechos |
| `jobs.rs` | store de jobs, progresso, ETA e status para o loading |
| `store.rs` | SQLite + cache de análises (TTL 30 min) |

### Fluxo de uma busca

1. Normaliza o tema e consulta o cache pelo hash `tema|janela|modelo`.
2. Busca variantes no Google News (`when:7d`), Bing News e, se houver base, buscas `site:` nos 17 veículos curados.
3. Se a amostra for pequena, tenta GDELT.
4. Deduplica por título normalizado + domínio; prioriza URL direta do publisher sobre redirect do Google News.
5. Aplica o catálogo de ratings; desconhecidos ficam como `unknown`.
6. Gera TF-IDF e agrupa manchetes por similaridade; calcula relevância por centralidade, recência, factualidade e diversidade.
7. Seleciona até `MIRANTE_FULLTEXT_MAX` matérias com URLs diretas, respeitando cotas por espectro, e lê a íntegra com concorrência limitada. Extrai um trecho curto para o relance; não armazena nem republica o texto completo.
8. Calcula estatísticas, timeline, dispersão e lista de fontes.
9. Gera o relance extrativo; se `MIRANTE_LLM_PROVIDER` estiver configurado, tenta o modelo e valida as citações. Em erro/omissão, mantém ou completa com o extrativo.
10. Grava o JSON no SQLite e responde. No fluxo de job, publica progresso, contagem e ETA a cada etapa.

### Relance

- **Extrativo (padrão sem chave):** seções “O que aconteceu”, “O que ficou de fora” e “Limitações”, com citações numéricas para as manchetes. A matéria própria é gerada apenas no modo LLM; sem chave, o top 3 por espectro continua no relance.
- **LLM (quando configurado):** OpenRouter `openai/gpt-4o-mini`, temperatura 0.2, `response_format=json_object`, `max_tokens=3200`. Gera uma matéria própria com título/dek e as seções factuais.
  - O prompt exige JSON, citações por índice e proíbe inventar fatos/números.
  - Citações fora do conjunto enviado são descartadas.
  - Citações inline `[n]` são convertidas para o campo `citations`.
  - A antiga seção “Como cada lado enquadra” foi removida; o frontend insere os top 3 por espectro no mesmo ponto do relance.
  - Se “O que ficou de fora” faltar, a seção/callout extrativo é inserido.
  - Falha do provedor → fallback extrativo + limitação visível.

## 3. Frontend (Yew/WASM)

- **Home:** busca, tópicos sugeridos, princípios e metodologia.
- **Relance:** manchete do tema, metadados, relance com citações, gráfico de espectro, factualidade, dispersão, timeline, fontes e lista de manchetes.
- **Top 3 por espectro:** cards com as matérias mais relevantes de cada grupo e atalho para a amostra completa.
- **Todas as matérias:** página com filtro multi-seleção por espectro, contagem, ordenação por relevância/recência e trechos das matérias lidas.
- **Salvos:** localStorage; nenhum dado sai do navegador.
- **Fontes:** catálogo ordenado, com viés, factualidade, ownership e aviso de curadoria demo.
- **Estados:** loading premium com etapas, progresso real, contagem e ETA; vazio, erro com retry, amostra pequena.
- **Tema:** claro/escuro persistido em `localStorage`; `data-theme` no `<html>`.
- **Deploy estático:** `frontend/deploy-vercel.sh` publica apenas o `dist`; a API é configurada em `frontend/config.js` via `window.MIRANTE_API_BASE` (ou por `MIRANTE_API_BASE` no script de deploy).
- **Compartilhar:** Clipboard API com fallback `execCommand("copy")`.
- **Acessibilidade:** axe WCAG 2.2 A/AA limpo em home/relance/todas as matérias, desktop/mobile e tema escuro; foco programático nos títulos; gráficos com `aria-label`; `prefers-reduced-motion` para o loading; sem overflow em 360 px.

## 4. Dados e limites

- **Ratings:** `backend/data/sources.json` é uma curadoria **demo**, não uma base licenciada. Em produção, troque por AllSides/Ad Fontes/MBFC ou metodologia própria auditável.
- **Cobertura:** a busca ampla do Google News traz a cauda longa; as buscas `site:` garantem os veículos avaliados. A maioria dos artigos de nicho fica “sem avaliação”, e isso é mostrado.
- **Texto:** a análise usa títulos, metadados e trechos extraídos de leituras profundas; não armazena nem republica o texto completo. As citações abrem a matéria original. Paywalls e bloqueios podem reduzir a cobertura de leitura profunda.
- **Amostra:** 7 dias, até 280 artigos. Amostras pequenas geram aviso e limitações; o produto não inventa uma conclusão de ausência.
- **Cache:** 30 minutos por tema. `?refresh=true` força nova análise.
- **Rate limit:** semáforo de 3 análises simultâneas; proteção básica, não um rate limit por IP.

## 5. Deploy

Arquivos:

- `Dockerfile` — multi-stage: WASM (Rust + wasm-pack) → backend Rust → Debian slim.
- `docker-compose.yml` — container `mirante-web`, volume `mirante_data`, rede externa `proxy_network`.
- `deploy/register-npm.sh` — cria certificado Let's Encrypt e proxy host no NPM.
- `deploy.sh` — sincroniza, builda, sobe, smoke local e público.

```bash
MIRANTE_HOST=deploy@news.example.com \
MIRANTE_DOMAIN=news.example.com \
./deploy.sh
```

Topologia de referência:

- Container: `mirante-web` (healthcheck em `/api/health`)
- Porta interna: `8091`; publicada só em `127.0.0.1`
- Proxy: `news.example.com` → `mirante-web:8091`
- Design system: `/design-system/` (estático, dentro do mesmo container)
- TLS: Let's Encrypt via NPM; HTTP → HTTPS 301
- Frontend estático opcional: apenas `frontend/dist`, com `window.MIRANTE_API_BASE` apontando para a API

O `.env` fica no host, fora do Git e da imagem. O `deploy.sh` exclui `.env` do rsync.

### Variáveis

```env
MIRANTE_LLM_PROVIDER=openrouter   # none | openrouter | openai | mock
MIRANTE_LLM_API_KEY=...
MIRANTE_LLM_MODEL=openai/gpt-4o-mini
MIRANTE_LLM_BASE_URL=https://openrouter.ai/api/v1
MIRANTE_FULLTEXT_ENABLED=true
MIRANTE_FULLTEXT_MAX=12
MIRANTE_USER_AGENT="Mirante/0.1 (+https://news.example.com; leitor de cobertura)"
MIRANTE_PUBLIC_URL=https://news.example.com
MIRANTE_DEBUG=false
```

O `.env` fica no host, fora do Git e da imagem. Sem chave, o app funciona em modo extrativo. A lista completa está em [`.env.example`](.env.example).

## 6. Testes executados

- `cargo test` no backend: 17 testes (catálogo, fetch RSS/Google/Bing, tokenização, clustering, relatório, fulltext e redação de segredos em logs).
- Build `wasm-pack` release.
- Docker build ARM64 + container healthy.
- Smoke HTTPS: `/api/health`, `/`, assets, fontes.
- Playwright em produção:
  - home, busca por tema, relance IA, gráficos, fontes, salvos, compartilhar, tema, deep link, vazio;
  - desktop 1440 e mobile 390;
  - sem erros de console; sem overflow horizontal.
- axe WCAG 2.2 A/AA: home e relance sem violações.
- LLM: integração testada com mock local (modo `llm`, citações) e com OpenRouter em produção.

## 7. Limitações conhecidas / próximos passos

- Ratings demo; substituir por base licenciada ou metodologia auditável.
- Clustering por TF-IDF em manchetes; para o próximo ciclo, embeddings multilíngues + adjudicação de casos ambíguos.
- Sem contas, feed personalizado, alertas, ownership na página de relance ou app mobile.
- Sem rate limit por IP nem fila distribuída; o semáforo é suficiente para o MVP interno.
- Sem testes end-to-end automatizados versionados no repositório (os testes de produção foram feitos via Playwright na sessão).
- Custo do LLM por relance é baixo, mas deve ser monitorado; cache reduz chamadas.
