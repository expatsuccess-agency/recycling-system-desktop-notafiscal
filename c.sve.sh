#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="/media/user/skllnxs.cloud/Projets/LXO"
PROJECT="$ROOT/V8-NotaFiscal/recycling-system-v8-notafiscal/desktop"
NAME="RECYCLING_NOTAFISCAL_RJ_5_ABAS_PREFLIGHT.zip"

if [[ "$(id -u)" -eq 0 ]]; then
  echo "ERREUR : utiliser le compte user."
  exit 1
fi

ZIP=""
for P in \
  "$HOME/Téléchargements/$NAME" \
  "$HOME/Downloads/$NAME" \
  "$ROOT/V8-NotaFiscal/$NAME"
do
  if [[ -f "$P" ]]; then
    ZIP="$P"
    break
  fi
done

if [[ -z "$ZIP" ]]; then
  echo "Archive introuvable."
  exit 1
fi

cd "$PROJECT"

echo "=== Vérifier les modifications locales ==="
git status --short

echo "=== Vérifier le ZIP ==="
unzip -t "$ZIP"

echo "=== Extraire le correctif ==="
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
unzip -q "$ZIP" -d "$WORK"

echo "=== Installer et tester ==="
bash "$WORK/fiscal_tabs_patch/INSTALAR_E_TESTAR.sh"

echo "=== État final ==="
git status --short

echo "Installation terminée."
echo "Aucune NF-e transmise."
