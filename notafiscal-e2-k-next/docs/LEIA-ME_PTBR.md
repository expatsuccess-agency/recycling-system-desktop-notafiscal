# Recycling System Industrial — NF-e RJ — desenvolvimento E2 até K

## Estado verdadeiro da entrega

Este pacote é **aditivo**, projetado para a árvore desktop V8.2 que já existe no computador do cliente. Não altera o banco SQLite, não instala a chave privada Ed25519, não autoriza NF-e, não transmite SOAP nem acessa SEFAZ.

- **E2:** laboratório XMLDSig incluído para testes isolados; exige integração Rust/Tauri, perfil NF-e e verificação independente.
- **F:** gerador de **rascunho deliberadamente incompleto** `engine/nfe_draft.py`, com bloqueio de produção, regime não definido, tributos não aprovados, UF diferente de RJ e ambiente distinto de homologação. Não emite XML fiscal validado. Validação XSD exige informar um esquema oficial obtido do portal nacional.
- **G:** conectores SOAP e autenticação mTLS devem ser implementados e testados na SVRS. Não há conexão ativa neste pacote.
- **H:** DANFE oficial e eventos dependem dos documentos e respostas SEFAZ reais.
- **I:** transações Pesagem/Controle/Caixa precisam ser integradas no banco da aplicação com migrações e testes de consistência; nenhuma integração automática foi feita.
- **J:** checklist de evidências não se aprova sozinho. Necessita homologação, certificação e orientação contábil/fiscal.
- **K:** CI Linux/Windows, criação e inspeção de artefatos; **instalação e restauração em máquinas Linux/Windows ainda requerem teste real**, não apenas inspeção de arquivos.

O código de teste `tax_rules_approved=True` é **fixture fictícia**, nunca uma aprovação fiscal.

## Instalação

Extrair ZIP em pasta temporária e executar `bash scripts/instalar_auditar.sh`. Os scripts irão recusar substituir `notafiscal-e2-k-next` se ele já existir. Não usar `sudo` para compilar. Copiar manualmente `.github/workflows/notafiscal-build-validation.yml` e os scripts de inspeção para o repositório Git após revisão. O workflow depende de `package-lock.json` limpo e de chave pública de licença válida; se não existirem, deve falhar.

## Padrão de segurança

- Nunca registrar senha A1, chave privada, XML contendo dados pessoais ou chave JWT em Debug.
- Certificado de testes e de produção devem ser distintos. Só habilitar produção com aprovação expressa, validação dos esquemas e contabilista.
- Não interpretar status HTTP 200 como autorização NF-e; validar protocolo, identidade, assinatura e códigos cStat.
- Não repetir automaticamente transmissão incerta sem consulta prévia de status.
- Nunca alterar pagamentos/estoque automaticamente com base em erro de transporte.

## Testes realizados nesta entrega

`python3 -m unittest discover -s tests -v` — testes unitários do módulo offline. Não há teste real de Tauri/Windows/SEFAZ aqui.

## Fonte oficial

https://www.nfe.fazenda.gov.br/portal/listaConteudo.aspx?tipoConteudo=BMPFMBoln3w%3D (esquemas XML, conferir a revisão vigente antes da implementação).
