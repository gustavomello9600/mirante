# Mirante — conceito de marca

> **Working name.** “Mirante” é o nome de trabalho. Antes de registrar domínio ou marca, vale checar colisões locais (há emissoras e publicações com esse nome). O conceito visual não depende do nome; o sistema continua válido se ele mudar.

## 1. A ideia

Um mirante não interfere na paisagem. Ele muda o ponto de vista.

**Mirante** é uma mesa aberta para ler a cobertura inteira de uma notícia: quem publicou, de onde fala, o que cada lado escolheu destacar e o que ficou de fora. Não é mais uma versão do mesmo fato. É o lugar de onde se vê o conjunto.

O nome vem do português: mirante é o ponto alto de onde se observa o horizonte. A marca não promete a verdade; promete **perspectiva com evidência**.

### Posicionamento

Para quem quer entender uma notícia sem cair no feed de um só lado, o Mirante é uma plataforma aberta de análise de cobertura que reúne fontes, mede o que foi publicado e mostra o enquadramento com links para os artigos. Diferente de agregadores tradicionais, não esconde a curadoria: mostra a amostra, a fonte de cada avaliação e o que é desconhecido.

### Promessa

**A notícia por inteiro.**

Em inglês: **See the whole story.**

### Público

Leitores curiosos, jornalistas, pesquisadores, professores e equipes de comunicação que precisam entender rapidamente como um assunto está sendo enquadrado — e que desconfiam de resumos sem fonte.

## 2. Personalidade

| É | Não é |
| --- | --- |
| Preciso | Dogmático |
| Calmo | Apático |
| Curioso | Cínico |
| Editorial | Nostálgico de jornal impresso |
| Transparente | Ingenuamente “neutro” |
| Contido | Asséptico |

### Voz

Frases curtas. Verbos concretos. Nada de “verdade”, “imparcialidade” ou “ambos os lados” como selo de qualidade. O Mirante não sentencia a notícia; descreve a cobertura e mostra a evidência.

- **Use:** “42% dos artigos vieram de veículos à esquerda.”
- **Não use:** “A esquerda distorceu a notícia.”
- **Use:** “Não há avaliação de viés para 3 veículos.”
- **Não use:** “Viés desconhecido.” como se fosse um defeito do veículo.
- **Use:** “O relance foi gerado a partir de 128 artigos. Limitações ao final.”
- **Não use:** “Resumo definitivo.”

### Princípios

1. **Perspectiva sem postura.** Mostrar posições não é tomar partido.
2. **Evidência antes do veredito.** Toda afirmação do relatório aponta para um artigo.
3. **Calma por padrão.** Sem urgência artificial, sem feed infinito, sem caixa alta gritando.
4. **Desconhecido é um estado.** “Sem avaliação” é informação, não erro a ser escondido.
5. **Aberto por desenho.** Código, método e limites visíveis; dados de terceiros são plugáveis.

## 3. Identidade visual

### 3.1 Marca

O símbolo é um **sol baixo sobre uma linha de horizonte**: o ponto de observação. Simples o suficiente para funcionar a 16 px; geométrico o suficiente para não virar ilustração.

- **Lockup principal:** símbolo + wordmark `mirante` em Instrument Serif.
- **Wordmark:** sempre em caixa baixa, com tracking levemente negativo. Nunca em caixa alta.
- **Assinatura curta:** o símbolo sozinho, em tinta ou sobre fundo de contraste.
- **Clearspace:** no mínimo a altura do símbolo em todos os lados.
- **Não:** esticar, inclinar, adicionar sombra, contornar, trocar a fonte ou aplicar gradiente.

### 3.2 Paleta

Uma paleta de **tinta sobre papel frio**, não de “creme com terracota”. O azul ultramarino é a cor de ação; o barro aparece apenas como dado de espectro (direita) e em pequenos acentos editoriais. O cinza-pedra é o centro; o desconhecido é um cinza claro, não um vermelho de erro.

| Papel | Tinta | Ultramar | Barro | Pedra | Régua |
| --- | --- | --- | --- | --- | --- |
| `#F4F3ED` | `#151611` | `#2438E8` | `#A9503B` | `#8B887B` | `#D8D5C8` |

**Regras**

