# Mirante Design System

**Versão:** 0.1 · outubro de 2026  
**Status:** fundação visual e kit de interface para o app em Rust + WebAssembly.  
**Fonte da verdade:** `design-system/styles/tokens.css` e `design-system/data/tokens.json`. Este documento explica as decisões; os tokens são a implementação.

---

## 1. Princípios de interface

1. **A leitura manda.** A coluna de texto nunca é apertada por dados ou navegação. Dados vão para a lateral, para baixo ou para um painel; o texto mantém 58–66 caracteres por linha.
2. **Hierarquia por escala, não por caixas.** Antes de adicionar um card, uma borda ou uma sombra, tente resolver com tamanho, peso, espaço e um fio.
3. **O dado tem contexto.** Todo gráfico mostra N, unidade, período e o que é desconhecido. Nunca uma porcentagem sem base.
4. **Desconhecido é um estado visível.** “Sem avaliação” tem estilo próprio e não é tratado como falha.
5. **Uma ousadia por tela.** O wordmark, uma manchete ou um gráfico pode ser o momento forte. O resto fica quieto.
6. **Movimento explica.** Nada de animação decorativa em hover de lista, teclado ou digitação. Uma entrada orquestrada por página é suficiente.

---

## 2. Tokens

### 2.1 Cor

A paleta é dividida em **fundação** (papel, tinta, régua), **ação** (ultramar) e **dados** (esquerda, centro, direita, factualidade). As cores de dados nunca viram cor de marca, botão ou link.

| Token | Claro | Escuro | Uso |
| --- | --- | --- | --- |
| `--paper` | `#F4F3ED` | `#11120F` | fundo do app |
| `--paper-raised` | `#FBFAF5` | `#181914` | superfície de leitura, menus |
| `--ink` | `#151611` | `#F1EFE6` | texto principal |
| `--ink-soft` | `#3E3F37` | `#D0CDC0` | texto secundário forte |
| `--muted` | `#6E6D63` | `#A3A094` | metadados, legendas |
| `--faint` | `#9B998D` | `#77756B` | placeholders, eixos, ícones desativados |
| `--rule` | `#D8D5C8` | `#2C2D26` | fios estruturais |
| `--rule-strong` | `#BDB9AA` | `#44453B` | foco de contorno, divisores fortes |
| `--brand` | `#2438E8` | `#8B9BFF` | links, ação primária, foco |
| `--brand-ink` | `#F7F6EF` | `#101118` | texto sobre marca |
| `--accent` | `#A9503B` | `#D98B72` | acento editorial raro; direita em dados |
| `--bias-left` | `#3F5FA8` | `#8FA9E8` | dados de esquerda |
| `--bias-center` | `#8B887B` | `#B5B1A3` | dados de centro |
| `--bias-right` | `#A9503B` | `#D98B72` | dados de direita |
| `--bias-unknown` | `#C6C2B4` | `#55564C` | sem avaliação |
| `--fact-high` | `#2F6B4F` | `#7FBF9A` | factualidade alta |
| `--fact-mixed` | `#7E5714` | `#D8B36A` | factualidade mista |
| `--fact-low` | `#A33A32` | `#E28B80` | factualidade baixa |
| `--success` | `#2F6B4F` | `#7FBF9A` | confirmação |
| `--warning` | `#7E5714` | `#D8B36A` | atenção |
| `--danger` | `#A33A32` | `#E28B80` | erro destrutivo |

**Contraste:** texto normal usa `--ink`, `--ink-soft` ou `--muted`. As cores de dado puras são para pontos, barras e fios; quando viram texto, o texto é `--ink-soft` sobre um fundo tintado. `--bias-center` puro não passa AA para texto pequeno (3.2:1), então nunca é usado como cor de texto.

### 2.2 Tipografia

```css
--font-display: "Instrument Serif", "Iowan Old Style", "Times New Roman", serif;
--font-body: "Literata", "Source Serif 4", Georgia, serif;
--font-ui: "Schibsted Grotesk", "Helvetica Neue", Arial, sans-serif;
```

| Token | Tamanho | Line-height | Uso |
| --- | --- | --- | --- |
| `--text-display-xl` | `clamp(4.75rem, 10vw, 9rem)` | `0.94` | capa, wordmark |
| `--text-display-l` | `clamp(3.5rem, 6.5vw, 6rem)` | `0.96` | manchete de capa |
| `--text-display-m` | `clamp(2.75rem, 4.6vw, 4.25rem)` | `1.00` | manchete de matéria |
| `--text-headline` | `clamp(2rem, 3.2vw, 2.75rem)` | `1.04` | título de seção |
| `--text-title` | `1.5rem` | `1.18` | título de card/painel |
| `--text-lede` | `1.375rem` | `1.45` | olho, standfirst |
| `--text-body` | `1.1875rem` | `1.66` | leitura longa |
| `--text-body-s` | `1.0625rem` | `1.6` | texto secundário |
| `--text-ui` | `0.9375rem` | `1.35` | interface |
| `--text-small` | `0.8125rem` | `1.45` | metadados |
| `--text-caption` | `0.75rem` | `1.4` | legendas |
| `--text-micro` | `0.65625rem` | `1.35` | labels de eixo, microdados |

