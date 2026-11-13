//! Etapas A-D: preparação e rastreamento LOCAL, sem assinatura/transmissão SEFAZ.
//! O certificado A1 continua no local escolhido pelo operador: nenhuma senha é gravada.
use rusqlite::{Connection, params};
use serde_json::{Value, json};
use std::path::Path;
use sha2::{Sha256,Digest};

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
    // Rascunho fiscal: permite dados pendentes sem inventar o regime tributário.
    let cnpj=val(v,"cnpj");
    if !cnpj.is_empty() && !cnpj_valid(cnpj) {return Err("CNPJ_INVALIDO".into())}
    let crt=val(v,"crt");
    if !["", "1", "2", "3", "4"].contains(&crt) {return Err("REGIME_TRIBUTARIO_INVALIDO".into())}
    let uf=val(v,"uf");
    if !uf.is_empty() && uf!="RJ" {return Err("UF_NAO_HABILITADA_NESTA_ETAPA".into())}
    let ibge=val(v,"codigo_municipio");
    if !ibge.is_empty() && !code(ibge,7) {return Err("CODIGO_IBGE_INVALIDO".into())}
    let cep=val(v,"cep");
    if !cep.is_empty() && !code(cep,8) {return Err("CEP_INVALIDO".into())}
    for k in ["razao_social","nome_fantasia","logradouro","municipio","complemento"] {
        if !basic(val(v,k),180) {return Err("DADOS_FISCAIS_INVALIDOS".into())}
    }
    for k in ["inscricao_estadual", "numero", "bairro"] {
        if !basic(val(v,k),80) {return Err("DADOS_FISCAIS_INVALIDOS".into())}
    }
    let environment=val(v,"ambiente");
    if environment!="homologacao" {return Err("AMBIENTE_DE_PRODUCAO_INDISPONIVEL".into())}
    Ok(())
}

fn pendencias(db:&Connection)->Result<Value,String>{
    let c: (String,String,String,String,String,String,String,String,String,String)=db.query_row(
      "SELECT razao_social,cnpj,inscricao_estadual,crt,uf,codigo_municipio,municipio,cep,logradouro,certificado_a1_caminho FROM fiscal_company WHERE id=1", [], |r| Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,r.get(6)?,r.get(7)?,r.get(8)?,r.get(9)?))
    ).map_err(err)?;
    let mut items:Vec<String>=Vec::new();
    for (v,txt) in [(&c.0,"Razão social não informada"),(&c.1,"CNPJ não informado"),(&c.2,"Inscrição Estadual não informada ou pendente de análise"),(&c.3,"Regime tributário não definido: valide com a contabilidade"),(&c.5,"Código IBGE do município não informado"),(&c.6,"Município não informado"),(&c.7,"CEP não informado"),(&c.8,"Logradouro não informado"),(&c.9,"Certificado A1 não selecionado ou não validado")]{
       if v.trim().is_empty(){items.push(txt.into());}
    }
    if c.4!="RJ" {items.push("A UF deve ser RJ nesta etapa".into());}
    let (missing,invalid):(i64,i64)=db.query_row("SELECT COALESCE(SUM(CASE WHEN ncm='' THEN 1 ELSE 0 END),0),COALESCE(SUM(CASE WHEN ncm<>'' AND (length(ncm)<>8 OR ncm GLOB '*[^0-9]*') THEN 1 ELSE 0 END),0) FROM articles WHERE active=1", [], |r|Ok((r.get(0)?,r.get(1)?))).map_err(err)?;
    if missing>0 {items.push(format!("{} produto(s) ativo(s) sem NCM",missing));}
    if invalid>0 {items.push(format!("{} produto(s) com NCM inválido",invalid));}
    items.push("Regra de tributação, CFOP e CST/CSOSN devem ser aprovados pelo contador antes de emitir".into());
    items.push("Credenciamento e certificado A1 não foram verificados com a SEFAZ-RJ".into());
    Ok(json!({"uf":"RJ","ambiente":"homologacao","emissao_habilitada":false,"pendencias":items,"produtos_sem_ncm":missing,"produtos_ncm_invalido":invalid}))
}

