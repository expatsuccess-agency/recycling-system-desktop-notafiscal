//! Signed offline licenses for Recycling System Industrial V4.0.
//! Never embed an Ed25519 private key in this client.
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize,Serialize};
use sha2::{Digest,Sha256};
use std::{fs,path::PathBuf,time::{SystemTime,UNIX_EPOCH}};

const PUBLIC_KEY_HEX:&str=include_str!("license_public_key.hex");
const PRODUCT:&str="recycling-system-industrial";

#[derive(Debug,Deserialize)]
struct Envelope {format:String,payload:String,signature:String}
#[derive(Debug,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
struct LicensePayload {
    schema:u32,license_id:String,product:String,customer:String,machine_id:String,
    edition:String,issued_at:String,expires_at:Option<String>,features:Vec<String>
}
#[derive(Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status {pub valid:bool,pub code:String,pub machine_id:String,pub customer:Option<String>,pub expires_at:Option<String>,pub license_location:String}

fn unix_now()->Result<i64,String> { Ok(SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_|"RELÓGIO_INVÁLIDO")?.as_secs() as i64) }
fn parse_time(s:&str)->Result<i64,String> { chrono::DateTime::parse_from_rfc3339(s).map(|d|d.timestamp()).map_err(|_|"DATA_INVÁLIDA".into()) }

#[cfg(target_os="windows")]
fn windows_machine_guid()->Result<String,String> {
    // Identifier stable de l'installation Windows, lu sans elevation.
    // Ne jamais utiliser un identifiant de demonstration ou une valeur aleatoire.
    let output = std::process::Command::new("reg")
        .args(["query", r"HKLM\SOFTWARE\Microsoft\Cryptography", "/v", "MachineGuid"])
        .output().map_err(|_|"IDENTIFICADOR_INDISPONÍVEL")?;
    if !output.status.success() { return Err("IDENTIFICADOR_INDISPONÍVEL".into()); }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let guid = stdout.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let label = parts.next()?;
        let ty = parts.next()?;
        let val = parts.next()?;
        if label.eq_ignore_ascii_case("MachineGuid") && ty.starts_with("REG_") {
            Some(val.to_ascii_lowercase())
        } else { None }
    }).ok_or("IDENTIFICADOR_INDISPONÍVEL")?;
    let valid = guid.len() >= 32 && guid.len() <= 64 &&
        guid.chars().all(|c| c.is_ascii_hexdigit() || c == '-');
    if !valid { return Err("IDENTIFICADOR_INVÁLIDO".into()); }
    Ok(guid)
}

pub fn machine_id()->Result<String,String> {
    #[cfg(target_os="linux")]
    {
        // Intact pour la compatibilite des licences Linux existantes.
        let value=fs::read_to_string("/etc/machine-id")
            .map_err(|_|"IDENTIFICADOR_INDISPONÍVEL")?;
        let v=value.trim().to_lowercase();
        if v.len()!=32 || !v.chars().all(|c|c.is_ascii_hexdigit()) {
            return Err("IDENTIFICADOR_INVÁLIDO".into());
        }
        let mut hash=Sha256::new();
        hash.update(b"recycling-system-linux-v1\0");
        hash.update(v.as_bytes());
        return Ok(format!("rs4-{}",hex::encode(hash.finalize())));
    }
    #[cfg(target_os="windows")]
    {
        let v=windows_machine_guid()?;
        let mut hash=Sha256::new();
        hash.update(b"recycling-system-windows-v1\0");
        hash.update(v.as_bytes());
        return Ok(format!("rs4-{}",hex::encode(hash.finalize())));
    }
    #[cfg(not(any(target_os="linux",target_os="windows")))]
    {
        Err("SISTEMA_NÃO_SUPORTADO".into())
    }
}

