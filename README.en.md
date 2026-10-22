# Mirante

**The whole story.** An open platform for reading the full coverage of a news story: who published it, where each outlet stands, what each side highlighted, and what was left out.

[![CI](https://github.com/gustavomello9600/mirante/actions/workflows/ci.yml/badge.svg)](https://github.com/gustavomello9600/mirante/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

> **Status:** reference MVP for study and self-hosting. It is not a fact-checking service or a single source of truth. Read the [limitations](README.md#limites-e-princípios) before deploying it publicly.

The primary documentation is in Portuguese: [`README.md`](README.md). This page is a short English overview.

## What it does

- Searches recent coverage of a topic across curated outlets, Google News, Bing News and GDELT.
- Clusters headlines by TF-IDF similarity and ranks articles by centrality, recency and source diversity.
- Produces an editorial brief with per-article citations and an extractive fallback when no LLM is configured.
- Optionally reads a sample of full articles, storing only a short excerpt to enrich the brief.
- Shows spectrum distribution, source factuality, source scatter and publication timeline.
- Lets readers filter all articles by spectrum and sort them by relevance or recency.
- Ships with a living design system, light/dark themes, accessible states and browser-local saved items.

## Stack

Rust + Axum backend, SQLite cache, optional OpenAI-compatible LLM, and a Yew/WebAssembly frontend. No Node runtime in production.

## Quick start

```bash
git clone https://github.com/gustavomello9600/mirante.git
cd mirante
cp .env.example .env
docker compose up --build
```

Open <http://localhost:8091>.

For local development without Docker, see the [Portuguese README](README.md#rodar-localmente).

## Configuration

All settings are environment variables; the full table is in [`README.md`](README.md#configuração). The most important ones:

| Variable | Default | Purpose |
| --- | --- | --- |
| `MIRANTE_LLM_PROVIDER` | `none` | `none`, `openrouter`, `openai` or `mock`. |
| `MIRANTE_LLM_API_KEY` | empty | Provider key. Never commit it. |
| `MIRANTE_FULLTEXT_ENABLED` | `true` | Enable selective full-text reading. |
| `MIRANTE_DEBUG` | `false` | Return technical detail on 500 errors. Development only. |

Without an API key the app works in extractive mode.

## Tests

```bash
cd backend && cargo test
cd frontend && cargo check --target wasm32-unknown-unknown
```

## Security

Never commit `.env`. Report vulnerabilities privately according to [`SECURITY.md`](SECURITY.md). The default API has no authentication or per-IP rate limiting; put it behind a proxy and restrict CORS before exposing it publicly.

## License

Code is available under **MIT OR Apache-2.0**. Third-party fonts are under the SIL Open Font License; see [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md). Bias/factuality ratings included in the repository are demonstrative data, not a licensed dataset.

---

Built for calm reading, with evidence and without shortcuts.
