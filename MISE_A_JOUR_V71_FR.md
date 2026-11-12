# Recycling System Industrial — correctif V7.1.0

Base : V7.0.0. Cette archive contient les sources complètes (Tauri, Rust, React) et les tests.

## Modifications

- Pesagem : le champ de recherche n'affiche plus la liste entière au focus ; les suggestions s'ouvrent au début de la saisie ou avec Flèche bas.
- Pesagem : Peso et Preço unitário ont la même largeur et une hauteur de 62 px.
- Menu : « Vendas » est renommé « Controle » (F2) sans changer la nature du module de vente ; Configuração apparaît après Gestão Financeira.
- Configuração : icône avec fond blanc et sélecteur réduit.
- Configuração : onglets Produtos e Categorias, Usuários e Senhas, Controle, Manutenção do banco.
- Funcionário : accès Caixa (F3/F6/F7) via le menu et contrôles Rust, autres modules administratifs toujours bloqués.
- Controle / Vendas : F10 ouvre le paiement de la vente.
- Messages : traduction lisible en portugais des principales erreurs, y compris le code d'authentification.
- Maintenance : abandon des prompts natifs au profit d'une confirmation intégrée ; événements de progression en provenance de Rust, sauvegarde SQLite avant suppression, rapport des nombres d'entrées effacées et diagnostic en cas d'échec.

## Installation Parrot Linux

1. Quitter Recycling System.
2. Sauvegarder la base SQLite existante, située généralement dans `~/.local/share/br.com.lxo.desktop/lxo.sqlite`.
3. Décompresser dans `/media/user/skllnxs.cloud/Projets/LXO/V7.1/`.
4. Dans `recycling-system-v7/desktop`, synchroniser uniquement la clé publique existante `bash sync-license-key.sh`.
5. Exécuter `npm install`, `npm test`, `npm run tauri:dev`.

Ne pas régénérer les clés Ed25519 et ne pas copier la clé privée dans le client.

## Remise à zéro

Dans Configuração → Manutenção do banco, le profil Administrador doit cocher la confirmation et saisir exactement `ZERAR FINANCEIRO`. Une sauvegarde complète SQLite est produite avant la transaction. Les produits, catégories, utilisateurs, clients, balances, paramètres et licences ne sont pas ciblés par la suppression. Conserver le fichier de sauvegarde dans un emplacement sûr.

Attention : les messages de progression Rust ne constituent pas une garantie de réussite avant le message de confirmation final. En cas d'échec, consulter le journal visible dans l'application.

## Validation

`npm test` : 56 tests réussis. Analyse syntaxique du TSX : aucune erreur. La compilation complète Tauri/Rust et les tests de caisse sur machine réelle restent à valider.
