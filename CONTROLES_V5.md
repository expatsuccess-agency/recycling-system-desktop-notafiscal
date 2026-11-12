# Recette de mise en production V5

- [ ] Licence V4.0.1 inchangée et acceptée
- [ ] Mot de passe admin `admin` changé au premier login
- [ ] Nouveau mot de passe trop court refusé
- [ ] Création d'un utilisateur `Funcionário` et changement de son mot de passe
- [ ] Tous les menus sauf Pesagem sont grisés pour Funcionário
- [ ] Appel direct à `finance:report`, `clients:list` ou `caixa:current` par Funcionário refusé côté Rust
- [ ] Pesagem : Entrée dans Peso (kg) ajoute la ligne et place le focus dans Produto
- [ ] Produit non autorisé : modification de prix au kg refusée côté Rust
- [ ] Produit autorisé : modification de prix au kg acceptée
- [ ] F12 affiche la boîte de paiement dans Pesagem
- [ ] Admin : ouverture de Caixa ; Funcionário : vente de pesée payée par PIX ou espèces
- [ ] Coût d'achat estimé / prix de vente proposés configurables
- [ ] Rapport par produit et rapports jour/semaine/mois cohérents
- [ ] Lignes historiques sans coût marquées « incomplètes »
- [ ] Dépense opérationnelle créée, résultats recalculés
- [ ] Redémarrage : utilisateurs et opérations retrouvés, connexion à nouveau requise
- [ ] AppImage démarre sur une seconde machine Linux autorisée par sa propre licence
