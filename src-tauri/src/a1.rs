//! Etapa E1: inspeção local do certificado A1, sem assinatura de NF-e nem transmissão.
//! A senha entra somente por stdin de um subprocesso e não é persistida ou registrada.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{io::Write, path::Path, process::{Command,Stdio}};

fn openssl(arguments:&[&str], input:&[u8])->Result<std::process::Output,String>{
    let mut child=Command::new("openssl").args(arguments)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
        .spawn().map_err(|_|"OPENSSL_NAO_INSTALADO")?;
    if let Some(mut stdin)=child.stdin.take() {stdin.write_all(input).map_err(|_|"CERTIFICADO_A1_LEITURA_FALHOU")?;}
    child.wait_with_output().map_err(|_|"CERTIFICADO_A1_LEITURA_FALHOU".to_string())
}

pub fn inspect(path:&str,password:&str)->Result<Value,String>{
    let file=Path::new(path);
    let ext=file.extension().and_then(|s|s.to_str()).unwrap_or("").to_ascii_lowercase();
    if !["pfx","p12"].contains(&ext.as_str()) || !file.is_file() {
        return Err("CERTIFICADO_A1_ARQUIVO_INVALIDO".into());
    }
    if std::fs::metadata(file).map_err(|_|"CERTIFICADO_A1_LEITURA_FALHOU")?.len()>5_000_000 {
        return Err("CERTIFICADO_A1_ARQUIVO_GRANDE".into());
    }
    if password.contains('\n') || password.contains('\r') || password.len()>1024 {
        return Err("CERTIFICADO_A1_SENHA_INVALIDA".into());
    }
    // OpenSSL lê a senha de stdin, nunca como argumento/variável de ambiente.
    let mut secret=password.as_bytes().to_vec(); secret.push(b'\n');
    let result=openssl(&["pkcs12","-in",path,"-clcerts","-nokeys","-passin","stdin"],&secret);
    secret.fill(0);
    let pem=result?;
    if !pem.status.success(){return Err("CERTIFICADO_A1_SENHA_OU_ARQUIVO_INVALIDO".into());}
    // Somente certificado público PEM, jamais chave privada.
    let text=String::from_utf8_lossy(&pem.stdout);
    let start=text.find("-----BEGIN CERTIFICATE-----").ok_or("CERTIFICADO_A1_SEM_CERTIFICADO")?;
    let tail=&text[start..];
    let end=tail.find("-----END CERTIFICATE-----").ok_or("CERTIFICADO_A1_SEM_CERTIFICADO")?+"-----END CERTIFICATE-----".len();
    let cert=tail[..end].as_bytes();
    let metadata=openssl(&["x509","-noout","-subject","-issuer","-dates","-fingerprint","-sha256"],cert)?;
    if !metadata.status.success(){return Err("CERTIFICADO_A1_X509_INVALIDO".into());}
    let validity=openssl(&["x509","-noout","-checkend","0"],cert)?;
    let details=String::from_utf8_lossy(&metadata.stdout);
    let get=|prefix:&str|details.lines().find_map(|l|l.strip_prefix(prefix).map(|s|s.trim().to_string())).unwrap_or_default();
    let fp=get("sha256 Fingerprint=");
    // CNPJ pode constar em otherName/SAN ICP-Brasil. Não inferir titularidade do CN.
    Ok(json!({"tipo":"A1","formato":"PKCS#12","arquivo_sha256":hex::encode(Sha256::digest(std::fs::read(file).map_err(|_|"CERTIFICADO_A1_LEITURA_FALHOU")?)),
        "sujeito":get("subject="),"emissor":get("issuer="),"inicio":get("notBefore="),"validade":get("notAfter="),
        "fingerprint_sha256":fp,"nao_expirado":validity.status.success(),"cnpj_verificado":false,
        "cadeia_icp_brasil_verificada":false,"revogacao_verificada":false,"chave_privada_testada":false,
        "assinatura_xml_implementada":false,"emissao_habilitada":false}))
}
