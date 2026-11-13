# Etapa E1 — Inspeção local do certificado digital A1

- **Configuração → Dados Fiscais · NF-e**: selecione o arquivo `.pfx` ou `.p12` e salve o cadastro; depois digite a senha no campo temporário e escolha **Verificar certificado A1**.
- O módulo usa **OpenSSL instalado no computador**. A senha é enviada pelo stdin ao processo, sem salvar no banco, logs, argumentos da linha de comando nem arquivo temporário. Não compartilhe o arquivo PFX ou a senha.
- Mostra sujeito, emissor, datas, impressão digital SHA-256 e resultado do teste de não expiração. A senha é apagada do campo após a operação.
- **Limitações**: ainda não verifica a cadeia de confiança ICP-Brasil, a revogação, a associação ao CNPJ (otherName ICP-Brasil), a posse da chave privada, nem gera XML assinado. A operação pode falhar em instalações OpenSSL incompatíveis com o PKCS#12 utilizado. Para Windows, OpenSSL deve estar instalado e acessível via PATH.
- **Sempre bloqueado**: transmissão e autorização NF-e, cancelamento e emissão em produção. Nenhuma regra tributária é presumida.
- Não confunda esta inspeção com homologação fiscal ou autorização SEFAZ.

## Teste local (sem enviar notas)

```bash
openssl version
npm test
npm run tauri:dev
```

Na interface, verifique que certificados inválidos retornam erro sem expor a senha, que os campos de validade aparecem para arquivo correto, e que nenhuma NF-e pode ser transmitida.

## Etapas seguintes

- E2: validar cadeia ICP-Brasil, revogação, CNPJ e posse da chave privada; implementar assinatura XMLDSig no padrão NF-e, com vetores oficiais.
- F: geração XML 4.00 e validação XSD com notas técnicas aplicáveis, incluindo IBS/CBS.
- G: comunicação oficial de homologação NF-e RJ/SVRS, com certificação mútua e logs expurgados.
- H–J: protocolo, DANFE, eventos fiscais, integração de negócio, auditoria e preparação de produção.
