# Recycling System Industrial — V8-NotaFiscal — Étape A

**Livraison de développement / version technique 8.1.0** — Interface brésilienne.

## Périmètre exact

Cette livraison ne comporte **aucune émission, signature, transmission SEFAZ, XML fiscal, DANFE ou consultation de certificat**. Les fonctions Pesagem, Controle, Caixa, Gestão Financeira, licence Ed25519, authentification et maintenance restent sur le code de la V8. L'étape A prépare le cadastre fiscal et le catalogue.

- A1 : Table locale SQLite `fiscal_company` + journal `fiscal_audit` ; raison sociale, nom, CNPJ (numérique et alphanumérique), IE, CRT, UF, code IBGE, adresse, environnement. Chemin de référence du fichier A1 externe (`.pfx/.p12`), **pas le contenu**, et **aucun mot de passe stocké**. La sécurité du fichier A1 dépend des permissions et du chiffrement du poste ; stockage dans un coffre-fort système à réaliser avant une émission réelle.
- A2 : Champs supplémentaires des produits `ncm`, `cest`, `origem_icms`, `cst_icms`, `csosn`, `cfop_entrada`, `cfop_saida`. Un NCM à 8 chiffres devient obligatoire **lorsqu'un produit est créé ou modifié** dans cette édition. Les produits historiques sont conservés ; mettre à jour leur NCM au fur et à mesure. La simple longueur du NCM ne garantit pas que le code soit fiscalement correct.
- A3 : Nouveau sous-onglet **Configuração → Dados Fiscais · NF-e** réservé au profil Administrador ; sélecteur de chemin certificat A1 et formulaire de données légales.

## Sauvegarde préalable obligatoire

L'identifiant Tauri `br.com.lxo.desktop` est volontairement conservé afin de réutiliser la base et les licences V8 ; **cette version utilise donc le même répertoire de données que la V8**. Il est fortement recommandé de travailler sur une copie du poste et de sauvegarder le fichier `lxo.sqlite` avec ses fichiers `-wal` et `-shm` si l'application est en cours d'exécution (préférer arrêter l'application avant de copier). Ne jamais réinitialiser la base pour la migration fiscale.

La migration est appliquée lors du démarrage, en ajoutant les colonnes manquantes de façon idempotente. Elle ne supprime aucune table existante.

## Installation sur Linux

```bash
unzip RECYCLING_V8_NOTAFISCAL_ETAPA_A.zip
cd recycling-system-v8-notafiscal/desktop
# Copier depuis la V8 existante la clé publique Ed25519 exacte (PAS la clé privée).
# Exemple, après vérification des chemins :
# cp /chemin/V8/desktop/src-tauri/src/license_public_key.hex src-tauri/src/license_public_key.hex
npm install
npm run tauri:dev
```

**Bâtir** : `bash build-nf-linux.sh` (AppImage). Windows : sur un ordinateur Windows avec Rust/Tauri/NSIS, lancer `./build-nf-windows.ps1` en PowerShell. Le correctif du code machine Windows de la V8 est inclus, mais une licence liée à la machine Windows doit être créée par le gestionnaire existant ; les licences Linux sont inchangées.

## Recette étape A

1. Démarrer avec une sauvegarde de la base V8, se connecter `Administrador`. Vérifier les anciens produits et leur prix, les utilisateurs, la caisse et l'historique.
2. Dans **Configuração → Dados Fiscais**, saisir un **CNPJ valide** et une société fictive de test, l'IE, le CRT, l'UF, le code IBGE, le CEP et l'adresse. Choisir `Homologação`. Enregistrer. Redémarrer : les données doivent persister.
3. Dans **Produtos e Categorias**, éditer `FERRO`, saisir un NCM correct approuvé par le comptable, et si approprié CEST, CST ou CSOSN, CFOP entrée/sortie. Enregistrer, rouvrir et vérifier les valeurs.
4. Essayer d'enregistrer un produit sans NCM ou avec NCM non numérique : refus attendu, sans perte des anciens produits.
5. Sélectionner un fichier `.pfx` ou `.p12` d'essai via le bouton. Le système conserve seulement son chemin et vérifie son existence, **pas sa validité cryptographique**. Ne pas saisir de mot de passe dans les autres champs.
6. Vérifier qu'un compte `Funcionário` n'accède pas à `Dados Fiscais` et ne peut appeler les commandes `fiscal:*`.
7. Vérifier la non-régression Pesagem, Caixa, Controle, Finance, licence et changement de mot de passe.

**Attention aux données sensibles** : la table fiscale contient des identifiants légaux et l'adresse de l'entreprise. Sauvegardes SQLite à chiffrer et accès filesystem à restreindre. La sélection d'un environnement « Produção » dans le formulaire n'active pas d'émission.

## Évolution fiscale 2026 / points bloquants B/C

Les règles NF-e/NFC-e doivent être évaluées au regard de la réglementation SEFAZ de la **UF** de l'émetteur et de son régime CRT. Pour les matières recyclables, ne pas déduire NCM ou CFOP du seul nom du matériau ; validation comptable nécessaire. Les champs IBS/CBS et les NT 2025.002/2026.x imposent une veille et une adaptation technique supplémentaire. **NFC-e modèle 65 ne remplace pas la NF-e modèle 55 pour les flux B2B proposés.** L'émission de NF-e de entrada pour achats auprès de particuliers dépend de l'opération et des règles applicables ; rien n'est déclenché automatiquement en étape A.

Avant B : implémenter coffre-fort du secret A1, parsing PKCS#12, validation CNPJ/IE/certificat et signature XML ; moteur fiscal par UF, environnement et CFOP ; contrôle des NCM et des documents des partenaires. Les autorisations SEFAZ, annulations, inutilisations et contingences sont hors périmètre de cette étape.

## Sources documentaires

- Portal Nacional da NF-e — MOC 7.0 et notas técnicas: https://www.nfe.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=04BIflQt1aY=
- Receita Federal — CNPJ Alfanumérico: https://www.gov.br/receitafederal/pt-br/acesso-a-informacao/acoes-e-programas/programas-e-atividades/cnpj-alfanumerico

