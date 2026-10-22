# Avisos de terceiros

Este arquivo reúne os principais componentes de terceiros distribuídos ou referenciados pelo Mirante. Ele não substitui as licenças completas incluídas no repositório.

## Fontes

As fontes são self-hosted em WOFF2 e distribuídas sob a **SIL Open Font License 1.1**. Os textos completos estão em:

| Família | Autoria | Licença | Arquivo |
| --- | --- | --- | --- |
| Instrument Serif | The Instrument Serif Project Authors | OFL 1.1 | [`design-system/fonts/instrumentserif-OFL.txt`](design-system/fonts/instrumentserif-OFL.txt) |
| Literata | The Literata Project Authors | OFL 1.1 | [`design-system/fonts/literata-OFL.txt`](design-system/fonts/literata-OFL.txt) |
| Schibsted Grotesk | The Schibsted-Grotesk Project Authors | OFL 1.1 | [`design-system/fonts/schibstedgrotesk-OFL.txt`](design-system/fonts/schibstedgrotesk-OFL.txt) |

Os mesmos arquivos são replicados em `frontend/fonts/`.

## Dependências Rust

As dependências declaradas em `backend/Cargo.toml` e `frontend/Cargo.toml` são distribuídas sob licenças próprias (em geral MIT, Apache-2.0, BSD ou ISC). Consulte `Cargo.lock` e os metadados de cada crate para a versão exata. Ao adicionar uma dependência, verifique a compatibilidade com **MIT OR Apache-2.0** e registre exceções neste arquivo.

## Conteúdo jornalístico

O Mirante não redistribui o texto integral de matérias. Títulos, trechos curtos, metadados e links apontam para os veículos originais, que mantêm todos os direitos sobre seu conteúdo. Marcas e logotipos citados pertencem aos seus respectivos titulares.

A coleta usa feeds e APIs públicos (Google News RSS, Bing News RSS, GDELT e feeds curados). Quem opera uma instância deve respeitar os termos de uso de cada fonte, `robots.txt`, limites de requisição e direitos autorais aplicáveis.

## Ratings de viés e factualidade

`backend/data/sources.json` contém uma **curadoria demonstrativa** criada para o MVP. Não é uma base licenciada da AllSides, Ad Fontes Media, MBFC ou de qualquer outro provedor. Não redistribua esses dados como se fossem uma avaliação oficial. Para uso sério, substitua por uma base licenciada ou por metodologia própria auditável, documentando fonte, data e confiança.

## Marcas

“Mirante” é um nome de trabalho. Verifique colisões de marca e domínio antes de registrar. Os nomes dos veículos são usados apenas para identificação editorial e não implicam endosso.
