# Política de segurança

## Versões suportadas

O projeto está em fase de MVP. Apenas a versão mais recente em `main` recebe correções de segurança.

| Versão | Suporte |
| --- | --- |
| 0.1.x | Sim |
| < 0.1 | Não |

## Como reportar uma vulnerabilidade

**Não abra uma issue pública.** Use o canal privado de segurança do GitHub:

1. Acesse <https://github.com/gustavomello9600/mirante/security/advisories/new>.
2. Descreva o impacto, os passos de reprodução e, se possível, uma prova de conceito mínima.
3. Informe a versão ou o commit afetado e o ambiente (Docker, local, navegador).

Você receberá uma confirmação em até **72 horas**. Vamos trabalhar com você para validar o problema, preparar a correção e coordenar a divulgação. Pedimos que não divulgue detalhes publicamente antes de uma correção ou de um prazo combinado.

## Escopo

São especialmente relevantes:

- Exposição de segredos, chaves de LLM ou dados de configuração.
- SSRF, path traversal ou leitura de arquivos locais pela API ou pela leitura profunda.
- Injeção de prompt com impacto real (execução de ações, vazamento de dados ou citações inventadas).
- XSS, injeção de HTML ou manipulação de links no frontend.
- Bypass de validação de citações ou de limites de amostra.
- Negação de serviço por uso ilimitado de recursos ou custo de LLM.
- Falhas de CORS, cache ou isolamento entre temas/usuários.

Fora do escopo:

- Falta de autenticação ou rate limit por IP no MVP (limitação conhecida, documentada).
- Conteúdo editorial ou avaliações de viés/factualidade de demonstração.
- Ataques que dependem de acesso físico ao host ou de credenciais já comprometidas.
- Relatórios automatizados sem impacto demonstrável.

## Boas práticas para quem opera

- Mantenha `.env` fora do Git, com permissão `600`, e rotacione a chave se houver qualquer suspeita de exposição.
- Rode atrás de um proxy reverso com TLS, limite de requisições e timeout.
- Restrinja CORS ao domínio do frontend em produção.
- Defina `MIRANTE_DEBUG=false` em produção.
- Monitore custo e volume de chamadas ao provedor de LLM.
- Mantenha a imagem base e as dependências atualizadas; o Dependabot está configurado.
- Faça backup do volume SQLite e trate o banco como dado interno.

## Rotação de chaves

Se uma chave de LLM for commitada, colada em uma issue ou exposta em log:

1. Revogue a chave no painel do provedor imediatamente.
2. Gere uma nova chave e atualize apenas o `.env` do host.
3. Remova a exposição do histórico com uma ferramenta como `git filter-repo` e faça force-push.
4. Verifique os logs de uso do provedor para atividade não autorizada.
5. Se o repositório for público, considere a chave comprometida mesmo que o commit seja removido.
