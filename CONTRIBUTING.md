# Como contribuir

Obrigado pelo interesse no Mirante. Este projeto é um MVP aberto e contribuições de código, documentação, design, acessibilidade e pesquisa são bem-vindas.

Ao participar, você concorda em seguir o [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md).

## Antes de começar

- Leia [`README.md`](README.md), [`ARCHITECTURE.md`](ARCHITECTURE.md) e [`PRODUCT.md`](PRODUCT.md).
- Para mudanças grandes, abra uma **issue de proposta** antes de implementar. Isso evita trabalho duplicado e alinha o escopo.
- Para bugs, use o template de bug report e inclua passos de reprodução, ambiente e logs sem segredos.
- Para vulnerabilidades, **não abra issue pública**. Siga [`SECURITY.md`](SECURITY.md).

## Ambiente

```bash
git clone https://github.com/gustavomello9600/mirante.git
cd mirante
cp .env.example .env
rustup target add wasm32-unknown-unknown
cargo install wasm-pack
```

O backend roda sem chave de LLM. Para o modo generativo, preencha `MIRANTE_LLM_API_KEY` apenas no seu `.env` local; nunca em um commit, issue, print ou pull request.

## Fluxo de trabalho

1. Faça um fork e crie uma branch a partir de `main`:
   `git switch -c feat/descricao-curta`
2. Faça mudanças pequenas e focadas. Evite reformatações massivas sem relação com o objetivo.
3. Rode os testes e lints localmente (veja abaixo).
4. Abra um pull request usando o template, descrevendo **o problema**, **a solução**, **como testar** e **riscos**.
5. Responda à revisão. O merge é feito por squash.

### Padrão de commits

Use mensagens no imperativo, curtas e específicas, preferencialmente no padrão Conventional Commits:

```text
feat: adiciona filtro por espectro na lista de matérias
fix: corrige timeout na leitura profunda
docs: documenta variáveis de ambiente
test: cobre parsing de feed sem data
chore: atualiza dependências
```

Evite mensagens genéricas como “update” ou “fix”. Se a mudança for grande, explique o contexto no corpo do commit ou do PR.

## Testes e qualidade

Execute antes de abrir o PR:

```bash
# Backend
cd backend
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# Frontend
cd frontend
cargo fmt --all -- --check
cargo check --target wasm32-unknown-unknown
```

O CI também executa verificação de segredos com Gitleaks. Não adicione exceções sem justificativa.

### Testes de interface

Para mudanças visuais, verifique:

- desktop (≥ 1280 px) e mobile (360–390 px);
- tema claro e escuro;
- navegação por teclado e foco visível;
- estados de carregando, vazio, erro e amostra pequena;
- ausência de overflow horizontal;
- contraste e nomes acessíveis (WCAG 2.2 A/AA).

Capture screenshots ou descreva a verificação no PR quando a mudança for visual.

## Estilo de código

- **Rust:** `rustfmt` e `clippy` sem warnings. Prefira nomes claros a comentários óbvios; comente decisões e limites.
- **Frontend:** siga os tokens de `design-system/styles/tokens.css`; não introduza cores, fontes ou espaçamentos fora do sistema sem discussão.
- **Texto e documentação:** português claro, frases curtas, sem jargão desnecessário. Use `N` e incerteza quando falar de dados.
- **Dados:** não inclua ratings proprietários, textos integrais de artigos ou dados pessoais. O catálogo em `backend/data/sources.json` é demonstrativo.
- **Dependências:** justifique novas dependências no PR. Prefira crates mantidos e com licença compatível.

## O que não entra

- Segredos, chaves, tokens, `.env` ou credenciais.
- Texto integral de matérias de terceiros ou conteúdo protegido por paywall.
- Dados de rating licenciados sem permissão de redistribuição.
- Conteúdo discriminatório, assédio ou ataques a pessoas.
- Mudanças que apresentem o relance como veredito de verdade.

## Licença das contribuições

Ao enviar uma contribuição, você concorda que ela será licenciada sob **MIT OR Apache-2.0**, sem termos adicionais. Não é necessário CLA.
