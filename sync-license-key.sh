#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
SRC="${RECYCLING_PUBLIC_KEY:-$HOME/.local/share/recycling-license-manager/public-key.hex}"
DST="src-tauri/src/license_public_key.hex"
if [[ ! -f "$SRC" ]]; then
  echo "ERRO: chave pública não encontrada em: $SRC" >&2
  exit 1
fi
python3 - "$SRC" <<'PY'
import sys,re,pathlib
text=pathlib.Path(sys.argv[1]).read_text().strip()
if not re.fullmatch('[0-9a-fA-F]{64}',text) or set(text)=={'0'}:
    sys.exit('ERRO: chave pública Ed25519 inválida')
print('Chave pública validada')
PY
cp -- "$SRC" "$DST"
echo "Chave pública sincronizada (não copia a chave privada)."