**Regras**

- Títulos: `text-wrap: balance`; corpo: `text-wrap: pretty`.
- Dados numéricos: `font-variant-numeric: tabular-nums`.
- Literata: `font-optical-sizing: auto`.
- Nunca use `font-weight` abaixo de 300 em texto pequeno; Instrument só tem 400.
- Itálico de Instrument é para citações e olhos, não para ênfase aleatória.
- Não use caixa alta em títulos ou labels; a exceção é um código de dados quando o contexto exigir (ex.: sigla), com tracking `0.08em`.

### 2.3 Espaço

Base 4 px. Use a escala; não invente valores.

`4 · 8 · 12 · 16 · 24 · 32 · 40 · 48 · 64 · 80 · 96 · 128 · 160`

- Dentro de componentes: 8–24 px.
- Entre blocos: 32–64 px.
- Entre seções: 96–160 px.
- Margem lateral: `clamp(20px, 4vw, 64px)`.
- Coluna de leitura: `66ch`; lede: `58ch`; relance: `68ch`.

### 2.4 Grade

12 colunas, gutter `clamp(16px, 2vw, 32px)`, container máximo 1440 px. A composição editorial preferida é **7 + 4 + 1**: leitura, evidência, respiro. Em telas menores, a evidência desce para depois do texto.

### 2.5 Raios, fios e sombras

- `--radius-xs: 2px` — superfícies editoriais, botões, inputs.
- `--radius-sm: 4px` — chips pequenos.
- `--radius-md: 8px` — menus, popovers.
- `--radius-lg: 12px` — apenas superfícies flutuantes grandes.
- `--radius-pill: 999px` — tags e avatares.
- Fio estrutural: `1px solid var(--rule)`.
- Sombra: somente para flutuação. `--shadow-menu` para menus; nada de sombra em cards de conteúdo.
- Raio concêntrico: `raio externo = raio interno + padding`. Em superfícies editoriais, prefira 2 px externo e 0–2 px interno.

### 2.6 Movimento

| Token | Valor | Uso |
| --- | --- | --- |
| `--duration-fast` | `140ms` | hover, foco, cor |
| `--duration-base` | `220ms` | abrir/fechar, estado |
| `--duration-slow` | `420ms` | entrada orquestrada |
| `--ease-out` | `cubic-bezier(.2,0,0,1)` | entradas e respostas |
| `--ease-in-out` | `cubic-bezier(.65,0,.35,1)` | transições de layout |

- Nunca `transition: all`; declare propriedades.
- `:active` usa `scale(.96)` em botões.
- `prefers-reduced-motion: reduce` zera animações e transições.
- Teclado nunca anima.
- Uma entrada por página; não anime cada seção.

---

## 3. Componentes

### 3.1 Botões

- Altura: 36 px (`sm`), 44 px (`md`), 52 px (`lg`). Área de toque mínima 44×44 px no mobile.
- Raio 2 px. Primário: tinta sobre papel; hover: ultramar. Secundário: fio; ghost: texto.
- Ícone opcional com 18 px e `stroke: 1.5`; nunca misture bibliotecas.
- Foco: anel de 2 px de marca com offset de 2 px.
- Loading: manter largura, trocar rótulo por “Gerando…” com `aria-busy="true"`.
- Destrutivo: usar apenas para ações irreversíveis.

### 3.2 Campos

- Label sempre visível, 12 px em Schibsted, `for` associado.
- Input 44 px, fundo `--paper-raised`, fio `--rule`; foco com fio de marca e anel.
- Texto de ajuda 12 px; erro com ícone + texto, `aria-invalid="true"` e `aria-describedby`.
- Placeholder é exemplo, não label.
- Select nativo estilizado; seta em `currentColor`.
- Checkbox/radio usam `accent-color: var(--brand)`; switch é um checkbox com papel `switch`.

### 3.3 Tags

- **Viés:** ponto colorido + rótulo em `--ink-soft`, fundo tintado a 10–12%. Esquerda, centro, direita, sem avaliação.
- **Factualidade:** marcador quadrado + rótulo. Alta, mista, baixa, sem avaliação.
- **Status:** sucesso, atenção, erro. Sempre com texto; cor nunca é o único sinal.
- Tamanho padrão 24 px de altura; `sm` 20 px. Não use tag como botão.

