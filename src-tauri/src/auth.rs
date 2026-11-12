use argon2::{Argon2,PasswordHash,PasswordHasher,PasswordVerifier};
use argon2::password_hash::SaltString;
use rand_core::OsRng;
use rusqlite::{params,Connection,OptionalExtension};
use serde_json::{json,Value};
use std::{collections::HashMap,sync::Mutex};
use uuid::Uuid;

#[derive(Clone)] pub struct Identity {pub id:i64,pub username:String,pub role:String,pub must_change:bool}
pub struct Sessions(pub Mutex<HashMap<String,Identity>>);
fn error(e:impl std::fmt::Display)->String{e.to_string()}
fn hash(p:&str)->Result<String,String>{let salt=SaltString::generate(&mut OsRng);Ok(Argon2::default().hash_password(p.as_bytes(),&salt).map_err(error)?.to_string())}
fn valid_pin(pin:&str)->bool{pin.len()==8 && pin.bytes().all(|b|b.is_ascii_digit())}
fn verify(p:&str,h:&str)->bool{PasswordHash::new(h).ok().map(|x|Argon2::default().verify_password(p.as_bytes(),&x).is_ok()).unwrap_or(false)}
pub fn initialize(db:&Connection)->Result<(),String>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS app_users(id INTEGER PRIMARY KEY,username TEXT NOT NULL UNIQUE,password_hash TEXT NOT NULL,role TEXT NOT NULL CHECK(role IN ('admin','employee')),must_change INTEGER NOT NULL DEFAULT 0,active INTEGER NOT NULL DEFAULT 1,created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);").map_err(error)?;
 let n:i64=db.query_row("SELECT count(*) FROM app_users",[],|r|r.get(0)).map_err(error)?;
 if n==0 {db.execute("INSERT INTO app_users(username,password_hash,role,must_change) VALUES('admin',?1,'admin',1)",params![hash("admin")?]).map_err(error)?;}
 Ok(())
}
pub fn current(token:Option<&str>, sessions:&Sessions)->Result<Identity,String>{let token=token.ok_or("SESSÃO_NECESSÁRIA")?;sessions.0.lock().map_err(error)?.get(token).cloned().ok_or_else(||"SESSÃO_INVÁLIDA".into())}
pub fn dispatch(db:&Connection,channel:&str,body:&Value,sessions:&Sessions,token:Option<&str>)->Result<Value,String>{
 match channel {
  "auth:login"=>{
    let username=body["username"].as_str().unwrap_or("").trim();let password=body["password"].as_str().unwrap_or("");
    let result=db.query_row("SELECT id,password_hash,role,must_change FROM app_users WHERE username=?1 AND active=1",[username],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,i64>(3)?))).optional().map_err(error)?;
    let (id,h,role,change)=result.ok_or("CREDENCIAIS_INVÁLIDAS")?;
    if !verify(password,&h){return Err("CREDENCIAIS_INVÁLIDAS".into())}
    let identity=Identity{id,username:username.into(),role,must_change:change!=0};let key=Uuid::new_v4().to_string();
    sessions.0.lock().map_err(error)?.insert(key.clone(),identity.clone());
    Ok(json!({"session":key,"username":identity.username,"role":identity.role,"mustChange":identity.must_change}))
  },
  "auth:change-password"=>{
    let me=current(token,sessions)?;let old=body["oldPassword"].as_str().unwrap_or("");let new=body["newPassword"].as_str().unwrap_or("");
    if !valid_pin(new) {return Err("SENHA_DEVE_TER_8_DÍGITOS".into())}
    let old_hash:String=db.query_row("SELECT password_hash FROM app_users WHERE id=?1",[me.id],|r|r.get(0)).map_err(error)?;
    if !verify(old,&old_hash){return Err("SENHA_ATUAL_INVÁLIDA".into())}
    db.execute("UPDATE app_users SET password_hash=?1,must_change=0 WHERE id=?2",params![hash(new)?,me.id]).map_err(error)?;
    for s in sessions.0.lock().map_err(error)?.values_mut(){if s.id==me.id{s.must_change=false}}
    Ok(json!(true))
  },
  "auth:users"=>{let me=current(token,sessions)?;if me.role!="admin"||me.must_change{return Err("ACESSO_NEGADO".into())}
    let mut stmt=db.prepare("SELECT id,username,role,active,must_change FROM app_users ORDER BY username").map_err(error)?;
    let rows=stmt.query_map([],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"username":r.get::<_,String>(1)?,"role":r.get::<_,String>(2)?,"active":r.get::<_,i64>(3)?}))).map_err(error)?;
    let mut v=vec![];for row in rows{v.push(row.map_err(error)?)}Ok(json!(v))
  },
  "auth:create-user"=>{let me=current(token,sessions)?;if me.role!="admin"||me.must_change{return Err("ACESSO_NEGADO".into())}
    let name=body["username"].as_str().unwrap_or("").trim();let pwd=body["password"].as_str().unwrap_or("");let role=body["role"].as_str().unwrap_or("");
    if name.len()<3||name.len()>40||!name.chars().all(|x|x.is_ascii_alphanumeric()||x=='_'||x=='-')||!valid_pin(pwd)||!["admin","employee"].contains(&role){return Err("DADOS_DE_USUÁRIO_INVÁLIDOS".into())}
    db.execute("INSERT INTO app_users(username,password_hash,role,must_change) VALUES(?1,?2,?3,1)",params![name,hash(pwd)?,role]).map_err(error)?;Ok(json!(true))
  },
  "auth:reset-password"=>{let me=current(token,sessions)?;if me.role!="admin"||me.must_change{return Err("ACESSO_NEGADO".into())}
    let id=body["userId"].as_i64().ok_or("USUÁRIO_INVÁLIDO")?;let new=body["newPassword"].as_str().unwrap_or("");
    if id==me.id{return Err("USE_ALTERAR_MINHA_SENHA".into())}
    if !valid_pin(new){return Err("SENHA_DEVE_TER_8_DÍGITOS".into())}
    let updated=db.execute("UPDATE app_users SET password_hash=?1,must_change=1 WHERE id=?2",params![hash(new)?,id]).map_err(error)?;
    if updated==0{return Err("USUÁRIO_NÃO_ENCONTRADO".into())}
    sessions.0.lock().map_err(error)?.retain(|_,s|s.id!=id);
    Ok(json!(true))
  },
  "auth:logout"=>{if let Some(t)=token {sessions.0.lock().map_err(error)?.remove(t);}Ok(json!(true))},
  _=>Err("COMANDO_DESCONHECIDO".into())
 }
}
