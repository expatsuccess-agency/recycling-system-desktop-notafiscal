//! Etapa A — cadastro fiscal local (não emite nem assina documentos fiscais).
//! O certificado A1 continua no local escolhido pelo operador: nenhuma senha é gravada.
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::path::Path;

fn err(e:rusqlite::Error)->String {format!("ERRO_BANCO_FISCAL: {e}")}
fn val<'a>(v:&'a Value,k:&str)->&'a str {v.get(k).and_then(Value::as_str).unwrap_or("").trim()}
fn basic(v:&str,max:usize)->bool {v.chars().count()<=max && !v.chars().any(|c|c.is_control())}
fn code(v:&str,n:usize)->bool {v.len()==n && v.bytes().all(|c|c.is_ascii_digit())}
fn cnpj_valid(v:&str)->bool {
    // Receita Federal: CNPJ alfanumérico, 12 posições com A-Z/0-9 + 2 dígitos verificadores.
    if !v.is_ascii() || v.len()!=14 || !v[..12].bytes().all(|b|b.is_ascii_uppercase()||b.is_ascii_digit()) ||
       !v[12..].bytes().all(|b|b.is_ascii_digit()) ||
       v.bytes().all(|b|b==v.as_bytes()[0]) {return false}
    // Valor numérico de cada posição: código ASCII menos 48 (A=17, ..., Z=42).
    let d:Vec<u32>=v.bytes().map(|b|(b-48) as u32).collect();
    let weights1=[5,4,3,2,9,8,7,6,5,4,3,2];
    let weights2=[6,5,4,3,2,9,8,7,6,5,4,3,2];
    let digit=|digits:&[u32],weights:&[u32]| {let n:u32=digits.iter().zip(weights).map(|(a,b)|a*b).sum();let r=n%11;if r<2{0}else{11-r}};
    digit(&d[..12],&weights1)==d[12] && digit(&d[..13],&weights2)==d[13]
}

fn validate(v:&Value)->Result<(),String> {
    let cnpj=val(v,"cnpj");
    // Aceita CNPJ numérico e alfanumérico com os dígitos verificadores válidos.
    if !cnpj_valid(cnpj){return Err("CNPJ_INVÁLIDO: informe 14 caracteres válidos (CNPJ numérico ou alfanumérico), sem pontuação".into())}
    let crt=val(v,"crt");if !["1","2","3","4"].contains(&crt){return Err("REGIME_TRIBUTARIO_INVALIDO".into())}
    let uf=val(v,"uf");if uf.len()!=2||!uf.bytes().all(|x|x.is_ascii_uppercase()){return Err("UF_INVALIDA".into())}
    let ibge=val(v,"codigo_municipio");if !code(ibge,7){return Err("CODIGO_IBGE_INVALIDO".into())}
    let cep=val(v,"cep");if !code(cep,8){return Err("CEP_INVALIDO".into())}
    if !basic(val(v,"razao_social"),180)||val(v,"razao_social").is_empty()||!basic(val(v,"nome_fantasia"),180) || !basic(val(v,"inscricao_estadual"),24) || !basic(val(v,"logradouro"),180)||val(v,"logradouro").is_empty()||!basic(val(v,"numero"),16)||!basic(val(v,"bairro"),80)||!basic(val(v,"municipio"),80) || !basic(val(v,"complemento"),120) {return Err("DADOS_FISCAIS_INVALIDOS".into())}
    let environment=val(v,"ambiente");if !["homologacao","producao"].contains(&environment){return Err("AMBIENTE_FISCAL_INVALIDO".into())}
    Ok(())
}

