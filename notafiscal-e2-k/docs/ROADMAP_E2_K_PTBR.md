# RECYCLING SYSTEM INDUSTRIAL — NOTAFISCAL RJ
## Plano técnico E2 a K e condições de aceitação

**Estado desta entrega:** laboratório criptográfico offline e automação de testes; **não é emissor NF-e autorizado**. Os fontes locais do usuário podem ter divergido das cópias de arquivo, portanto a instalação é **aditiva**, não substitui `src-tauri` nem `src/renderer`.

| Etapa | Funcionalidade de produto exigida | Entregue neste pacote | Pendência antes de concluir |
|---|---|---|---|
| E2 | A1 + XMLDSig infNFe | Assinatura RSA-SHA256 enveloped de laboratório; verificação, controle de alteração, PFX e senha interativa | Integrar ao Tauri, comprovar perfil NF-e atual, CNPJ, ICP-Brasil, revogação, validação independente |
| F | XML NF-e 55 | Validador offline por XSD fornecido pelo operador | Gerador fiscal completo, schema oficial versionado, tributos e CFOP aprovados pelo contador |
| G | Comunicação SEFAZ-RJ/SVRS | Bloqueio explícito; nenhum HTTP/SOAP real | Cliente mTLS SOAP, WSDL, homologação, estados, consulta e idempotência |
| H | DANFE e eventos | Manifesto de integridade SHA-256, sem validade fiscal | DANFE conforme MOC, autorização, cancelamento, inutilização, protocolos reais |
| I | Pesagem, Controle, Caixa | Sem alteração de dados; módulo independente | Integração transacional, tratamento de rejeição sem perda de movimento |
| J | Segurança e homologação | Testes unitários offline; checklist de evidências | Certificação contábil, credenciamento, testes oficiais e backup/restore |
| K | Instalação/release | Script aditivo e exemplo GitHub Actions | Build Linux e Windows, assinatura instaladores, migração/rollback testado |

### Características de segurança

- `sign-lab` exige `--ack-test-only`, não envia nada à rede, solicita senha PFX por `getpass` sem gravação.
- O `.pfx` permanece fora da árvore Git; não copie certificados nem chaves privadas para repositórios.
- `verify-local` verifica assinatura criptográfica, **não** revogação, cadeia ICP-Brasil, autorização SEFAZ ou validade fiscal.
- Não gerar uma regra tributária com base em suposição. Configuração de regime permanece pendente.
- `readiness` marca requisitos como pendentes por padrão e jamais modifica flags de emissão.
- O manifesto de XML produzido é marcado como **ARTEFATO_INTERNO_NAO_FISCAL**, não é DANFE.

### Uso offline após instalação

```bash
cd /media/user/skllnxs.cloud/Projets/LXO/V8-NotaFiscal/recycling-system-v8-notafiscal/desktop/notafiscal-e2-k
VENV=/media/user/skllnxs.cloud/Projets/LXO/cache-build/venv-notafiscal-e2-k
"$VENV/bin/python" -m tools.notafiscal.nfe_lab selftest
"$VENV/bin/python" -m tools.notafiscal.readiness
"$VENV/bin/python" -m tools.notafiscal.nfe_lab validate-xsd --xml documento.xml --xsd /caminho/schema-oficial.xsd
"$VENV/bin/python" -m tools.notafiscal.nfe_lab sign-lab --ack-test-only --xml nfe.xml --pfx /caminho/certificado.pfx --out assinado-teste.xml
"$VENV/bin/python" -m tools.notafiscal.nfe_lab verify-local --xml assinado-teste.xml
```

**Nota:** o `sign-lab` não torna o XML apto à SEFAZ. O algoritmo RSA-SHA256 usado na prova deve ser confrontado com o manual e schemas aplicáveis antes de integração. O XML de teste do `selftest` não é uma NF-e válida.

### Fontes oficiais para revisão em cada release
- https://www.nfe.fazenda.gov.br/portal/ — Portal NF-e
- https://dfe-portal.svrs.rs.gov.br/NFe/Documentos — MOC, NT, schemas, DANFE
- https://dfe-portal.svrs.rs.gov.br/NFe/Servicos — serviços web

A versão dos schemas e das NTs deve ser fixada por build/release, não importada automaticamente de origem desconhecida.
