//! Planejamento/aprovação administrativa de lotes. NÃO transmite, não assina e não autoriza NF-e.
use rusqlite::{params,Connection,OptionalExtension};
use serde_json::{json,Value};
use chrono::Local;
fn err(e:impl std::fmt::Display)->String{e.to_string()}
fn date_ok(s:&str)->bool{chrono::NaiveDate::parse_from_str(s,"%Y-%m-%d").is_ok()}
fn time_ok(s:&str)->bool{chrono::NaiveTime::parse_from_str(s,"%H:%M").is_ok()}
pub fn init(db:&Connection)->Result<(),String>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS fiscal_batch_settings(
   id INTEGER PRIMARY KEY CHECK(id=1), enabled INTEGER NOT NULL DEFAULT 0,
   local_hour TEXT NOT NULL DEFAULT '18:00', updated_at TEXT DEFAULT CURRENT_TIMESTAMP);
 INSERT OR IGNORE INTO fiscal_batch_settings(id) VALUES(1);
 CREATE TABLE IF NOT EXISTS fiscal_batches (
   id INTEGER PRIMARY KEY, for_date TEXT NOT NULL, origin TEXT NOT NULL CHECK(origin IN ('manual','scheduled')),
   status TEXT NOT NULL CHECK(status IN ('review','approved')) DEFAULT 'review',
   created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP, approved_at TEXT,
   approved_by TEXT, note TEXT NOT NULL DEFAULT 'Sem integração SEFAZ');
 CREATE TABLE IF NOT EXISTS fiscal_batch_items (
   batch_id INTEGER NOT NULL REFERENCES fiscal_batches(id),
   document_id INTEGER NOT NULL REFERENCES fiscal_documents(id),
   PRIMARY KEY(batch_id,document_id));
 CREATE UNIQUE INDEX IF NOT EXISTS idx_fiscal_scheduled_once
   ON fiscal_batches(for_date) WHERE origin='scheduled';
 CREATE INDEX IF NOT EXISTS idx_fiscal_batch_items_doc ON fiscal_batch_items(document_id);
 ").map_err(err)
}
fn list(db:&Connection)->Result<Value,String>{
 let mut st=db.prepare("SELECT b.id,b.for_date,b.origin,b.status,b.created_at,b.approved_at,b.approved_by,
  (SELECT COUNT(*) FROM fiscal_batch_items bi WHERE bi.batch_id=b.id) AS item_count
  FROM fiscal_batches b ORDER BY b.id DESC LIMIT 100").map_err(err)?;
 let rows=st.query_map([],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"date":r.get::<_,String>(1)?,
 "origin":r.get::<_,String>(2)?,"status":r.get::<_,String>(3)?,"created_at":r.get::<_,String>(4)?,
 "approved_at":r.get::<_,Option<String>>(5)?,"approved_by":r.get::<_,Option<String>>(6)?,
 "count":r.get::<_,i64>(7)?,"transmitted":false}))).map_err(err)?;
 let mut result=Vec::new();for r in rows {result.push(r.map_err(err)?)}Ok(json!(result))
}
fn create(db:&Connection,date:&str,origin:&str)->Result<Value,String>{
 if !date_ok(date){return Err("DATA_LOTE_INVALIDA".into())}
 let docs={
  let mut st=db.prepare("SELECT d.id FROM fiscal_documents d
     WHERE substr(d.created_at,1,10)=?1 AND d.status NOT IN ('cancelado','autorizado')
     AND NOT EXISTS (SELECT 1 FROM fiscal_batch_items bi JOIN fiscal_batches b ON b.id=bi.batch_id
       WHERE bi.document_id=d.id AND b.status='approved') ORDER BY d.id LIMIT 50").map_err(err)?;
  let rows=st.query_map([date],|r|r.get::<_,i64>(0)).map_err(err)?;
  let mut ids=Vec::new();for row in rows{ids.push(row.map_err(err)?)}ids
 };
 if docs.is_empty(){return Err("SEM_RASCUNHOS_ELEGIVEIS_PARA_REVISAO".into())}
 // Os itens representam apenas uma lista de revisão; não há XML assinado nem lote SOAP.
 db.execute("INSERT INTO fiscal_batches(for_date,origin) VALUES (?1,?2)",params![date,origin]).map_err(err)?;
 let id=db.last_insert_rowid();
 for d in &docs {db.execute("INSERT INTO fiscal_batch_items(batch_id,document_id) VALUES (?1,?2)",params![id,d]).map_err(err)?;}
 Ok(json!({"id":id,"count":docs.len(),"status":"review","transmitted":false}))
}
pub fn dispatch(db:&Connection,channel:&str,v:&Value,user:&str)->Result<Value,String>{
 match channel {
  "batch:settings"=>{let (enabled,time):(i64,String)=db.query_row("SELECT enabled,local_hour FROM fiscal_batch_settings WHERE id=1",[],|r|Ok((r.get(0)?,r.get(1)?))).map_err(err)?;
   Ok(json!({"enabled":enabled!=0,"time":time,"automatic_transmission":false}))},
  "batch:settings:save"=>{let time=v.get("time").and_then(Value::as_str).ok_or("HORA_OBRIGATORIA")?;
   if !time_ok(time){return Err("HORA_INVALIDA".into())}
   let enabled=v.get("enabled").and_then(Value::as_bool).unwrap_or(false);
   db.execute("UPDATE fiscal_batch_settings SET enabled=?1,local_hour=?2,updated_at=CURRENT_TIMESTAMP WHERE id=1",params![enabled as i64,time]).map_err(err)?;
   Ok(json!({"ok":true,"enabled":enabled,"time":time,"transmitted":false}))},
  "batch:list"=>list(db),
  "batch:create"=>create(db,v.get("date").and_then(Value::as_str).ok_or("DATA_OBRIGATORIA")?,"manual"),
  "batch:tick"=>{
    let (enabled,hour):(i64,String)=db.query_row("SELECT enabled,local_hour FROM fiscal_batch_settings WHERE id=1",[],|r|Ok((r.get(0)?,r.get(1)?))).map_err(err)?;
    let now=Local::now();let day=now.format("%Y-%m-%d").to_string();let current=now.format("%H:%M").to_string();
    if enabled==0 || current<hour {return Ok(json!({"due":false,"transmitted":false}))}
    let exists:Option<i64>=db.query_row("SELECT id FROM fiscal_batches WHERE origin='scheduled' AND for_date=?1",[&day],|r|r.get(0)).optional().map_err(err)?;
    if exists.is_some(){return Ok(json!({"due":true,"created":false,"transmitted":false}))}
    match create(db,&day,"scheduled") {Ok(value)=>Ok(json!({"due":true,"created":true,"batch":value,"transmitted":false})),Err(e) if e=="SEM_RASCUNHOS_ELEGIVEIS_PARA_REVISAO"=>Ok(json!({"due":true,"created":false,"reason":e,"transmitted":false})),Err(e)=>Err(e)}
  },
  "batch:approve"=>{
   let id=v.get("id").and_then(Value::as_i64).ok_or("LOTE_INVALIDO")?;
   if v.get("confirmed").and_then(Value::as_bool)!=Some(true){return Err("APROVACAO_EXPLICITA_OBRIGATORIA".into())}
   let n=db.execute("UPDATE fiscal_batches SET status='approved',approved_at=CURRENT_TIMESTAMP,approved_by=?1 WHERE id=?2 AND status='review'",params![user,id]).map_err(err)?;
   if n==0{return Err("LOTE_NAO_ENCONTRADO_OU_JA_APROVADO".into())}
   Ok(json!({"id":id,"status":"approved","transmitted":false,"message":"Aprovado para planejamento; emissão e transmissão permanecem bloqueadas"}))
  },
  "batch:transmit"=>Err("ENVIO_LOTE_SEFAZ_NAO_IMPLEMENTADO_NAO_TRANSMITIDO".into()),
  _=>Err("CANAL_LOTE_DESCONHECIDO".into())
 }
}
