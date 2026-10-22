#!/usr/bin/env bash
# Registra o Mirante no Nginx Proxy Manager do host de deploy.
# Roda no host, depois do docker compose up.
set -euo pipefail

: "${MIRANTE_DOMAIN:?defina MIRANTE_DOMAIN (ex.: news.example.com)}"
DOMAIN="$MIRANTE_DOMAIN"
FORWARD_HOST="${MIRANTE_CONTAINER:-mirante-web}"
FORWARD_PORT="${MIRANTE_PORT:-8091}"
NPM_BASE="${NPM_BASE:-http://localhost:81}"
NPM_CONTAINER="${NPM_CONTAINER:-nginx_proxy_manager}"

# Credenciais do Nginx Proxy Manager: exporte NPM_EMAIL e NPM_PASSWORD ou
# aponte NPM_ENV_FILE para um arquivo fora do repositório (ex.: ~/.config/mirante/npm.env).
if [ -z "${NPM_EMAIL:-}" ] || [ -z "${NPM_PASSWORD:-}" ]; then
  if [ -n "${NPM_ENV_FILE:-}" ] && [ -f "${NPM_ENV_FILE}" ]; then
    set -a
    # shellcheck disable=SC1090
    . "${NPM_ENV_FILE}"
    set +a
  fi
fi

: "${NPM_EMAIL:?NPM_EMAIL ausente no host}"
: "${NPM_PASSWORD:?NPM_PASSWORD ausente no host}"

echo "[npm] login ${NPM_BASE}"
TOKEN="$(curl -sS -m 20 -X POST "${NPM_BASE}/api/tokens" \
  -H "Content-Type: application/json" \
  -d "{\"identity\":\"${NPM_EMAIL}\",\"secret\":\"${NPM_PASSWORD}\"}" \
  | python3 -c "import sys,json;print(json.load(sys.stdin).get('token',''))")"
[ -n "$TOKEN" ] || { echo "[npm] erro: login falhou" >&2; exit 1; }

AUTH=(-H "Authorization: Bearer ${TOKEN}")

echo "[npm] certificado para ${DOMAIN}"
CERT_ID="$(curl -sS -m 20 "${AUTH[@]}" "${NPM_BASE}/api/nginx/certificates" | python3 -c "
import sys,json
d=json.load(sys.stdin)
items=d if isinstance(d,list) else d.get('certificates',d)
for c in items:
    if isinstance(c,dict) and '${DOMAIN}' in (c.get('domain_names') or []):
        print(c.get('id','')); break
" 2>/dev/null || true)"

if [ -n "$CERT_ID" ]; then
  echo "[npm] certificado existente (id ${CERT_ID})"
else
  RESP="$(curl -sS -m 120 -X POST "${AUTH[@]}" "${NPM_BASE}/api/nginx/certificates" \
    -H "Content-Type: application/json" \
    -d "{\"provider\":\"letsencrypt\",\"nice_name\":\"${DOMAIN}\",\"domain_names\":[\"${DOMAIN}\"],\"meta\":{\"dns_challenge\":false}}")"
  CERT_ID="$(echo "$RESP" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('id','') if isinstance(d,dict) else '')" 2>/dev/null || true)"
  [ -n "$CERT_ID" ] || { echo "[npm] erro: certificado não emitido: ${RESP}" >&2; exit 1; }
  echo "[npm] certificado emitido (id ${CERT_ID})"
fi

PAYLOAD="$(cat <<EOF
{
  "domain_names": ["${DOMAIN}"],
  "forward_scheme": "http",
  "forward_host": "${FORWARD_HOST}",
  "forward_port": ${FORWARD_PORT},
  "certificate_id": ${CERT_ID},
  "ssl_forced": true,
  "http2_support": true,
  "hsts_enabled": false,
  "block_exploits": true,
  "allow_websocket_upgrade": false,
  "caching_enabled": false,
  "enabled": true,
  "access_list_id": 0,
  "meta": {},
  "advanced_config": "",
  "locations": [],
  "hsts_subdomains": false
}
EOF
)"

HOST_ID="$(curl -sS -m 20 "${AUTH[@]}" "${NPM_BASE}/api/nginx/proxy-hosts" | python3 -c "
import sys,json
d=json.load(sys.stdin)
items=d if isinstance(d,list) else d.get('hosts',d)
for h in items:
    if isinstance(h,dict) and '${DOMAIN}' in (h.get('domain_names') or []):
        print(h.get('id','')); break
" 2>/dev/null || true)"

if [ -n "$HOST_ID" ]; then
  echo "[npm] proxy host existente (id ${HOST_ID}); atualizando"
  curl -sS -m 30 -X PUT "${AUTH[@]}" "${NPM_BASE}/api/nginx/proxy-hosts/${HOST_ID}" \
    -H "Content-Type: application/json" -d "$PAYLOAD" >/dev/null
else
  RESP="$(curl -sS -m 30 -X POST "${AUTH[@]}" "${NPM_BASE}/api/nginx/proxy-hosts" \
    -H "Content-Type: application/json" -d "$PAYLOAD")"
  HOST_ID="$(echo "$RESP" | python3 -c "import sys,json;d=json.load(sys.stdin);print(d.get('id','') if isinstance(d,dict) else '')" 2>/dev/null || true)"
  [ -n "$HOST_ID" ] || { echo "[npm] erro: proxy host recusado: ${RESP}" >&2; exit 1; }
  echo "[npm] proxy host criado (id ${HOST_ID})"
fi

docker exec "$NPM_CONTAINER" nginx -s reload >/dev/null 2>&1 || true

echo "[npm] verificando https://${DOMAIN}/api/health"
CODE=""
for _ in 1 2 3 4 5 6 7 8; do
  CODE="$(curl -sS -o /tmp/mirante-health.json -w '%{http_code}' -m 15 "https://${DOMAIN}/api/health" 2>/dev/null || true)"
  [ "$CODE" = "200" ] && break
  sleep 5
done
[ "$CODE" = "200" ] || { echo "[npm] erro: https não respondeu (HTTP ${CODE:-sem resposta})" >&2; exit 1; }
echo "[npm] OK · https://${DOMAIN}"
cat /tmp/mirante-health.json
echo
