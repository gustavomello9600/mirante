#!/usr/bin/env bash
# Deploy do Mirante em um host com Docker Compose e Nginx Proxy Manager.
#
# Variáveis obrigatórias:
#   MIRANTE_HOST        destino SSH (ex.: deploy@news.example.com)
#   MIRANTE_DOMAIN      domínio público (ex.: news.example.com)
#
# Variáveis opcionais:
#   MIRANTE_SSH_KEY     chave SSH (padrão: ~/.ssh/id_ed25519)
#   MIRANTE_DEPLOY_DIR  diretório remoto (padrão: mirante, relativo ao home)
#
# O script sincroniza o código, reconstrói a imagem no host, sobe a stack e
# executa um smoke test local e público. Nenhum segredo é enviado: o arquivo
# .env permanece apenas no host e é excluído do rsync.
set -euo pipefail

: "${MIRANTE_HOST:?defina MIRANTE_HOST (ex.: deploy@news.example.com)}"
: "${MIRANTE_DOMAIN:?defina MIRANTE_DOMAIN (ex.: news.example.com)}"

HOST="$MIRANTE_HOST"
KEY="${MIRANTE_SSH_KEY:-$HOME/.ssh/id_ed25519}"
DIR="${MIRANTE_DEPLOY_DIR:-mirante}"
DOMAIN="$MIRANTE_DOMAIN"

if [ ! -f "$KEY" ]; then
  echo "erro: chave SSH não encontrada em $KEY" >&2
  exit 1
fi

SSH=(ssh -i "$KEY" -o StrictHostKeyChecking=accept-new -o ConnectTimeout=15)
RSYNC=(rsync -az --delete \
  --exclude target --exclude dist --exclude .vercel --exclude .git \
  --exclude .env --exclude '.env.*' \
  -e "ssh -i $KEY -o StrictHostKeyChecking=accept-new")

echo "== 1) sincronizando fonte para $HOST:$DIR"
"${RSYNC[@]}" ./ "$HOST:$DIR/"

echo "== 2) build no host e up"
"${SSH[@]}" "$HOST" "cd '$DIR' && docker compose build && docker compose up -d"

echo "== 3) smoke local no host"
"${SSH[@]}" "$HOST" "curl -fsS http://127.0.0.1:8091/api/health && echo && curl -fsS -o /dev/null -w 'home %{http_code}\\n' http://127.0.0.1:8091/"

echo "== 4) Nginx Proxy Manager + TLS"
"${SSH[@]}" "$HOST" "cd '$DIR' && bash deploy/register-npm.sh"

echo "== 5) smoke público"
curl -fsS -m 20 "https://${DOMAIN}/api/health"
echo
curl -fsS -o /dev/null -w "https://${DOMAIN} -> %{http_code}\n" -m 20 "https://${DOMAIN}/"
