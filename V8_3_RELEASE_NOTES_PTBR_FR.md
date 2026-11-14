# Recycling System Industrial V8.3 NotaFiscal — Patch de fontes

## Português brasileiro

- **Atalhos Fn**: F1 Pesagem; F3 Controle; F5 Caixa Atual; F7 Gestão Financeira; F9 Configuração; F11 Caixas Anteriores; F12 validar a operação.
- **Lotes NF-e**: preparação manual ou preparação programada durante a execução do aplicativo, com revisão e aprovação administrativa explícita. A programação não continua com o aplicativo fechado.
- **Importante**: lote "approved" significa **somente aprovação interna**. Não há envio SOAP, assinatura XMLDSig NF-e ou autorização SEFAZ. A API `batch:transmit` sempre retorna erro.
- **Relatórios PDF**: Caixa, Compras, Vendas, controle fiscal e consolidado por intervalo; PDF consolidado automático após fechar o caixa. Os arquivos são armazenados em Documentos/Recycling System/Relatorios.
- **Git**: instalador exige repositório limpo, cria branch específica e guarda backups. Não faz push; --commit disponível.
- **DADOS**: migração local aditiva `fiscal_batch_settings`, `fiscal_batches`, `fiscal_batch_items`. Não apaga operações fiscais ou comerciais.
- **CONFORMIDADE**: relatórios não são DANFE ou XML oficial. Emissão e consulta SEFAZ exigem implementação/homologação.

## Français

- Raccourcis Fn corrigés et affichés dans les menus.
- Lots NF-e créés manuellement ou préparés à heure fixe **uniquement lorsque l'application fonctionne**; validation administrative explicite, jamais de transmission SEFAZ réelle.
- Rapports PDF sur demande, rapport consolidé après clôture de caisse, PDF paginés.
- Installation sans écrasement des modifications Git non validées, sauvegarde préalable, branche dédiée, commit facultatif.
- Aucune promesse d'émission fiscale homologuée ; les statuts administratifs n'équivalent pas aux autorisations SEFAZ.

## Limites / étapes restantes

- Ce correctif doit encore être compilé et testé dans les environnements Linux/Windows de destination (Cargo non disponible dans l'environnement d'édition).
- Le planificateur de préparation des lots n'est pas un daemon : il se déclenche à l'ouverture du sous-onglet fiscal puis toutes les 60 s tant qu'il reste monté.
- Le rapport PDF privilégie une présentation textuelle structurée, et non un modèle de DANFE.
- Une préparation de lot est limitée à 50 brouillons par lot ; ceci ne constitue pas une validation des limites de transmission SEFAZ.
