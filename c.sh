#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="/media/user/skllnxs.cloud/Projets/LXO"

if [[ "$(id -u)" -eq 0 ]]; then
    echo "ERREUR : ne pas exécuter en root."
    exit 1
fi

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

for VARIANT in desktop notafiscal; do

    if [[ "$VARIANT" == "desktop" ]]; then
        PROJECT="$ROOT/V8/recycling-system-v8/desktop"
        ZIPNAME="RECYCLING_V8_DESKTOP_CODIGO_VENDAS_ATALHOS_2026.zip"
    else
        PROJECT="$ROOT/V8-NotaFiscal/recycling-system-v8-notafiscal/desktop"
        ZIPNAME="RECYCLING_V8_NOTAFISCAL_CODIGO_VENDAS_ATALHOS_2026.zip"
    fi

    echo ""
    echo "========================================"
    echo " INSTALLATION : $VARIANT"
    echo "========================================"

    ZIP=""

    for P in \
        "$HOME/Téléchargements/$ZIPNAME" \
        "$HOME/Downloads/$ZIPNAME" \
        "$ROOT/$ZIPNAME"
    do
        if [[ -f "$P" ]]; then
            ZIP="$P"
            break
        fi
    done

    if [[ -z "$ZIP" ]]; then
        echo "ERREUR : archive absente : $ZIPNAME"
        exit 1
    fi

    if [[ ! -d "$PROJECT/.git" ]]; then
        echo "ERREUR : dépôt Git introuvable : $PROJECT"
        exit 1
    fi

    echo "=== VERIFICATION ARCHIVE ==="
    unzip -tq "$ZIP"

    echo "=== EXTRACTION ==="
    mkdir -p "$WORK/$VARIANT"
    unzip -q "$ZIP" -d "$WORK/$VARIANT"

    echo "=== APPLICATION ET TESTS ==="

    bash "$WORK/$VARIANT/INSTALL.sh" \
        "$VARIANT" \
        "$PROJECT"

    echo "=== $VARIANT TERMINE ==="
done

echo ""
echo "========================================"
echo " LES DEUX PATCHES SONT INSTALLES"
echo "========================================"
