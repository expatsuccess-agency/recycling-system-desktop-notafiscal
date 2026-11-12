# Recycling System Industrial — V8.0.0

## Modifications
- Login amélioré : logo, nom, version **V8.0.0**, champs avec pictogrammes et œilleton ; indicateur de progression réel pendant la vérification des identifiants. Blocage des soumissions répétées pendant l’opération.
- Champ mot de passe : huit chiffres, conformément aux règles existantes. Aucun changement du hachage des mots de passe.
- Configuração > Produtos e Categorias : l’ancien `select` système d’emoji est remplacé par un sélecteur de pictogrammes à fond blanc, recherche par catégorie, grille de choix et fermeture par Échap ou clic à l’extérieur.
- Métadonnées d’application synchronisées : `package.json`, `Cargo.toml`, `tauri.conf.json` et labels UI.
- Nouveau `lancer-avec-cache.sh` : compilation Cargo centralisée, téléchargement npm mis en cache et copie de la clé publique existante si présente.
- Les opérations SQLite, les comptes, les droits et les licences sont conservés.

## Installation Parrot Linux
1. Extraire l’archive dans le dossier de version souhaité (par exemple `/media/user/skllnxs.cloud/Projets/LXO/V8/recycling-system-v8/desktop`).
2. Exécuter `bash lancer-avec-cache.sh` depuis `desktop`. La première compilation centralisée reste longue ; les suivantes réutilisent le cache.
3. Tester la connexion avec un compte existant ; l’animation s’affiche uniquement pendant l’attente de Rust.
4. Dans Configuração > Produtos e Categorias, ouvrir Ícone do material, essayer les filtres, sélectionner un pictogramme et enregistrer le produit.

## Points de vigilance
Ne distribuez jamais `private-key.pem`. Le script ne modifie ni la base de données ni les comptes. La génération de la clé publique n’est jamais déclenchée automatiquement. Les liens « mot de passe oublié » et « créer un compte » de la maquette n’ont pas été ajoutés au login, car leur autorisation et leur récupération sécurisée nécessitent des flux backend dédiés.
