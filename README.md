# Mirante

**A notícia por inteiro.** Uma plataforma aberta para ler a cobertura inteira de uma notícia: quem publicou, de onde fala, o que cada lado destacou e o que ficou de fora.

[![CI](https://github.com/gustavomello9600/mirante/actions/workflows/ci.yml/badge.svg)](https://github.com/gustavomello9600/mirante/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/WebAssembly-Yew-654ff0.svg)](https://yew.rs/)

> **Status:** MVP de referência, pronto para estudo e auto-hospedagem. Não é um serviço de verificação de fatos nem uma fonte única de verdade. Leia os [limites](#limites-e-princípios) antes de usar em produção.

[English summary](README.en.md) · [Arquitetura](ARCHITECTURE.md) · [Produto](PRODUCT.md) · [Design](DESIGN.md) · [Marca](BRAND.md) · [Como contribuir](CONTRIBUTING.md)

---

## O que é

O Mirante recebe um tema, busca a cobertura publicada em uma janela de tempo e devolve:

- **Relance** editorial curto, com citações por artigo e fallback extrativo quando não há LLM.
- **Distribuição por espectro** (esquerda, centro, direita e sem avaliação), factualidade e dispersão das fontes.
- **Top 3 matérias por espectro**, com acesso à amostra completa.
- **Todas as matérias**, com filtro por espectro, contagem e ordenação por relevância ou recência.
- **Leitura profunda seletiva**: o servidor abre algumas matérias representativas e usa um trecho curto para enriquecer o relance — sem republicar o texto integral.
- **Fontes** com viés, factualidade, ownership e aviso explícito de curadoria demonstrativa.
- **Estados reais**: carregando com progresso e ETA, vazio, erro com retry, amostra pequena.
- **Salvos no navegador**, compartilhamento, tema claro/escuro e layout acessível.

## Stack

| Camada | Tecnologia |
| --- | --- |
| Backend | Rust, Axum, Tokio, SQLite (`sqlx`) |
| Análise | TF-IDF, stemming PT/EN, clustering aglomerativo, extração de trechos |
| LLM (opcional) | API compatível com OpenAI: OpenRouter, OpenAI ou mock local |
| Frontend | Rust + Yew 0.23, compilado para WebAssembly |
| Design system | HTML/CSS/JS estático, tokens CSS e JSON |
| Deploy | Docker multi-stage, Docker Compose, Nginx Proxy Manager; frontend estático opcional no Vercel |

```text
navegador
  │ HTTPS
  ▼
[ mirante-web :8091 ]
  ├── Axum / Rust
  │    ├── /api/health
  │    ├── /api/topics
  │    ├── /api/sources
  │    ├── /api/search
  │    └── /api/jobs  (progresso por polling)
  ├── SQLite (cache de análises)
  └── Google News RSS · Bing News RSS · GDELT · feeds curados
```

Detalhes de fluxo, módulos e decisões em [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Rodar localmente

### Pré-requisitos

- Rust 1.85+ (o projeto é testado com a toolchain estável atual)
- `wasm-pack` e o target `wasm32-unknown-unknown` para o frontend
- Docker e Docker Compose (opcional, para o caminho mais simples)

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

### Caminho rápido com Docker

```bash
git clone https://github.com/gustavomello9600/mirante.git
cd mirante
cp .env.example .env        # opcional: preencha o LLM
docker compose up --build
```

Abra <http://localhost:8091>. O container publica a porta apenas em `127.0.0.1`; use um proxy reverso para expor publicamente.

### Backend sem Docker

```bash
cd backend
MIRANTE_PORT=8091 \
MIRANTE_DB=/tmp/mirante.db \
MIRANTE_DIST=../frontend/dist \
MIRANTE_FULLTEXT_MAX=12 \
cargo run
```

### Frontend

```bash
cd frontend
wasm-pack build --target web --release --no-opt --out-dir dist/pkg
cp index.html config.js dist/
cp -R styles/. dist/styles/
cp -R fonts/. dist/fonts/
```

O backend serve `MIRANTE_DIST`; abra <http://localhost:8091>. Para iterar apenas no visual, o design system também roda sem build:

```bash
python3 -m http.server 4173 --directory design-system
```

## Configuração

Todas as variáveis são opcionais; o app funciona sem chave de LLM e sem leitura profunda.

| Variável | Padrão | Descrição |
| --- | --- | --- |
| `MIRANTE_PORT` | `8091` | Porta HTTP do backend. |
| `MIRANTE_DB` | `./data/mirante.db` | Caminho do SQLite usado como cache. |
| `MIRANTE_DIST` | `./dist` | Diretório com o build do frontend. |
| `MIRANTE_CACHE_TTL` | `1800` | TTL do cache de análises, em segundos. |
| `MIRANTE_FULLTEXT_ENABLED` | `true` | Liga a leitura profunda seletiva. |
| `MIRANTE_FULLTEXT_MAX` | `12` | Máximo de matérias lidas (4–24). |
| `MIRANTE_LLM_PROVIDER` | `none` | `none`, `openrouter`, `openai` ou `mock`. |
| `MIRANTE_LLM_API_KEY` | vazio | Chave do provedor. **Nunca versione.** |
| `MIRANTE_LLM_MODEL` | `openai/gpt-4o-mini` | Modelo usado no relance. |
| `MIRANTE_LLM_BASE_URL` | `https://openrouter.ai/api/v1` | Endpoint compatível com OpenAI. |
| `MIRANTE_USER_AGENT` | repositório do projeto | Identificação enviada aos veículos. |
| `MIRANTE_PUBLIC_URL` | repositório do projeto | `HTTP-Referer` enviado ao OpenRouter. |
| `MIRANTE_DEBUG` | `false` | Expõe detalhes técnicos de erros 500. Só em desenvolvimento. |

Sem `MIRANTE_LLM_API_KEY`, o relance é **extrativo**: títulos, estatísticas, citações e limitações. Com chave, o modelo gera o texto, mas o servidor valida as citações e mantém o fallback extrativo se o provedor falhar ou omitir uma seção.

### Frontend estático

O `frontend/config.js` define `window.MIRANTE_API_BASE`. Vazio significa mesma origem (Docker). Para publicar só o frontend:

```bash
cd frontend
MIRANTE_API_BASE=https://api.example.com ./deploy-vercel.sh
```

`VERCEL_PROJECT` permite escolher outro projeto no Vercel. O backend precisa liberar CORS para o domínio do frontend; a configuração padrão é permissiva e deve ser restringida em produção se o serviço for público.

## Testes e qualidade

```bash
# Backend: unidade, formatação e lints
cd backend
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# Frontend: compilação para WebAssembly
cd frontend
cargo fmt --all -- --check
cargo check --target wasm32-unknown-unknown
```

O CI executa esses comandos em cada push e pull request. A suíte cobre catálogo, parsing de feeds, tokenização, clustering, relatório e extração de trechos.

## Deploy

### Docker Compose

```bash
cp .env.example .env
# edite .env apenas no host; nunca faça commit dele
docker compose build
docker compose up -d
curl http://127.0.0.1:8091/api/health
```

O `docker-compose.yml` espera uma rede externa `proxy_network` e publica a porta em `127.0.0.1:8091`. Ajuste os nomes de rede, volume e container ao seu ambiente.

### Script de deploy

`deploy.sh` sincroniza o código, reconstrói a imagem no host, sobe a stack e roda smoke tests. Ele não envia `.env` nem arquivos de build.

```bash
MIRANTE_HOST=deploy@news.example.com \
MIRANTE_DOMAIN=news.example.com \
MIRANTE_SSH_KEY="$HOME/.ssh/id_ed25519" \
./deploy.sh
```

Para registrar TLS e proxy host no Nginx Proxy Manager:

```bash
MIRANTE_DOMAIN=news.example.com \
NPM_EMAIL=admin@example.com \
NPM_PASSWORD='...' \
bash deploy/register-npm.sh
```

Prefira `NPM_ENV_FILE` apontando para um arquivo fora do repositório. Veja [`deploy/`](deploy/) e [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Segurança

- **Nunca faça commit de `.env`.** O arquivo está no `.gitignore`; use `.env.example` para documentar variáveis.
- O CI tem verificação de segredos com Gitleaks. Se uma chave vazar, **rotacione imediatamente** no provedor; remover o commit não basta.
- A API não tem autenticação nem rate limit por IP; o limite é um semáforo de 3 análises simultâneas. Para exposição pública, coloque-a atrás de proxy, limite requisições e monitore custo de LLM.
- Erros 500 não devolvem detalhes técnicos por padrão (`MIRANTE_DEBUG=false`).
- A configuração padrão de CORS é permissiva. Restrinja origens se o backend for público.
- O container roda como root no MVP; para produção, aplique um usuário não privilegiado e políticas de rede do seu ambiente.

Encontrou uma vulnerabilidade? Siga [`SECURITY.md`](SECURITY.md) e **não abra uma issue pública**.

## Limites e princípios

- **Cobertura, não veredito.** O produto descreve o que foi publicado; não decide o que é verdade.
- **Toda afirmação tem fonte.** O relance cita artigos; números são calculados em código, não pelo modelo.
- **N visível.** Nenhuma porcentagem sem base; “sem avaliação” é um estado de primeira classe.
- **Aberto por desenho.** Código, método e limites visíveis; dados de terceiros são plugáveis.

Limites conhecidos:

- As avaliações de viés/factualidade em `backend/data/sources.json` são uma **curadoria demonstrativa**, não uma base licenciada. Não use como verdade editorial. Veja [`backend/data/README.md`](backend/data/README.md).
- A análise usa títulos, metadados e trechos curtos; não armazena nem republica o texto integral.
- Paywalls, consentimentos e bloqueios podem reduzir a leitura profunda.
- O clustering usa manchetes e TF-IDF; é uma aproximação, não uma compreensão semântica completa.
- Ainda não há contas, feed personalizado, alertas, app mobile ou ownership detalhado no relance.
- O custo do LLM por relance deve ser monitorado; o cache reduz chamadas.

## Estrutura

```text
backend/            Rust/Axum: fetch, clustering, ratings, relatório, LLM, SQLite
  src/              módulos da API e do motor de análise
  data/             catálogo de fontes e feeds curados (dados de demonstração)
frontend/           Yew/WASM: home, relance, gráficos, fontes, salvos
design-system/      tokens, componentes de referência e documentação viva
deploy/             registro no Nginx Proxy Manager
Dockerfile          multi-stage: WASM → binário Rust → Debian slim
docker-compose.yml  stack de referência
```

## Documentação

- [`ARCHITECTURE.md`](ARCHITECTURE.md) — arquitetura, fluxo, deploy, testes e limites.
- [`PRODUCT.md`](PRODUCT.md) — produto, público, escopo do MVP e métricas.
- [`DESIGN.md`](DESIGN.md) — regras do design system, tokens, componentes e acessibilidade.
- [`BRAND.md`](BRAND.md) — conceito, voz, identidade, símbolo, paleta e tipografia.
- [`design-system/README.md`](design-system/README.md) — site e tokens do design system.
- [`CONTRIBUTING.md`](CONTRIBUTING.md) — como contribuir, padrões de commit e revisão.
- [`CHANGELOG.md`](CHANGELOG.md) — histórico de versões.
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) — fontes e licenças de terceiros.

## Licença

O código é distribuído sob **MIT OR Apache-2.0**, à sua escolha. Veja [`LICENSE-MIT`](LICENSE-MIT) e [`LICENSE-APACHE`](LICENSE-APACHE).

Os dados de rating de terceiros têm licença própria e não são empacotados. As fontes do design system são distribuídas sob a SIL Open Font License; os textos das licenças estão em `design-system/fonts/` e `frontend/fonts/`.

As marcas, nomes e logotipos de veículos citados pertencem aos seus respectivos titulares. O nome “Mirante” é um nome de trabalho; verifique colisões de marca antes de registrar domínio ou marca.

---

Feito para leitura calma, com evidência e sem atalhos.
