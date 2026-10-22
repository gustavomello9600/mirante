# Mirante Design System

Fundação visual e kit de interface do Mirante. O site é uma peça de documentação viva: mostra marca, tipografia, cor, espaço, componentes e um relance completo em uso.

## Rodar localmente

Não há build. Basta servir a pasta:

```bash
python3 -m http.server 4173 --directory design-system
# abra http://localhost:4173
```

Ou, de dentro de `design-system/`:

```bash
npx serve .
```

## Estrutura

```
design-system/
  index.html                 # documentação viva
  styles/tokens.css          # fonte da verdade: cores, tipo, espaço, movimento
  styles/site.css            # base + componentes de referência
  scripts/site.js            # tema, menu, charts, cópia, estados
  data/tokens.json           # tokens para geração de código / Figma
  fonts/                     # Instrument Serif, Literata, Schibsted Grotesk
```

## Tokens

- **CSS:** importe `styles/tokens.css` no root do app. O tema é `data-theme="light"` ou `"dark"` no `<html>`.
- **JSON:** `data/tokens.json`, com valores claro/escuro e escalas.
- **Aliases semânticos:** `--bg-canvas`, `--bg-surface`, `--text-primary`, `--text-secondary`, `--border-subtle`, `--action`.
- **Dados:** `--bias-left`, `--bias-center`, `--bias-right`, `--bias-unknown`, `--fact-high`, `--fact-mixed`, `--fact-low`.
- **Tipografia:** `--font-display`, `--font-body`, `--font-ui`; escala fluida com `clamp()`.

## Regras rápidas

- Leitura em 58–68 caracteres; dados na lateral, nunca dentro da medida do texto.
- Hierarquia por escala e espaço; card/sombra só quando houver uma função.
- Vermelho, azul e barro são dados; não são cor de marca, botão ou link.
- “Sem avaliação” é cinza e tem ponto vazio; nunca vermelho.
- Uma ousadia por tela. O wordmark ou a manchete é o momento forte.
- Movimento: 140 ms hover, 220 ms estado, 420 ms entrada; respeite `prefers-reduced-motion`.

## Acessibilidade

O site passa nas regras WCAG 2.2 A/AA do axe em claro e escuro, desktop e mobile. Isso inclui contraste, foco, nome acessível, alvos de toque e ausência de overflow horizontal em 360 px. Gráficos têm resumo em `aria-label` e tabela oculta quando necessário.

## Deploy

A pasta é estática. O Dockerfile copia o design system para `/design-system/` dentro do `dist` servido pelo backend. Também é possível publicar separadamente:

```bash
npx vercel deploy design-system --prod
```

O `vercel.json` define cache de fontes e cabeçalhos básicos de segurança.

## Próximo passo

O app em Rust + WebAssembly deve consumir `tokens.css` e reproduzir a anatomia dos componentes em `index.html`. O design system não depende de React, Tailwind ou de qualquer runtime de tema.
