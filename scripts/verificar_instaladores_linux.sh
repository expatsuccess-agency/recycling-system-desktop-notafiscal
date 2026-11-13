#!/usr/bin/env bash
set -Eeuo pipefail
ROOT="${1:-src-tauri/target/release/bundle}"
[[ -d "$ROOT" ]] || { echo "SEM_BUNDLE_LINUX: $ROOT";exit 2; }
shopt -s nullglob globstar
FILES=("$ROOT"/**/*.AppImage "$ROOT"/**/*.deb)
[[ ${#FILES[@]} -gt 0 ]] || { echo 'INSTALADOR_LINUX_NAO_ENCONTRADO';exit 2; }
for f in "${FILES[@]}";do
 echo "ARQUIVO: $f";sha256sum "$f";test -s "$f"
 if [[ "$f" == *.deb ]];then dpkg-deb --info "$f" | head -35;fi
done
echo 'PACOTES_LINUX_ENCONTRADOS; instalação real e restauração requerem ambiente de testes isolado.'
