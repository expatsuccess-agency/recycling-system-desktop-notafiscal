#!/usr/bin/env bash
set -Eeuo pipefail

SOURCE="/media/user/skllnxs.cloud/Projets/LXO/V8/recycling-system-v8/desktop"

# Destination personnalisée ou dossier voisin horodaté
DESTINATION="${1:-$(dirname "$SOURCE")/desktop-copie-$(date +%Y%m%d-%H%M%S)}"

if [[ ! -d "$SOURCE" ]]; then
    echo "ERREUR : dossier source introuvable : $SOURCE" >&2
    exit 1
fi

if ! command -v rsync >/dev/null 2>&1; then
    echo "Installe rsync : sudo apt install rsync" >&2
    exit 1
fi

SOURCE="$(realpath "$SOURCE")"
DESTINATION="$(realpath -m "$DESTINATION")"

# Éviter une copie dans le dossier source
case "$DESTINATION/" in
    "$SOURCE/"*) echo "ERREUR : destination dans la source" >&2; exit 1 ;;
esac

mkdir -p "$DESTINATION"

echo "SOURCE : $SOURCE"
echo "DESTINATION : $DESTINATION"

rsync -avh --progress \
    --exclude='/node_modules/' \
    --exclude='/src-tauri/target/' \
    "$SOURCE/" "$DESTINATION/"

echo ""
echo "Copie terminée."
echo "Destination : $DESTINATION"
