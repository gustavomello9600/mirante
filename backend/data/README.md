# Dados de fontes e feeds

## `sources.json`

Catálogo de veículos usado para atribuir espectro político e factualidade às manchetes. Cada item segue, aproximadamente:

```json
{
  "name": "Nome do veículo",
  "domain": "exemplo.com",
  "lean": "left | center | right | unknown",
  "factuality": "high | mixed | low | unknown",
  "ownership": "Grupo ou descrição curta"
}
```

> **Aviso:** os ratings deste repositório são uma **curadoria demonstrativa** para o MVP. Não são uma base licenciada da AllSides, Ad Fontes Media ou MBFC e não devem ser apresentados como avaliação oficial. Veja [`../../THIRD_PARTY_NOTICES.md`](../../THIRD_PARTY_NOTICES.md).

Para produção, substitua por uma base licenciada ou por metodologia própria auditável. Documente, no mínimo:

- quem avaliou e com qual método;
- data da avaliação e versão da metodologia;
- margem de erro ou confiança;
- como correções e contestações são tratadas.

O código trata veículos ausentes como `unknown`; “sem avaliação” é um estado visível, não um erro.

## `feeds.json`

Lista de feeds RSS/Atom curados usados além de Google News, Bing News e GDELT. Inclui nome, URL e, quando aplicável, dicas de parsing. Ao adicionar um feed:

1. Confirme que o feed é público e permite uso.
2. Verifique se as URLs dos artigos são diretas (sem redirect de agregador).
3. Respeite `robots.txt` e limites de requisição.
4. Não inclua conteúdo pago, credenciais ou parâmetros privados.

## Dados que não entram no repositório

- Texto integral de matérias.
- Bases de ratings proprietárias.
- Chaves, tokens, cookies ou credenciais.
- Bancos SQLite locais (`*.db` estão no `.gitignore`).
