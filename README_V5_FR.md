# Recycling System Industrial V5.0 — Linux (Tauri + React + Rust)

Cette archive reprend **Recycling System V4.0.1** et conserve **l'identifiant Tauri `br.com.lxo.desktop`**, donc le même emplacement de données pour une installation du même utilisateur Linux. **Ne pas supprimer les données de V4**.

## 1. Changements

### Pesagem
- `Entrée` dans `Peso (kg)` : ajoute la ligne courante au panier, efface les champs de la ligne et place le focus dans `Produto`.
- Zone `Cliente` supprimée de **Pesagem** (conservée sur les autres opérations où elle est utile).
- Champ de modification de `Preço por kg` visible seulement si le produit autorise le changement de prix.
- `F12` ouvre le choix de paiement pour valider la **compra**.
- Icône `Vendas` remplacée par une flèche de sortie.

### Comptes et permissions
- Premier utilisateur `admin` avec mot de passe initial `admin`, **changement obligatoire au premier accès** avant toute opération. Nouveau mot de passe : exactement 8 chiffres.
- L'administrateur peut gérer les mots de passe et créer des utilisateurs `admin` et `employee` (`Funcionário`). Une première modification du mot de passe temporaire est obligatoire pour chaque nouvel utilisateur.
- `Funcionário` : accès uniquement à **Pesagem**. Les autres menus sont grisés et les opérations correspondantes sont **refusées dans Rust**. Les articles retournés au salarié excluent les coûts d'acquisition.
- L'administrateur doit **ouvrir Caixa** avant que les employés puissent enregistrer des pesées payées.
- Les sessions sont conservées uniquement en mémoire du processus ; il faut se reconnecter après fermeture de l'application.

### Finance (administrateur uniquement)
- Menu `Gestão Financeira` avec raccourci `F8`.
- Configuration produit : prix d'achat/kg, prix de vente suggéré/kg et coût d'acquisition estimé/kg.
- Les opérations `Vendas` conservent au moment de la vente un **coût unitaire estimé figé** : moyenne pondérée des achats précédemment enregistrés, sinon coût configuré, sinon coût inconnu.
- Rapports jour/semaine/mois : chiffre d'affaires, coût des marchandises vendues estimé, bénéfice brut, dépenses et résultat net estimé.
- Marges par produit. Une ligne sans coût connu est marquée **incomplète**, et non arbitrairement considérée comme profitable.
- Catégories de dépenses personnalisées et saisie de charges opérationnelles.
- Les rapports sont indicatifs (ni bilan comptable légal ni documents fiscaux brésiliens).

## 2. Préparation et sauvegarde des données V4

1. Fermer l'application V4 complètement.
2. Identifier le chemin réel de la base : `find "$HOME/.local/share" -name lxo.sqlite -type f 2>/dev/null`.
3. Faire une sauvegarde SQLite **cohérente** (important si le mode WAL est actif), par exemple :

```bash
DB="/chemin/vers/lxo.sqlite"
mkdir -p "$HOME/Sauvegardes/RecyclingSystem"
sqlite3 "$DB" ".backup '$HOME/Sauvegardes/RecyclingSystem/lxo-avant-v5.sqlite'"
```

4. Conserver également une copie hors ligne de la **clé privée** du License Manager, sans l'ajouter au projet applicatif.

> La V5 ajoute des tables/colonnes de manière additive lors du premier lancement. Une base ouverte sous V5 ne doit pas être réutilisée en écriture par une ancienne version sans vérification.

## 3. Conserver la même clé publique que V4

L'archive source contient un emplacement `src-tauri/src/license_public_key.hex` avec une valeur neutre composée de zéros. **Ce n'est pas une clé de production.** Il faut y placer la clé publique de la paire Ed25519 déjà utilisée par Recycling License Manager V1, sans créer de nouvelle paire.

Si la V4.0.1 fonctionne avec ses licences et que la clé publique est dans son projet :

```bash
OLD="/media/user/skllnxs.cloud/Projets/LXO/V4/V4.0.1/recycling-system-v4/desktop"
NEW="/media/user/skllnxs.cloud/Projets/LXO/V5.0/recycling-system-v5/desktop"
cp "$OLD/src-tauri/src/license_public_key.hex" "$NEW/src-tauri/src/license_public_key.hex"
```

Vérifier qu'il y a exactement 64 caractères hexadécimaux **et que la valeur n'est pas composée uniquement de zéros**. Ne jamais copier la clé privée dans l'application cliente.

## 4. Démarrer sous Parrot Linux

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V5.0/recycling-system-v5/desktop
npm install
npm test
npm run tauri:dev
```

À la première ouverture : licence V4 valide, puis utilisateur `admin` / mot de passe initial `admin`, changement obligatoire, ouverture de `Caixa` en admin, création des employés dans `Configuração`.

Les commandes npm sont à lancer comme utilisateur normal (pas `root`).

## 5. Construire l'AppImage

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V5.0/recycling-system-v5/desktop
bash build-appimage-linux.sh
```

Tauri construit en mode release et tente le packaging AppImage. En cas d'erreur « couldn't find a square icon », régénérer les icônes Tauri carrées et vérifier `bundle.icon` dans `src-tauri/tauri.conf.json`. Le script peut demander des dépendances système sur Parrot Linux.

Le résultat, si la compilation réussit, se trouve normalement dans `src-tauri/target/release/bundle/appimage/` ou dans `linux-build/` si le script copie les artefacts.

## 6. Limites et validations restantes

- **Compilation complète Rust/Tauri et AppImage non exécutées dans l'environnement de génération de l'archive** ; exécuter `cargo check` et `npm run tauri:dev` sur Parrot Linux.
- Protéger les clés privées Ed25519 dans License Manager ; l'archive V5 ne les contient pas.
- Authentification locale : Argon2id avec sel aléatoire et sessions volatiles ; elle réduit les usages non autorisés, mais ne protège pas une machine contre un administrateur système malveillant qui modifierait le programme.
- Les coûts historiques de V4 ne sont pas renseignés automatiquement : ils sont déclarés incomplets. Ne pas utiliser les marges de la V5 à des fins comptables avant rapprochement des stocks et des coûts.
- Les tests sont des contrôles de structure et des requêtes financières représentatives. Valider les parcours métier, les permissions et la migration de données réelles sur ton environnement.
