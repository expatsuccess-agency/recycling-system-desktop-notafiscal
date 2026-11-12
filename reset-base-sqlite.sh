#!/usr/bin/env bash
set -Eeuo pipefail
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/br.com.lxo.desktop"
DB="$APP_DIR/lxo.sqlite"
BACKUPS="$APP_DIR/backups"
echo "=== RECYCLING SYSTEM — REINITIALISATION SQLITE ==="
echo "Fichier : $DB"
echo "ATTENTION : tous les clients, produits, ventes, caisses, utilisateurs et depenses seront reinitialises."
echo "Les fichiers .lic, la configuration du chemin de licence et les cles ne seront pas modifies."
if [[ ! -f "$DB" ]]; then
  echo "Aucune base SQLite a cet emplacement. Verifiez XDG_DATA_HOME / la configuration Tauri."
  exit 1
fi
if pgrep -u "$(id -u)" -f '(^|/)(lxo-industrial-pos|recycling-system)( |$)' >/dev/null 2>&1; then
  echo "ERREUR : l'application semble ouverte. Fermez-la completement avant de continuer."
  exit 1
fi
read -r -p "Pour confirmer, saisissez exactement RESET-SQLITE : " ans
[[ "$ans" == 'RESET-SQLITE' ]] || { echo 'Annule.'; exit 1; }
mkdir -p "$BACKUPS"
STAMP="$(date +%Y%m%d-%H%M%S)"
DEST="$BACKUPS/lxo-before-reset-$STAMP"
mkdir -m 700 "$DEST"
for ext in '' '-wal' '-shm'; do
  if [[ -e "$DB$ext" ]]; then mv -- "$DB$ext" "$DEST/lxo.sqlite$ext"; fi
done
chmod 700 "$DEST"
echo "Base sauvegardee dans : $DEST"
echo "Au prochain demarrage, Recycling System recreera une base vide."
echo "Important : ne definissez pas LXO_IMPORT_SQLITE lors du premier redemarrage."
