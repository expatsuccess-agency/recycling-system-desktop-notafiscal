#!/usr/bin/env bash
set -Eeuo pipefail

ROOT="/media/user/skllnxs.cloud/Projets/LXO"

DESKTOP="$ROOT/V8/recycling-system-v8/desktop"
NOTAFISCAL="$ROOT/V8-NotaFiscal/recycling-system-v8-notafiscal/desktop"

echo "============================================"
echo " RECYCLING SYSTEM — GITHUB PUSH"
echo " DESKTOP V8 + NOTAFISCAL V8.2"
echo "============================================"

if [[ "$(id -u)" -eq 0 ]]; then
    echo "ERREUR : ne pas exécuter en root."
    exit 1
fi

push_project() {
    local PROJECT="$1"
    local NAME="$2"
    local REPO="$3"
    local MESSAGE="$4"

    echo ""
    echo "============================================"
    echo " PROJET : $NAME"
    echo "============================================"

    cd "$PROJECT"

    if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        echo "ERREUR : dépôt Git non initialisé."
        return 1
    fi

    echo "=== 1. DÉPÔT DISTANT ==="

    git remote -v

    local URL
    URL="$(git remote get-url origin)"

    if [[ "$URL" != *"$REPO"* ]]; then
        echo "ERREUR : dépôt distant inattendu : $URL"
        return 1
    fi

    echo "=== 2. BRANCHE ==="

    local BRANCH
    BRANCH="$(git branch --show-current)"

    if [[ -z "$BRANCH" ]]; then
        echo "ERREUR : HEAD détachée."
        return 1
    fi

    echo "Branche : $BRANCH"

    echo "=== 3. EXCLUSIONS LOCALES ==="

    # Exclusions Git locales, sans modifier le .gitignore
    # versionné du projet.
    mkdir -p "$(git rev-parse --git-path info)"

    cat >> "$(git rev-parse --git-path info/exclude)" <<'EOF'

# Recycling System - fichiers locaux
node_modules/
target/
src-tauri/target/
.cache/
.pytest_cache/
__pycache__/
.ui-patch-backups/
*.backup-*
*.bak
*.log
*.zip
*.pfx
*.p12
*.pem
*.key
*.lic
*.sqlite
*.sqlite3
*.db
*.db-wal
*.db-shm
.env
.env.*
EOF

    echo "=== 4. AJOUT DES SOURCES ==="

    git add -A

    echo "=== 5. FICHIERS INDEXÉS ==="

    git diff --cached --stat
    git diff --cached --name-only

    echo "=== 6. CONTRÔLE DES FICHIERS SENSIBLES ==="

    local SENSITIVE
    SENSITIVE="$(
        git diff --cached --name-only |
        grep -Ei '(^|/)\.env($|\.)|\.pfx$|\.p12$|\.pem$|\.key$|\.lic$|\.sqlite|\.db($|-wal$|-shm$)|backup-|\.log$|\.zip$' \
        || true
    )"

    if [[ -n "$SENSITIVE" ]]; then
        echo "ERREUR : fichiers sensibles indexés :"
        echo "$SENSITIVE"
        echo "Retire-les de l'index avec git restore --staged."
        return 1
    fi

    if git diff --cached --quiet; then
        echo "Aucune nouvelle modification à enregistrer."
    else
        echo ""
        echo "ATTENTION : examine aussi le contenu du diff."
        git --no-pager diff --cached --check

        echo ""
        read -r -p "Valider le commit de $NAME ? (oui/non) : " CONFIRM

        if [[ "$CONFIRM" != "oui" ]]; then
            echo "Commit annulé pour $NAME."
            return 1
        fi

        git commit -m "$MESSAGE"
    fi

    echo "=== 7. SYNCHRONISATION DISTANTE ==="

    git fetch origin

    # Refuser un push si la branche distante contient des
    # commits absents de la branche locale.
    if git show-ref --verify --quiet "refs/remotes/origin/$BRANCH"; then
        if ! git merge-base --is-ancestor "origin/$BRANCH" HEAD; then
            echo "ERREUR : la branche GitHub contient des commits"
            echo "qui ne sont pas encore intégrés localement."
            echo "Aucun force push effectué."
            return 1
        fi
    fi

    echo "=== 8. PUSH GITHUB ==="

    git push -u origin "$BRANCH"

    echo "=== 9. VÉRIFICATION ==="

    git status -sb
    git log -1 --oneline

    echo "PUSH TERMINÉ : $NAME"
}

push_project \
    "$DESKTOP" \
    "Recycling System Desktop V8" \
    "recycling-system-desktop" \
    "feat(desktop): improve Vendas keyboard navigation and F12"

push_project \
    "$NOTAFISCAL" \
    "Recycling System NotaFiscal RJ V8.2" \
    "recycling-system-desktop-notafiscal" \
    "feat(notafiscal): improve fiscal UI and Vendas workflow"

echo ""
echo "============================================"
echo " LES DEUX DÉPÔTS ONT ÉTÉ SYNCHRONISÉS"
echo "============================================"