### 3.4 Fonte / veículo

- Linha de 56 px com nome, domínio/ownership, tag de viés, factualidade e contagem.
- Fio inferior; hover com tint de 3%; foco com anel.
- O nome do veículo é o link principal; o ícone externo indica saída.
- Ordenação padrão: relevância/reach, não alfabética.

### 3.5 Card de história

- Superfície `--paper-raised`, fio `--rule`, raio 2 px, padding 24–32 px.
- Sem sombra. Hover apenas no fio (`--rule-strong`) e no título (sublinhado).
- Estrutura: metadados → manchete → olho → barra de viés → rodapé com veículos e tempo.
- Variante compacta: uma linha de manchete, tags e contagem; sem olho.

### 3.6 Gráficos

- **Barra de viés:** 8 px de altura, segmentos sem gap, raio 2 px; legenda com ponto + rótulo + %. Sempre mostrar N.
- **Factualidade:** mesma anatomia, cores de factualidade; “sem avaliação” em cinza.
- **Dispersão:** eixos em `--faint`, grade de 1 px, pontos de 4–10 px por volume, cor por viés. Tooltip em foco e hover; tabela oculta para leitores de tela.
- **Timeline:** três séries, linhas de 1.5 px, sem área preenchida; eixo x com 4–6 datas; legenda.
- **Regras:** nada de 3D, donut, gradiente ou eixos sem unidade. O gráfico precisa de título, N e período.

### 3.7 Relance de IA

- Título “O relance” em Instrument Serif; metadados “128 artigos · 64 veículos · atualizado há 12 min”.
- Corpo em Literata 19/1.66; citações sobrescritas `[1]` em ultramar, link para o artigo.
- Seções: o que aconteceu; como cada lado enquadra; o que ficou de fora; limitações.
- Números são calculados em código; o texto só interpreta. Nunca deixe o modelo inventar contagens.
- Callout de “o que ficou de fora” com fundo tintado, sem borda.
- Limitações sempre visíveis, não escondidas em tooltip.

### 3.8 Estados

- **Vazio:** título claro, explicação e ação. Sem ilustração.
- **Carregando:** skeleton com a mesma altura do conteúdo; `aria-busy`.
- **Erro:** o que aconteceu, o que fazer, ação de retry. Sem desculpas vagas.
- **Amostra pequena:** aviso textual “Amostra pequena (12 artigos); leia com cautela.” ao lado do gráfico.

---

## 4. Acessibilidade

- WCAG 2.2 AA como piso.
- Foco visível em todos os controles; `:focus-visible` com anel de 2 px.
- Área de toque 44×44 px no mobile; 40×40 px em desktop denso.
- Contraste: texto normal ≥ 4.5:1; texto grande ≥ 3:1; gráficos ≥ 3:1.
- Nunca cor sozinha: sempre rótulo, forma ou texto.
- Gráficos têm `role="img"`, `aria-label` com resumo e tabela oculta com os dados.
- `prefers-reduced-motion` respeitado.
- Idioma do documento e dos trechos em outra língua.
- Skip link no topo.
- Navegação por teclado em tabs, menus e campos.
- Zoom de 200% sem perda de conteúdo.

---

## 5. Uso no app Rust + WebAssembly

- O frontend consome `tokens.css` como CSS global; não há runtime de tema em JS. O tema é `data-theme="light|dark"` no `<html>`.
- Componentes podem ser implementados em Leptos/Yew/Dioxus; as classes e a anatomia deste kit são a referência.
- Tokens também estão em `data/tokens.json` para geração de código (Rust, CSS, Figma).
- Fontes são self-hosted em `fonts/`; não dependa de Google Fonts em produção.
- O layout usa CSS Grid e `clamp()`; não há breakpoints de dispositivo, apenas breakpoints de conteúdo (640, 900, 1200 px).

---

## 6. Checklist antes de publicar uma tela

- [ ] A coluna de leitura tem 58–68 caracteres e não é invadida por dados.
- [ ] A hierarquia se resolve sem cards/sombras desnecessários.
- [ ] Todo número tem N, unidade e período.
- [ ] “Sem avaliação” está visível e estilizado.
- [ ] Toda afirmação do relance tem citação.
- [ ] Foco de teclado visível; tab order lógico.
- [ ] Contraste AA; cor não é o único sinal.
- [ ] `prefers-reduced-motion` desliga a entrada.
- [ ] Mobile em 360 px sem scroll horizontal.
- [ ] Nenhum texto em caixa alta decorativa, número 01/02/03 ou gradiente.
- [ ] A página tem um único momento de ousadia.
