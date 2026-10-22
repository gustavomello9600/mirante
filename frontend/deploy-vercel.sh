#!/usr/bin/env bash
# Build estático do frontend (WASM + assets) e deploy opcional no Vercel.
#
# Uso:
#   MIRANTE_API_BASE=https://api.example.com ./deploy-vercel.sh
#
# Variáveis:
#   MIRANTE_API_BASE  URL pública do backend (vazio = mesma origem)
#   VERCEL_PROJECT    nome do projeto no Vercel (padrão: mirante)
set -euo pipefail
cd "$(dirname "$0")"

API_BASE="${MIRANTE_API_BASE:-}"
PROJECT="${VERCEL_PROJECT:-mirante}"

if [[ "$API_BASE" == *'"'* ]]; then
  echo "erro: MIRANTE_API_BASE não pode conter aspas" >&2
  exit 1
fi

if [ -z "$API_BASE" ]; then
  echo "aviso: MIRANTE_API_BASE vazio; o frontend usará a mesma origem. Em um host estático sem proxy, defina a URL pública do backend." >&2
fi

wasm-pack build --target web --release --no-opt --out-dir dist/pkg
cp index.html dist/
cp config.js dist/
cp -R styles/. dist/styles/
cp -R fonts/. dist/fonts/
cp vercel.json dist/vercel.json

printf 'window.MIRANTE_API_BASE = "%s";\n' "$API_BASE" > dist/config.js

npx --yes vercel deploy --prod --yes --create-if-missing --project "$PROJECT" dist "$@"
