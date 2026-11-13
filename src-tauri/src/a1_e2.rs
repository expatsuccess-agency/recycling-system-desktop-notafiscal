//! E2: prova de posse da chave RSA no certificado A1, SEM assinar XML/NF-e.
//! Não transmite dados, não altera SQLite e não habilita emissão fiscal.
use openssl::{asn1::Asn1Time, hash::MessageDigest, nid::Nid, pkcs12::Pkcs12,
              pkey::Id, rand::rand_bytes, sign::{Signer,Verifier}};
use serde_json::{json, Value};
use std::{cmp::Ordering, fs, path::Path};

pub fn proof(input: &Value) -> Result<Value,String> {
    let path = input.get("path").and_then(Value::as_str).ok_or("E2_CAMINHO_OBRIGATORIO")?;
    let password = input.get("password").and_then(Value::as_str).ok_or("E2_SENHA_OBRIGATORIA")?;
    if password.len() > 1024 || password.contains('\n') || password.contains('\r') {return Err("E2_SENHA_INVALIDA".into());}
    let file = Path::new(path);
    let ext = file.extension().and_then(|e|e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !["pfx","p12"].contains(&ext.as_str()) || !file.is_file() { return Err("E2_ARQUIVO_INVALIDO".into()); }
    let metadata = fs::metadata(file).map_err(|_|"E2_ARQUIVO_INACESSIVEL")?;
    if metadata.len()==0 || metadata.len()>5_000_000 {return Err("E2_TAMANHO_INVALIDO".into());}
    let bytes = fs::read(file).map_err(|_|"E2_ARQUIVO_INACESSIVEL")?;
    let container = Pkcs12::from_der(&bytes).map_err(|_|"E2_PKCS12_INVALIDO")?;
    let parsed = container.parse2(password).map_err(|_|"E2_SENHA_OU_PKCS12_INVALIDO")?;
    let cert = parsed.cert.ok_or("E2_CERTIFICADO_AUSENTE")?;
    let key = parsed.pkey.ok_or("E2_CHAVE_PRIVADA_AUSENTE")?;
    if key.id() != Id::RSA {return Err("E2_CHAVE_NAO_RSA".into());}
    let public = cert.public_key().map_err(|_|"E2_CHAVE_PUBLICA_INVALIDA")?;
    if !key.public_eq(&public) {return Err("E2_CERTIFICADO_CHAVE_DIVERGENTE".into());}
    let now=Asn1Time::days_from_now(0).map_err(|_|"E2_RELOGIO_INVALIDO")?;
    if cert.not_before().compare(&now).map_err(|_|"E2_VALIDADE_INVALIDA")? == Ordering::Greater
       || cert.not_after().compare(&now).map_err(|_|"E2_VALIDADE_INVALIDA")? != Ordering::Greater {
        return Err("E2_CERTIFICADO_FORA_DA_VALIDADE".into());
    }
    // Desafio aleatório, não é o conteúdo de uma NF-e.
    let mut challenge=[0u8;32];
    rand_bytes(&mut challenge).map_err(|_|"E2_ALEATORIEDADE_INDISPONIVEL")?;
    let mut signer=Signer::new(MessageDigest::sha256(), &key).map_err(|_|"E2_ASSINADOR_INDISPONIVEL")?;
    signer.update(&challenge).map_err(|_|"E2_ASSINATURA_FALHOU")?;
    let signature=signer.sign_to_vec().map_err(|_|"E2_ASSINATURA_FALHOU")?;
    let mut verifier=Verifier::new(MessageDigest::sha256(), &public).map_err(|_|"E2_VERIFICADOR_INDISPONIVEL")?;
    verifier.update(&challenge).map_err(|_|"E2_VERIFICACAO_FALHOU")?;
    if !verifier.verify(&signature).map_err(|_|"E2_VERIFICACAO_FALHOU")? {
        return Err("E2_VERIFICACAO_FALHOU".into());
    }
    let fingerprint=cert.digest(MessageDigest::sha256()).map_err(|_|"E2_FINGERPRINT_FALHOU")?;
    let cn=cert.subject_name().entries_by_nid(Nid::COMMONNAME)
        .next().and_then(|v|v.data().as_utf8().ok()).map(|s|s.to_string()).unwrap_or_default();
    // Não infere CNPJ de CN. Não afirmar cadeia, revogação, nem XMLDSig.
    Ok(json!({"etapa":"E2","tipo":"PROVA_LOCAL_RSA_SHA256",
        "certificado_cn":cn,"fingerprint_sha256":hex::encode(fingerprint),
        "chave_privada_corresponde_ao_certificado":true,"assinatura_desafio_verificada":true,
        "certificado_dentro_da_validade":true,"cnpj_validado":false,
        "cadeia_icp_brasil_validada":false,"revogacao_validada":false,
        "xml_dsig_nfe_assinado":false,"transmitido_sefaz":false,"emissao_habilitada":false}))
}