pub fn initialize(db:&Connection)->Result<(),String>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS fiscal_company(
 id INTEGER PRIMARY KEY CHECK(id=1),
 razao_social TEXT NOT NULL DEFAULT '',nome_fantasia TEXT NOT NULL DEFAULT '',
 cnpj TEXT NOT NULL DEFAULT '',inscricao_estadual TEXT NOT NULL DEFAULT '',
 crt TEXT NOT NULL DEFAULT '1',uf TEXT NOT NULL DEFAULT '',
 codigo_municipio TEXT NOT NULL DEFAULT '',municipio TEXT NOT NULL DEFAULT '',
 cep TEXT NOT NULL DEFAULT '',logradouro TEXT NOT NULL DEFAULT '',numero TEXT NOT NULL DEFAULT '',
 bairro TEXT NOT NULL DEFAULT '',complemento TEXT NOT NULL DEFAULT '',
 ambiente TEXT NOT NULL DEFAULT 'homologacao',
 certificado_a1_caminho TEXT NOT NULL DEFAULT '',
 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
 INSERT OR IGNORE INTO fiscal_company(id) VALUES(1);
 CREATE TABLE IF NOT EXISTS fiscal_audit(id INTEGER PRIMARY KEY, at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, action TEXT NOT NULL);").map_err(err)?;
 let mut columns=std::collections::HashSet::new();
 let mut st=db.prepare("PRAGMA table_info(articles)").map_err(err)?;
 let rows=st.query_map([],|r|r.get::<_,String>(1)).map_err(err)?;
 for r in rows {columns.insert(r.map_err(err)?);}
 for (name,typ) in [
  ("ncm","TEXT NOT NULL DEFAULT ''"),("cest","TEXT NOT NULL DEFAULT ''"),
  ("origem_icms","TEXT NOT NULL DEFAULT '0'"),("cst_icms","TEXT NOT NULL DEFAULT ''"),
  ("csosn","TEXT NOT NULL DEFAULT ''"),("cfop_entrada","TEXT NOT NULL DEFAULT ''"),
  ("cfop_saida","TEXT NOT NULL DEFAULT ''")
 ] {if !columns.contains(name) {db.execute_batch(&format!("ALTER TABLE articles ADD COLUMN {name} {typ}")).map_err(err)?;}}
 Ok(())
}
pub fn dispatch(db:&Connection,channel:&str,v:&Value)->Result<Value,String>{
 match channel {
  "fiscal:company:get"=>{
   db.query_row("SELECT razao_social,nome_fantasia,cnpj,inscricao_estadual,crt,uf,codigo_municipio,municipio,cep,logradouro,numero,bairro,complemento,ambiente,certificado_a1_caminho FROM fiscal_company WHERE id=1",[],|r|{
    Ok(json!({"razao_social":r.get::<_,String>(0)?,"nome_fantasia":r.get::<_,String>(1)?,"cnpj":r.get::<_,String>(2)?,"inscricao_estadual":r.get::<_,String>(3)?,"crt":r.get::<_,String>(4)?,"uf":r.get::<_,String>(5)?,"codigo_municipio":r.get::<_,String>(6)?,"municipio":r.get::<_,String>(7)?,"cep":r.get::<_,String>(8)?,"logradouro":r.get::<_,String>(9)?,"numero":r.get::<_,String>(10)?,"bairro":r.get::<_,String>(11)?,"complemento":r.get::<_,String>(12)?,"ambiente":r.get::<_,String>(13)?,"certificado_a1_caminho":r.get::<_,String>(14)?}))
   }).map_err(err)
  },
  "fiscal:company:save"=>{
   validate(v)?;
   let cert=val(v,"certificado_a1_caminho");
   if !cert.is_empty(){
     let p=Path::new(cert);
     let ext=p.extension().and_then(|x|x.to_str()).unwrap_or("").to_ascii_lowercase();
     if !["p12","pfx"].contains(&ext.as_str())||!p.is_file() {return Err("CERTIFICADO_A1_ARQUIVO_INVALIDO".into())}
   }
   db.execute("UPDATE fiscal_company SET razao_social=?1,nome_fantasia=?2,cnpj=?3,inscricao_estadual=?4,crt=?5,uf=?6,codigo_municipio=?7,municipio=?8,cep=?9,logradouro=?10,numero=?11,bairro=?12,complemento=?13,ambiente=?14,certificado_a1_caminho=?15,updated_at=CURRENT_TIMESTAMP WHERE id=1",
    params![val(v,"razao_social"),val(v,"nome_fantasia"),val(v,"cnpj"),val(v,"inscricao_estadual"),val(v,"crt"),val(v,"uf"),val(v,"codigo_municipio"),val(v,"municipio"),val(v,"cep"),val(v,"logradouro"),val(v,"numero"),val(v,"bairro"),val(v,"complemento"),val(v,"ambiente"),cert]).map_err(err)?;
   db.execute("INSERT INTO fiscal_audit(action) VALUES ('CONFIGURACAO_FISCAL_ATUALIZADA')",[]).map_err(err)?;
   Ok(json!({"ok":true,"certificado_referenciado":!cert.is_empty(),"emissao_habilitada":false}))
  },
  _=>Err("CANAL_FISCAL_DESCONHECIDO".into())
 }
}
