# Recycling System Industrial V6.1.0 — Caixa, Vendas et Rentabilidade

Cette version reprend V6.0.0 et conserve le format SQLite, les droits admin/funcionário, la licence Ed25519 locale et les données existantes.

## Fonctionnalités

1. **Caixa** : touche `Entrée` dans les formulaires de caisse = valider, sans indication `Ctrl+Entrée`. Les numéros de caisse peuvent être saisis dans le champ « Número do caixa (opcional) », vide = auto-attribution. Les numéros déjà utilisés ne peuvent pas être réattribués.
2. **Pesagem** : chaque ligne affiche le poids et le prix/kg, en plus du montant.
3. **Vendas** : catalogue partagé avec les achats, prix de vente/kg saisi indépendamment du prix d'achat, plusieurs matériaux dans une vente, entreprise acheteuse facultative, paiement enregistrable dans Caixa.
4. **Histórico de vendas** : filtre par date, société acheteuse, produit, poids, prix/kg et total dans les détails. La liste indique la durée écoulée depuis la vente immédiatement précédente, même si elle est hors du filtre courant.
5. **Rentabilidade** : coût d'acquisition moyen pondéré calculé à partir des achats enregistrés au moment de la vente et figé dans chaque ligne ; sinon coût d'acquisition de référence si renseigné. Marge brute = recette de vente moins coûts connus. Si le coût manque, la marge et le résultat net sont affichés « — » pour éviter un bénéfice fictif.

**Limites à valider en situation réelle :** la version ne gère pas encore un inventaire physique par lots/FIFO ni une facture fiscale électronique du Brésil (NF-e). Les coûts d'acquisition sont une estimation moyenne pondérée ; ils ne remplacent pas une valorisation comptable validée. Le nom de l'entreprise acheteuse est conservé comme champ texte, pas un registre CNPJ complet.

## Précautions

- Fermer la V6.0 avant la migration, sauvegarder la base `lxo.sqlite` (ainsi que les fichiers WAL/SHM éventuels si l'application n'est pas fermée).
- Ne jamais copier la clé privée Ed25519 dans les sources distribuées.
- L'archive possède une clé publique exemple `src-tauri/src/license_public_key.hex` (64 zéros). **La remplacer par ta clé publique réelle existante**, par exemple :

```bash
cp "$HOME/.local/share/recycling-license-manager/public-key.hex" \
   src-tauri/src/license_public_key.hex
```

## Démarrer sous Parrot Linux

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V6.1/recycling-system-v5/desktop
# La première fois seulement, ou si package-lock/deps ont changé :
npm install
npm test
npm run tauri:dev
```

## AppImage Linux

Après vérification du parcours complet et des données :

```bash
bash build-appimage-linux.sh
```

L'AppImage généré se trouve normalement sous `src-tauri/target/release/bundle/appimage/`, ou est copié dans `linux-build/` par le script.

## Recette minimale

- Caisse : ouvrir par `Entrée`, avec numéro libre ou automatique ; vérifier collision de numéro ; fermer par `Entrée`.
- Pesagem : acheter deux matières, vérifier poids/prix/kg et caisse pour un paiement en espèces.
- Vendas : ajouter deux matériaux, fixer deux prix de vente/kg différents de ceux d'achat, choisir une entreprise, puis confirmer la vente et le paiement.
- Vendas : relire les deux ventes dans l'historique et vérifier l'intervalle entre elles, les quantités, les coûts et la marge brute.
- Gestão Financeira : confirmer la cohérence des marges par produit et du résultat ; vérifier le cas « custo incompleto ».
- Vérifier l'impossibilité d'accéder aux ventes ou aux données financières en tant que `funcionário`.

## Vérifications réalisées dans l'environnement de préparation

- 45 tests JavaScript de structure et de logique source réussis ;
- Analyse syntaxique TSX : 0 erreur ;
- Requête d'historique SQLite : résultat contrôlé sur données fictives.

**Non vérifié ici** : compilation complète Rust/Tauri, lancement de l'application, intégrité après migration sur une base de production, ou génération AppImage. Ces vérifications doivent être faites sur Parrot Linux avant distribution.