fn internal_license_path(app:&tauri::AppHandle)->Result<PathBuf,String> {
    use tauri::Manager;
    let root=app.path().app_data_dir().map_err(|e|e.to_string())?;
    Ok(root.join("license").join("recycling.lic"))
}
fn pointer_path(app:&tauri::AppHandle)->Result<PathBuf,String> {
    use tauri::Manager;
    Ok(app.path().app_data_dir().map_err(|e|e.to_string())?.join("license-location.json"))
}
fn license_path(app:&tauri::AppHandle)->Result<PathBuf,String> {
    let pointer=pointer_path(app)?;
    if !pointer.exists() { return internal_license_path(app); }
    let text=fs::read_to_string(pointer).map_err(|_|"LOCAL_DA_LICENÇA_INVÁLIDO")?;
    let path:PathBuf=serde_json::from_str(&text).map_err(|_|"LOCAL_DA_LICENÇA_INVÁLIDO")?;
    if !path.is_absolute() {return Err("CAMINHO_DA_LICENÇA_INVÁLIDO".into())}
    Ok(path)
}
fn checked_file(path:&std::path::Path)->Result<String,String> {
    let metadata=fs::metadata(path).map_err(|_|"ARQUIVO_DE_LICENÇA_NÃO_ENCONTRADO")?;
    if !metadata.is_file() || metadata.len()>16384 {return Err("ARQUIVO_DE_LICENÇA_INVÁLIDO".into())}
    fs::read_to_string(path).map_err(|_|"ARQUIVO_DE_LICENÇA_INACESSÍVEL".into())
}
fn check_content(contents:&str)->Result<LicensePayload,String> {
    let envelope:Envelope=serde_json::from_str(contents).map_err(|_|"ARQUIVO_INVÁLIDO")?;
    if envelope.format!="RS-LIC-1" {return Err("FORMATO_INVÁLIDO".into())}
    let pubbytes=hex::decode(PUBLIC_KEY_HEX.trim()).map_err(|_|"CHAVE_PÚBLICA_NÃO_CONFIGURADA")?;
    let bytes:[u8;32]=pubbytes.try_into().map_err(|_|"CHAVE_PÚBLICA_INVÁLIDA")?;
    let key=VerifyingKey::from_bytes(&bytes).map_err(|_|"CHAVE_PÚBLICA_INVÁLIDA")?;
    let payload_bytes=B64.decode(envelope.payload).map_err(|_|"ARQUIVO_INVÁLIDO")?;
    let sig_bytes=B64.decode(envelope.signature).map_err(|_|"ASSINATURA_INVÁLIDA")?;
    let signature=Signature::from_slice(&sig_bytes).map_err(|_|"ASSINATURA_INVÁLIDA")?;
    key.verify(&payload_bytes,&signature).map_err(|_|"ASSINATURA_INVÁLIDA")?;
    let p:LicensePayload=serde_json::from_slice(&payload_bytes).map_err(|_|"DADOS_INVÁLIDOS")?;
    if p.schema!=1 || p.product!=PRODUCT {return Err("PRODUTO_INVÁLIDO".into())}
    if p.machine_id!=machine_id()? {return Err("LICENÇA_DE_OUTRO_COMPUTADOR".into())}
    let now=unix_now()?;
    if parse_time(&p.issued_at)?>now+300 {return Err("RELÓGIO_DO_SISTEMA_INVÁLIDO".into())}
    if let Some(ref expiry)=p.expires_at {if now>parse_time(expiry)? {return Err("LICENÇA_EXPIRADA".into())}}
    Ok(p)
}
fn read_checked(app:&tauri::AppHandle)->Result<LicensePayload,String> {
    let s=checked_file(&license_path(app)?)?;
    check_content(&s)
}
pub fn assert_valid(app:&tauri::AppHandle)->Result<(),String> {read_checked(app).map(|_|())}
pub fn status(app:&tauri::AppHandle)->Status {
    let location=license_path(app).map(|p|p.to_string_lossy().to_string()).unwrap_or_default();
    let machine=machine_id().unwrap_or_else(|_|"indisponível".into());
    match read_checked(app){
        Ok(p)=>Status{valid:true,code:"ATIVA".into(),machine_id:machine,customer:Some(p.customer),expires_at:p.expires_at,license_location:location},
        Err(code)=>Status{valid:false,code,machine_id:machine,customer:None,expires_at:None,license_location:location}
    }
}
pub fn install(app:&tauri::AppHandle,content:&str)->Result<Status,String> {
    if content.len()>16384 {return Err("ARQUIVO_MUITO_GRANDE".into())}
    check_content(content)?;
    let path=internal_license_path(app)?;
    let parent=path.parent().ok_or("CAMINHO_INVÁLIDO")?;
    fs::create_dir_all(parent).map_err(|e|e.to_string())?;
    let temp=parent.join("recycling.lic.tmp");
    fs::write(&temp,content).map_err(|e|e.to_string())?;
    fs::rename(&temp,&path).map_err(|e|e.to_string())?;
    // Importação: manter uma cópia privada, mesmo após remoção do USB.
    let pointer=pointer_path(app)?;
    if pointer.exists() {fs::remove_file(pointer).map_err(|e|e.to_string())?;}
    Ok(status(app))
}

/// Vincula um arquivo .lic já existente sem copiá-lo. Ele deve permanecer acessível.
pub fn use_external(app:&tauri::AppHandle,path_str:&str)->Result<Status,String> {
    let candidate=std::path::Path::new(path_str);
    if candidate.extension().and_then(|x|x.to_str()).map(|x|x.eq_ignore_ascii_case("lic"))!=Some(true) {
        return Err("SELECIONE_UM_ARQUIVO_LIC".into());
    }
    let canonical=fs::canonicalize(candidate).map_err(|_|"ARQUIVO_DE_LICENÇA_NÃO_ENCONTRADO")?;
    let content=checked_file(&canonical)?;
    check_content(&content)?;
    let pointer=pointer_path(app)?;
    if let Some(dir)=pointer.parent() {fs::create_dir_all(dir).map_err(|e|e.to_string())?;}
    let tmp=pointer.with_extension("tmp");
    fs::write(&tmp,serde_json::to_vec(&canonical).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
    fs::rename(tmp,pointer).map_err(|e|e.to_string())?;
    Ok(status(app))
}
pub fn use_internal(app:&tauri::AppHandle)->Result<Status,String> {
    let path=internal_license_path(app)?;
    let content=checked_file(&path)?;
    check_content(&content)?;
    let pointer=pointer_path(app)?;
    if pointer.exists() { fs::remove_file(pointer).map_err(|e|e.to_string())?; }
    Ok(status(app))
}