pub fn initialize(db:&Connection)->Result<(),String>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS fiscal_company(
 id INTEGER PRIMARY KEY CHECK(id=1),
 razao_social TEXT NOT NULL DEFAULT '',nome_fantasia TEXT NOT NULL DEFAULT '',
 cnpj TEXT NOT NULL DEFAULT '',inscricao_estadual TEXT NOT NULL DEFAULT '',
 crt TEXT NOT NULL DEFAULT '',uf TEXT NOT NULL DEFAULT 'RJ',
 codigo_municipio TEXT NOT NULL DEFAULT '',municipio TEXT NOT NULL DEFAULT '',
 cep TEXT NOT NULL DEFAULT '',logradouro TEXT NOT NULL DEFAULT '',numero TEXT NOT NULL DEFAULT '',
 bairro TEXT NOT NULL DEFAULT '',complemento TEXT NOT NULL DEFAULT '',
 ambiente TEXT NOT NULL DEFAULT 'homologacao',
 certificado_a1_caminho TEXT NOT NULL DEFAULT '',
 updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
 INSERT OR IGNORE INTO fiscal_company(id) VALUES(1);
 UPDATE fiscal_company SET crt='', uf='RJ' WHERE crt='1' AND razao_social='' AND cnpj='' AND inscricao_estadual='';
 CREATE TABLE IF NOT EXISTS fiscal_audit(id INTEGER PRIMARY KEY, at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, action TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS fiscal_partners (
  id INTEGER PRIMARY KEY, kind TEXT NOT NULL CHECK(kind IN ('fornecedor','cliente')),
  name TEXT NOT NULL, document TEXT NOT NULL DEFAULT '', ie TEXT NOT NULL DEFAULT '',
  uf TEXT NOT NULL DEFAULT 'RJ', address TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
 CREATE UNIQUE INDEX IF NOT EXISTS idx_fiscal_partner_doc ON fiscal_partners(kind,document) WHERE document<>'';
 CREATE TABLE IF NOT EXISTS fiscal_documents (
  id INTEGER PRIMARY KEY, direction TEXT NOT NULL CHECK(direction IN ('entrada','saida')),
  partner_id INTEGER REFERENCES fiscal_partners(id),
  sale_id INTEGER REFERENCES sales(id),
  model TEXT NOT NULL DEFAULT '55', series TEXT NOT NULL DEFAULT '',
  number TEXT NOT NULL DEFAULT '',status TEXT NOT NULL DEFAULT 'rascunho'
    CHECK(status IN ('rascunho','pendente_configuracao','pendente_revisao','xml_importado','rejeitado','autorizado','cancelado')),
  operation_description TEXT NOT NULL DEFAULT '',
  cfop TEXT NOT NULL DEFAULT '',
  xml_source_path TEXT NOT NULL DEFAULT '',xml_sha256 TEXT NOT NULL DEFAULT '',
  sefaz_protocol TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP);
 CREATE TABLE IF NOT EXISTS fiscal_document_events (
  id INTEGER PRIMARY KEY, document_id INTEGER NOT NULL REFERENCES fiscal_documents(id),
  at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, event TEXT NOT NULL, note TEXT NOT NULL DEFAULT '');
 CREATE TABLE IF NOT EXISTS debug_transport_log (
  id INTEGER PRIMARY KEY,at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  category TEXT NOT NULL,method TEXT NOT NULL DEFAULT '',
  endpoint TEXT NOT NULL DEFAULT '',status TEXT NOT NULL DEFAULT '',
  duration_ms INTEGER NOT NULL DEFAULT 0, detail TEXT NOT NULL DEFAULT '');
 CREATE INDEX IF NOT EXISTS idx_debug_at ON debug_transport_log(id DESC);
 CREATE INDEX IF NOT EXISTS idx_fiscal_docs_status ON fiscal_documents(status);").map_err(err)?;
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
fn cpf_valid(v:&str)->bool {
 if !code(v,11) || v.bytes().all(|b|b==v.as_bytes()[0]) {return false;}
 let d: Vec<u32>=v.bytes().map(|b|u32::from(b-b'0')).collect();
 let check=|n:usize| {let sum:u32=(0..n).map(|i| d[i]*(n as u32+1-i as u32)).sum();let r=(sum*10)%11;if r==10 {0}else{r}};
 check(9)==d[9] && check(10)==d[10]
}
fn partner_valid(v:&Value)->bool {
 let name=val(v,"name");let doc=val(v,"document");
 !name.is_empty() && basic(name,180) && (doc.is_empty() || cpf_valid(doc) || cnpj_valid(doc)) &&
 ["RJ","SP","MG","ES","AC","AL","AP","AM","BA","CE","DF","GO","MA","MT","MS","PA","PB","PR","PE","PI","RN","RS","RO","RR","SC","SE","TO"].contains(&val(v,"uf")) && basic(val(v,"address"),350)
}
fn list_query(db:&Connection,sql:&str)->Result<Value,String>{
 let mut st=db.prepare(sql).map_err(err)?;
 let names:Vec<String>=st.column_names().iter().map(|x|x.to_string()).collect();
 let data=st.query_map([],|r|{
  let mut o=serde_json::Map::new();
  for (i,k) in names.iter().enumerate(){
   let x:rusqlite::types::Value=r.get(i)?;
   let j=match x {rusqlite::types::Value::Integer(n)=>json!(n),rusqlite::types::Value::Real(n)=>json!(n),rusqlite::types::Value::Text(s)=>json!(s),_=>Value::Null};
   o.insert(k.clone(),j);
  }
  Ok(Value::Object(o))
 }).map_err(err)?;
 let mut out=Vec::new();for x in data {out.push(x.map_err(err)?)}Ok(json!(out))
}
fn import_xml_reference(db:&Connection,v:&Value)->Result<Value,String>{
 let id=v.get("id").and_then(Value::as_i64).ok_or("DOCUMENTO_INVALIDO")?;
 let path=val(v,"path");
 let p=Path::new(path);
 if !p.is_file() || p.extension().and_then(|s|s.to_str()).unwrap_or("").to_lowercase()!="xml" {return Err("XML_INVALIDO".into())}
 let bytes=std::fs::read(p).map_err(|_|"XML_NAO_ACESSIVEL")?;
 if bytes.len()>10_000_000 || bytes.len()<20 || !bytes.starts_with(b"<?xml") && !bytes.starts_with(b"<") {return Err("XML_INVALIDO".into())}
 // Reference only: not an XML parser, signature verifier or fiscal authorization check.
 let hash=hex::encode(Sha256::digest(&bytes));
 let n=db.execute("UPDATE fiscal_documents SET xml_source_path=?1,xml_sha256=?2,status='xml_importado',updated_at=CURRENT_TIMESTAMP WHERE id=?3 AND status IN ('rascunho','pendente_configuracao','pendente_revisao','xml_importado')",params![path,hash,id]).map_err(err)?;
 if n!=1 {return Err("DOCUMENTO_NAO_EDITAVEL".into())}
 db.execute("INSERT INTO fiscal_document_events(document_id,event,note) VALUES (?1,'XML_REFERENCIADO','XML recebido; assinatura, estrutura e validade fiscal NÃO verificadas')",params![id]).map_err(err)?;
 Ok(json!({"ok":true,"sha256":hash,"validade_fiscal_confirmada":false}))
}
pub fn dispatch(db:&Connection,channel:&str,v:&Value)->Result<Value,String>{
 match channel {
  "fiscal:a1:inspect"=>{
   let path:String=db.query_row("SELECT certificado_a1_caminho FROM fiscal_company WHERE id=1",[],|r|r.get(0)).map_err(err)?;
   if path.trim().is_empty(){return Err("CERTIFICADO_A1_NAO_SELECIONADO".into());}
   let password=v.get("senha").and_then(Value::as_str).ok_or("CERTIFICADO_A1_SENHA_OBRIGATORIA")?;
   let result=crate::a1::inspect(&path,password)?;
   db.execute("INSERT INTO fiscal_audit(action) VALUES ('CERTIFICADO_A1_INSPECIONADO_LOCALMENTE')",[]).map_err(err)?;
   Ok(result)
  },
  "fiscal:partners:list"=>list_query(db,"SELECT id,kind,name,document,ie,uf,address,created_at FROM fiscal_partners ORDER BY id DESC LIMIT 300"),
  "fiscal:partner:save"=>{
   if !partner_valid(v) || !["fornecedor","cliente"].contains(&val(v,"kind")) {return Err("PARCEIRO_FISCAL_INVALIDO".into())}
   let id=v.get("id").and_then(Value::as_i64);
   if let Some(id)=id {
     let changed=db.execute("UPDATE fiscal_partners SET kind=?1,name=?2,document=?3,ie=?4,uf=?5,address=?6 WHERE id=?7",params![val(v,"kind"),val(v,"name"),val(v,"document"),val(v,"ie"),val(v,"uf"),val(v,"address"),id]).map_err(err)?;
     if changed!=1{return Err("PARCEIRO_NAO_ENCONTRADO".into())}
     Ok(json!({"id":id,"ok":true}))
   }else{
     db.execute("INSERT INTO fiscal_partners(kind,name,document,ie,uf,address) VALUES (?1,?2,?3,?4,?5,?6)",params![val(v,"kind"),val(v,"name"),val(v,"document"),val(v,"ie"),val(v,"uf"),val(v,"address")]).map_err(err)?;
     Ok(json!({"id":db.last_insert_rowid(),"ok":true}))
   }
  },
  "fiscal:documents:list"=>list_query(db,"SELECT d.id,d.direction,d.partner_id,d.sale_id,d.model,d.series,d.number,d.status,d.operation_description,d.cfop,d.xml_sha256,d.sefaz_protocol,d.created_at,p.name AS partner_name FROM fiscal_documents d LEFT JOIN fiscal_partners p ON p.id=d.partner_id ORDER BY d.id DESC LIMIT 300"),
  "fiscal:document:events"=>{
   let id=v.get("id").and_then(Value::as_i64).ok_or("DOCUMENTO_INVALIDO")?;
   let mut st=db.prepare("SELECT id,at,event,note FROM fiscal_document_events WHERE document_id=?1 ORDER BY id").map_err(err)?;
   let rows=st.query_map(params![id],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"at":r.get::<_,String>(1)?,"event":r.get::<_,String>(2)?,"note":r.get::<_,String>(3)?}))).map_err(err)?;
   let mut out=Vec::new();for r in rows{out.push(r.map_err(err)?)}Ok(json!(out))
  },
  "fiscal:document:create"=>{
   let direction=val(v,"direction");
   if !["entrada","saida"].contains(&direction) || !basic(val(v,"operation_description"),350) || !basic(val(v,"cfop"),4) {return Err("DOCUMENTO_INVALIDO".into())}
   let cfop=val(v,"cfop");if !cfop.is_empty() && !code(cfop,4) {return Err("CFOP_INVALIDO".into())}
   let partner=v.get("partner_id").and_then(Value::as_i64).ok_or("PARCEIRO_OBRIGATORIO")?;
   let exists:i64=db.query_row("SELECT COUNT(*) FROM fiscal_partners WHERE id=?1 AND kind=?2",params![partner,if direction=="entrada"{"fornecedor"}else{"cliente"}],|r|r.get(0)).map_err(err)?;
   if exists==0 {return Err("PARCEIRO_INCOMPATIVEL".into())}
   let sale=v.get("sale_id").and_then(Value::as_i64);
   if let Some(sale)=sale {let count:i64=db.query_row("SELECT COUNT(*) FROM sales WHERE id=?1 AND kind=?2",params![sale,if direction=="entrada"{"purchase"}else{"sale"}],|r|r.get(0)).map_err(err)?;if count==0{return Err("OPERACAO_NAO_ENCONTRADA".into())}}
   db.execute("INSERT INTO fiscal_documents(direction,partner_id,sale_id,operation_description,cfop,status) VALUES (?1,?2,?3,?4,?5,'pendente_configuracao')",params![direction,partner,sale,val(v,"operation_description"),cfop]).map_err(err)?;
   let id=db.last_insert_rowid();
   db.execute("INSERT INTO fiscal_document_events(document_id,event,note) VALUES (?1,'RASCUNHO_CRIADO','Emissão bloqueada; dados fiscais dependem de revisão')",params![id]).map_err(err)?;
   Ok(json!({"id":id,"status":"pendente_configuracao","emissao_habilitada":false}))
  },
  "fiscal:document:import_xml"=>import_xml_reference(db,v),
  "fiscal:document:request_authorization"=>Err("EMISSAO_SEFAZ_NAO_IMPLEMENTADA_NAO_TRANSMITIDO".into()),
  "fiscal:document:cancel"=>Err("CANCELAMENTO_SEFAZ_NAO_IMPLEMENTADO".into()),
  "fiscal:document:consult"=>Err("CONSULTA_SEFAZ_NAO_IMPLEMENTADA".into()),
  "debug:events"=>list_query(db,"SELECT id,at,category,method,endpoint,status,duration_ms,detail FROM debug_transport_log ORDER BY id DESC LIMIT 500"),
  "debug:clear"=>{db.execute("DELETE FROM debug_transport_log",[]).map_err(err)?;Ok(json!({"ok":true}))},
  "debug:record"=>{
   let cat=val(v,"category");let endpoint=val(v,"endpoint");let status=val(v,"status");
   if !["IPC","REST","SOAP","WEBHOOK","WEB"].contains(&cat) || !basic(endpoint,120) || !basic(status,24) || !basic(val(v,"method"),16) || !basic(val(v,"detail"),120) {return Err("DEBUG_EVENTO_INVALIDO".into())}
   let ms=v.get("duration_ms").and_then(Value::as_i64).unwrap_or(0).clamp(0,120000);
   db.execute("INSERT INTO debug_transport_log(category,method,endpoint,status,duration_ms,detail) VALUES (?1,?2,?3,?4,?5,?6)",params![cat,val(v,"method"),endpoint,status,ms,val(v,"detail")]).map_err(err)?;
   db.execute("DELETE FROM debug_transport_log WHERE id NOT IN (SELECT id FROM debug_transport_log ORDER BY id DESC LIMIT 1500)",[]).map_err(err)?;
   Ok(json!({"ok":true}))
  },
  "fiscal:company:get"=>{
   db.query_row("SELECT razao_social,nome_fantasia,cnpj,inscricao_estadual,crt,uf,codigo_municipio,municipio,cep,logradouro,numero,bairro,complemento,ambiente,certificado_a1_caminho FROM fiscal_company WHERE id=1",[],|r|{
    Ok(json!({"razao_social":r.get::<_,String>(0)?,"nome_fantasia":r.get::<_,String>(1)?,"cnpj":r.get::<_,String>(2)?,"inscricao_estadual":r.get::<_,String>(3)?,"crt":r.get::<_,String>(4)?,"uf":r.get::<_,String>(5)?,"codigo_municipio":r.get::<_,String>(6)?,"municipio":r.get::<_,String>(7)?,"cep":r.get::<_,String>(8)?,"logradouro":r.get::<_,String>(9)?,"numero":r.get::<_,String>(10)?,"bairro":r.get::<_,String>(11)?,"complemento":r.get::<_,String>(12)?,"ambiente":r.get::<_,String>(13)?,"certificado_a1_caminho":r.get::<_,String>(14)?}))
   }).map_err(err)
  },
  "fiscal:pendencias"=>pendencias(db),
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
   Ok(json!({"ok":true,"certificado_referenciado":!cert.is_empty(),"emissao_habilitada":false,"configuracao":"rascunho"}))
  },
  _=>Err("CANAL_FISCAL_DESCONHECIDO".into())
 }
}
