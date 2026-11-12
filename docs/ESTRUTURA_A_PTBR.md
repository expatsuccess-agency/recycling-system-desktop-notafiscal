# Etapa A — Estrutura fiscal V8-NotaFiscal

- `src-tauri/src/fiscal.rs`: cadastro legal da empresa, migração aditiva, validação CNPJ e referência de arquivo A1.
- `src-tauri/src/store.rs`: colunas fiscais dos produtos e validação NCM/CEST/CST/CSOSN/CFOP no salvamento.
- `src-tauri/src/lib.rs`: registro e proteção das chamadas `fiscal:*` pela sessão Administrador.
- `src/renderer/index.tsx`: aba Dados Fiscais, CRUD de campos fiscais de produtos.
- `src/renderer/style.css`: estilo da nova aba.
- `build-nf-linux.sh`, `build-nf-windows.ps1`: builds direcionados.

## Limites

Não há emissão fiscal nem assinatura do XML nesta etapa. Nenhum segredo do certificado A1 deve ser gravado no banco. O arquivo de certificado fica fora da base; permissões e backup seguro devem ser configurados no sistema operacional. Confirmar todos os códigos tributários com a contabilidade.
