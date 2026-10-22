# Changelog

Todas as mudanças relevantes são documentadas aqui.

O formato segue [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e o projeto adere ao [Versionamento Semântico](https://semver.org/lang/pt-BR/).

## [Unreleased]

### Adicionado

- Documentação de governança: contribuição, segurança e código de conduta.
- CI para formatação, lints, testes e compilação WebAssembly.
- Verificação de segredos com Gitleaks.
- Templates de issue e pull request.

### Alterado

- Configuração do frontend movida para `frontend/config.js`, sem URLs de produção embutidas.
- Erros 500 da API não expõem detalhes técnicos por padrão.
- A chave do LLM é redigida na formatação de diagnóstico (`Debug`).
- Scripts de deploy passam a exigir host e domínio por variáveis de ambiente.
- User-Agent e `HTTP-Referer` configuráveis por ambiente.

## [0.1.0] - 2026-10-22

### Adicionado

- Backend em Rust/Axum com busca em Google News, Bing News, GDELT e feeds curados.
- Clustering de manchetes por TF-IDF e ranqueamento por relevância, recência e diversidade.
- Relance extrativo com citações e integração opcional com LLM compatível com OpenAI.
- Leitura profunda seletiva de matérias, com extração de trechos e limite de concorrência.
- API de jobs com progresso, contagem e ETA.
- Frontend em Yew/WebAssembly: home, relance, gráficos, todas as matérias, fontes e salvos.
- Design system com tokens CSS/JSON, tipografia, componentes e documentação viva.
- Deploy com Docker Compose, Nginx Proxy Manager e frontend estático opcional.
- Testes de unidade para catálogo, feeds, texto, clustering, relatório e fulltext.

[Unreleased]: https://github.com/gustavomello9600/mirante/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/gustavomello9600/mirante/releases/tag/v0.1.0
