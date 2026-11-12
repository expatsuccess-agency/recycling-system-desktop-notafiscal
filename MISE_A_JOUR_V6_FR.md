# Recycling System Industrial V6.0.0 — Mise à jour Linux

## Modifications
- Dans **Configuração → Produtos e Categorias**, la liste des produits est déplacée immédiatement sous le titre **Produtos e Categorias**, avant les champs de saisie commençant par **Código**.
- La grille de boutons sous **Ícone do material** est supprimée. Le sélecteur d’icônes est plus étroit (112 px), avec des pictogrammes plus grands (42 px sur le sélecteur ; 34 px sur les options, selon le moteur d'affichage du système).
- Aucune migration du schéma SQLite ni changement de licence Ed25519.

## Installer
Sauvegarder la base et la clé publique avant de remplacer les sources. Copier **uniquement la clé publique existante** dans `src-tauri/src/license_public_key.hex` si l'archive contient la valeur par défaut. Ne jamais transférer `private-key.pem` dans les sources clientes.

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V6.0/recycling-system-v5/desktop
npm install
npm test
npm run tauri:dev
```

Pour l'AppImage, une fois validé :

```bash
bash build-appimage-linux.sh
```

## Repartir de zéro — BASE SQLITE UNIQUEMENT
**Attention, cette opération réinitialise aussi les comptes et les mots de passe**, ainsi que les produits, transactions, caisses, finances, historiques. Elle ne supprime PAS les fichiers de licence `.lic`, ni `license-location.json`.

1. Fermer complètement Recycling System, y compris les instances de développement.
2. Vérifier le chemin utilisé dans le code : `src-tauri/src/lib.rs` appelle `app.path().app_data_dir()` et ouvre `lxo.sqlite`. Avec l'identifiant `br.com.lxo.desktop`, sous Linux l'emplacement habituel est `~/.local/share/br.com.lxo.desktop/lxo.sqlite`. Confirmer l'emplacement sur le poste.
3. Exécuter `bash reset-base-sqlite.sh` et confirmer avec `RESET-SQLITE`. Le script déplace `lxo.sqlite` et, s'ils existent, les journaux `-wal` et `-shm` vers un dossier `backups/` horodaté, sans les effacer.
4. Vérifier que `LXO_IMPORT_SQLITE` n'est pas défini (`unset LXO_IMPORT_SQLITE`), puis redémarrer l'application. Une nouvelle base sera initialisée.
5. En cas de besoin, fermer l'application et restaurer ensemble les fichiers sauvegardés depuis le répertoire de backup.

**Ne pas supprimer `~/.local/share/br.com.lxo.desktop` dans son ensemble** : on risquerait de perdre la configuration de l'emplacement de la licence. Ne pas réinitialiser les clés Ed25519 pour vider la base.