- Cor de viés é informação, não decoração. Sempre acompanhada de rótulo textual.
- Nunca use a cor de um lado como cor de marca, botão ou link.
- “Sem avaliação” usa cinza neutro e um ponto vazado; nunca vermelho.
- Contraste mínimo AA; textos pequenos usam `--text-secondary`, não as cores de dado puras.

### 3.3 Tipografia

Três famílias, três funções. A combinação é editorial, mas não é a dupla óbvia “serifa de display + sans geométrica”.

| Função | Fonte | Por quê |
| --- | --- | --- |
| Display e wordmark | **Instrument Serif** | Alto contraste, terminações afiadas, presença de capa de revista sem virar luxo genérico. |
| Leitura longa | **Literata** | Serifa desenhada para tela, com eixo óptico; aguenta 19 px e textos de 2.000 palavras. |
| Interface e dados | **Schibsted Grotesk** | Grotesca de redação, com números claros e personalidade discreta; não é Inter. |

**Escala** (fluida, com proporções de terça maior e quarta):

- Display XL: 76–144 px / 0.94
- Display L: 56–96 px / 0.96
- Display M: 44–68 px / 1.00
- Manchete: 32–44 px / 1.04
- Título: 24 px / 1.18
- Olho / lede: 22 px / 1.45
- Corpo: 19 px / 1.66
- Corpo pequeno: 17 px / 1.6
- UI: 15 px / 1.35
- Legenda: 12 px / 1.4
- Micro: 10.5 px / 1.35

**Medida:** 66 caracteres para corpo; 58 para lede. Nunca acima de 72.

**Detalhes:** `text-wrap: balance` em títulos, `pretty` no corpo, `tabular-nums` em dados, `font-optical-sizing: auto` na Literata, `-webkit-font-smoothing: antialiased`.

### 3.4 Espaço e proporção

Base de 4 px. A escala é 4, 8, 12, 16, 24, 32, 48, 64, 96, 128, 160. O ritmo vertical é generoso: seções respiram 96–160 px; componentes internos usam 12–24 px. A coluna de leitura é sagrada; dados podem ocupar a lateral, mas nunca invadem a medida do texto.

Grade de 12 colunas, com gutter fluido. A composição preferida é assimétrica: **7 colunas de leitura + 4 de evidência + 1 de respiro**, não 50/50.

### 3.5 Superfícies e movimento

- Cantos: 2 px em superfícies editoriais; 8 px em controles; pill apenas em tags.
- Bordas: fios de 1 px com alfa; sombra só para flutuação (menus, popovers), nunca para decorar cartões.
- Movimento: 140 ms para hover, 220 ms para estados, 420 ms para uma entrada orquestrada. `ease-out`; nada de `transition: all`.
- Uma única entrada na página: o wordmark revela por clip-path; o resto entra sem coreografia.
- `prefers-reduced-motion` desliga tudo.

### 3.6 Imagem

Pouca imagem. Quando houver, é documento: foto de arquivo, mapa, gráfico, frame de vídeo. Sem banco de imagens genérico, sem ilustração 3D, sem gradiente. A imagem entra com um fio de 1 px e legenda em Schibsted 12 px.

## 4. Aplicação

- **App:** interface calma, coluna de leitura central, evidência na lateral, dados sempre com N e fonte.
- **Editorial:** títulos em Instrument, olho em Literata itálico, metadados em Schibsted.
- **Relance de IA:** sempre com citações, seção de limitações e link para os artigos. O texto nunca calcula os números; os números são calculados em código e citados pelo texto.
- **Social:** um cartão por história, fundo papel, manchete em Instrument, uma barra de viés e a frase “128 artigos · 64 veículos”. Sem logo gigante.

## 5. Frases de marca

- A notícia por inteiro.
- Veja o ângulo antes da manchete.
- Toda cobertura tem um enquadramento. Este é o mapa.
- Menos opinião sobre a imprensa. Mais evidência sobre a cobertura.
- O mirante não escolhe o lado. Ele mostra o horizonte.

## 6. O que evitar

- A fórmula “creme + serifa + terracota” e o selo de “imparcial”.
- Vermelho/azul saturados como marca; eles pertencem aos dados.
- Caixa alta em tudo, números 01/02/03 e labels decorativos.
- Cards idênticos com sombra suave para tudo.
- Gradientes, glassmorphism e ilustrações 3D.
- Prometer “verdade” ou “neutralidade”; prometer **cobertura visível**.
