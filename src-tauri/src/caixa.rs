use chrono::Utc;
use rusqlite::{params,Connection,OptionalExtension};
use serde_json::{json,Value};

fn er(e:impl std::fmt::Display)->String{e.to_string()}
fn now()->String{Utc::now().to_rfc3339()}
fn row_json(r:&rusqlite::Row<'_>)->rusqlite::Result<Value>{
 let id:i64=r.get(0)?;let status:String=r.get(1)?;let opened:String=r.get(2)?;let closed:Option<String>=r.get(3)?;let initial:i64=r.get(4)?;
 let deposits:i64=r.get(5)?;let withdrawals:i64=r.get(6)?;let total_sales:i64=r.get(7)?;let cash_net:i64=r.get(8)?;let purchases:i64=r.get(9)?;
 Ok(json!({"id":id,"status":status,"openedAt":opened,"closedAt":closed,"initialBalance":initial,"totalDeposits":deposits,"totalWithdrawals":withdrawals,"totalSales":total_sales,"cashNet":cash_net,"totalPurchases":purchases,"finalBalance":initial+deposits+cash_net-withdrawals,"grossProfit":null}))
}
const SUMMARY:&str="SELECT c.id,c.status,c.opened_at,c.closed_at,c.initial_balance_cents,
 COALESCE((SELECT SUM(m.amount_cents) FROM caixa_movements m WHERE m.caixa_id=c.id AND m.kind='deposit'),0),
 COALESCE((SELECT SUM(m.amount_cents) FROM caixa_movements m WHERE m.caixa_id=c.id AND m.kind='withdrawal'),0),
 COALESCE((SELECT SUM(s.total_cents) FROM sales s WHERE s.caixa_id=c.id AND s.kind='sale'),0),
 COALESCE((SELECT SUM(p.amount_cents) FROM payments p JOIN sales s ON s.id=p.sale_id WHERE p.caixa_id=c.id AND p.method='cash' AND s.kind='sale'),0)
 - COALESCE((SELECT SUM(p.amount_cents) FROM payments p JOIN sales s ON s.id=p.sale_id WHERE p.caixa_id=c.id AND p.method='cash' AND s.kind='purchase'),0),
 COALESCE((SELECT SUM(s.total_cents) FROM sales s WHERE s.caixa_id=c.id AND s.kind='purchase'),0)
 FROM caixa_sessions c";
fn get_summary(db:&Connection,id:i64)->Result<Value,String>{
 let mut val=db.query_row(&format!("{SUMMARY} WHERE c.id=?1"),[id],row_json).optional().map_err(er)?.ok_or("CAIXA_NOT_FOUND")?;
 let mut st=db.prepare("SELECT user, action, created_at FROM caixa_logs WHERE caixa_id=?1 ORDER BY id").map_err(er)?;
 let items=st.query_map([id],|r| Ok(json!({"user":r.get::<_,String>(0)?,"action":r.get::<_,String>(1)?,"timestamp":r.get::<_,String>(2)?}))).map_err(er)?;
 let mut logs=vec![];for item in items{logs.push(item.map_err(er)?)}val["historyLogs"]=json!(logs);
 // Persisted payment chronology for this cash drawer; pending UI entries are shown separately.
 let mut st=db.prepare("SELECT s.id,s.created_at,s.kind,s.invoice_number,s.total_cents,
 COALESCE((SELECT SUM(l.weight_grams) FROM sale_lines l WHERE l.sale_id=s.id),0),
 COALESCE((SELECT GROUP_CONCAT(a.designation || ' (' || printf('%.3f',l.weight_grams/1000.0) || ' kg)', ', ') FROM sale_lines l JOIN articles a ON a.id=l.article_id WHERE l.sale_id=s.id),''),
 COALESCE((SELECT GROUP_CONCAT(DISTINCT p.method) FROM payments p WHERE p.sale_id=s.id),''),
 COALESCE((SELECT SUM(p.amount_cents) FROM payments p WHERE p.sale_id=s.id),0)
 FROM sales s WHERE s.caixa_id=?1 ORDER BY s.created_at DESC,s.id DESC LIMIT 500").map_err(er)?;
 let entries=st.query_map([id],|row|{
  let total:i64=row.get(4)?;let paid:i64=row.get(8)?;
  Ok(json!({"id":row.get::<_,i64>(0)?,"when":row.get::<_,String>(1)?,"kind":row.get::<_,String>(2)?,
   "invoice":row.get::<_,Option<String>>(3)?,"amountCents":total,"weightGrams":row.get::<_,i64>(5)?,
   "title":row.get::<_,String>(6)?,"method":row.get::<_,String>(7)?,"status":if paid>=total {"paid"} else {"pending"}}))
 }).map_err(er)?;
 let mut timeline=vec![];for e in entries {timeline.push(e.map_err(er)?)}val["timeline"]=json!(timeline);
 Ok(val)
}
pub fn init(db:&Connection)->Result<(),String>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS caixa_sessions(id INTEGER PRIMARY KEY,status TEXT NOT NULL CHECK(status IN ('open','closed')),opened_at TEXT NOT NULL,closed_at TEXT,initial_balance_cents INTEGER NOT NULL CHECK(initial_balance_cents>=0));
 CREATE UNIQUE INDEX IF NOT EXISTS only_one_open_caixa ON caixa_sessions(status) WHERE status='open';
 CREATE TABLE IF NOT EXISTS caixa_movements(id INTEGER PRIMARY KEY,caixa_id INTEGER NOT NULL REFERENCES caixa_sessions(id),kind TEXT NOT NULL CHECK(kind IN ('deposit','withdrawal')),amount_cents INTEGER NOT NULL CHECK(amount_cents>0),created_at TEXT NOT NULL,user TEXT NOT NULL DEFAULT 'operador');
 CREATE TABLE IF NOT EXISTS caixa_logs(id INTEGER PRIMARY KEY,caixa_id INTEGER NOT NULL REFERENCES caixa_sessions(id),user TEXT NOT NULL,action TEXT NOT NULL,created_at TEXT NOT NULL);").map_err(er)?;
 for (t,c) in [("sales","caixa_id"),("payments","caixa_id")] {
  let mut stmt=db.prepare(&format!("PRAGMA table_info({t})")).map_err(er)?;
  let cols=stmt.query_map([],|r|r.get::<_,String>(1)).map_err(er)?;
  let mut found=false;for col in cols{if col.map_err(er)?==c {found=true}}
  if !found {db.execute_batch(&format!("ALTER TABLE {t} ADD COLUMN {c} INTEGER REFERENCES caixa_sessions(id)")).map_err(er)?;}
 }
 Ok(())
}
pub fn active_id(db:&Connection)->Result<i64,String>{db.query_row("SELECT id FROM caixa_sessions WHERE status='open' LIMIT 1",[],|r|r.get(0)).optional().map_err(er)?.ok_or_else(||"CAIXA_FECHADO: abra um caixa antes de registrar vendas".to_string())}
fn log(db:&Connection,id:i64,user:&str,action:&str)->Result<(),String>{db.execute("INSERT INTO caixa_logs(caixa_id,user,action,created_at) VALUES(?1,?2,?3,?4)",params![id,user,action,now()]).map_err(er)?;Ok(())}
pub fn dispatch(db:&mut Connection,channel:&str,args:&[Value],user:&str)->Result<Value,String>{
 let v=args.first().cloned().unwrap_or(Value::Null);
 match channel {
  "caixa:current"=>{
   match active_id(db){Ok(id)=>get_summary(db,id),Err(_)=>Ok(Value::Null)}
  },
  "caixa:open"=>{
   if active_id(db).is_ok(){return Err("CAIXA_ALREADY_OPEN".into())}
   let amount=v.get("amountCents").and_then(Value::as_i64).or_else(||v.as_i64()).ok_or("INVALID_AMOUNT")?;
   if !(0..=100_000_000_00).contains(&amount){return Err("INVALID_AMOUNT".into())}
   let number=v.get("number").and_then(Value::as_i64);
   if number.is_some_and(|id| id<=0||id>999_999_999){return Err("NUMERO_CAIXA_INVALIDO".into())}
   let tx=db.transaction().map_err(er)?;
   if let Some(id)=number {tx.execute("INSERT INTO caixa_sessions(id,status,opened_at,initial_balance_cents) VALUES(?1,'open',?2,?3)",params![id,now(),amount]).map_err(|e| format!("NUMERO_CAIXA_INDISPONIVEL: {e}"))?;}
   else {tx.execute("INSERT INTO caixa_sessions(status,opened_at,initial_balance_cents) VALUES('open',?1,?2)",params![now(),amount]).map_err(er)?;}
   let id=tx.last_insert_rowid();log(&tx,id,user,"Caixa aberto")?;tx.commit().map_err(er)?;get_summary(db,id)
  },
  "caixa:close"=>{
   let id=active_id(db)?;let tx=db.transaction().map_err(er)?;tx.execute("UPDATE caixa_sessions SET status='closed',closed_at=?1 WHERE id=?2",params![now(),id]).map_err(er)?;log(&tx,id,user,"Caixa fechado")?;tx.commit().map_err(er)?;get_summary(db,id)
  },
  "caixa:reopen"=>{
   if active_id(db).is_ok(){return Err("CAIXA_ALREADY_OPEN".into())}
   let id=v.as_i64().ok_or("CAIXA_NOT_FOUND")?;
   let tx=db.transaction().map_err(er)?;let changed=tx.execute("UPDATE caixa_sessions SET status='open',closed_at=NULL WHERE id=?1 AND status='closed'",[id]).map_err(er)?;
   if changed!=1{return Err("CAIXA_NOT_FOUND".into())}log(&tx,id,user,&format!("Usuário {} reabriu o caixa",user))?;tx.commit().map_err(er)?;get_summary(db,id)
  },
  "caixa:move"=>{
   let kind=v.get("kind").and_then(Value::as_str).unwrap_or("");
   let amount=v.get("amountCents").and_then(Value::as_i64).ok_or("INVALID_AMOUNT")?;
   if !["deposit","withdrawal"].contains(&kind)||amount<=0||amount>100_000_000_00{return Err("INVALID_MOVEMENT".into())}
   let id=active_id(db)?;
   let summary=get_summary(db,id)?;if kind=="withdrawal" && amount>summary["finalBalance"].as_i64().unwrap_or(0){return Err("SALDO_INSUFICIENTE".into())}
   let tx=db.transaction().map_err(er)?;tx.execute("INSERT INTO caixa_movements(caixa_id,kind,amount_cents,created_at,user) VALUES(?1,?2,?3,?4,?5)",params![id,kind,amount,now(),user]).map_err(er)?;
   log(&tx,id,user,if kind=="deposit"{"Dinheiro adicionado"}else{"Sangria realizada"})?;tx.commit().map_err(er)?;get_summary(db,id)
  },
  "caixa:list"=>{
   let from=v.get("from").and_then(Value::as_str).unwrap_or("0000-01-01");let to=v.get("to").and_then(Value::as_str).unwrap_or("9999-12-31");
   let query=v.get("search").and_then(Value::as_str).unwrap_or("").trim();
   let mut stmt=db.prepare(&format!("{SUMMARY} WHERE substr(c.opened_at,1,10) BETWEEN ?1 AND ?2 AND (CAST(c.id AS TEXT) LIKE ?3 OR c.status LIKE ?3) ORDER BY c.id DESC LIMIT 500")).map_err(er)?;
   let pattern=format!("%{}%",query);let result=stmt.query_map(params![from,to,pattern],row_json).map_err(er)?;
   let mut values=vec![];for r in result{let mut x=r.map_err(er)?;let details=get_summary(db,x["id"].as_i64().unwrap_or(0))?;x["historyLogs"]=details["historyLogs"].clone();x["timeline"]=details["timeline"].clone();values.push(x)}Ok(json!(values))
  },
  "caixa:period"=>{
   let from=v.get("from").and_then(Value::as_str).ok_or("INVALID_DATE")?;
   let to=v.get("to").and_then(Value::as_str).ok_or("INVALID_DATE")?;
   if from.len()!=10||to.len()!=10||from>to{return Err("INVALID_DATE".into())}
   let mut st=db.prepare(&format!("{SUMMARY} WHERE substr(c.opened_at,1,10) BETWEEN ?1 AND ?2")).map_err(er)?;
   let found=st.query_map(params![from,to],row_json).map_err(er)?;
   let mut count=0_i64;let mut sales=0_i64;let mut purchases=0_i64;
   let mut deposits=0_i64;let mut withdrawals=0_i64;
   let mut cash_net=0_i64;let mut initial_balances=0_i64;let mut final_balances=0_i64;
   for item in found {
    let x=item.map_err(er)?;
    count+=1;
    sales+=x["totalSales"].as_i64().unwrap_or(0);
    purchases+=x["totalPurchases"].as_i64().unwrap_or(0);
    deposits+=x["totalDeposits"].as_i64().unwrap_or(0);
    withdrawals+=x["totalWithdrawals"].as_i64().unwrap_or(0);
    cash_net+=x["cashNet"].as_i64().unwrap_or(0);
    initial_balances+=x["initialBalance"].as_i64().unwrap_or(0);
    final_balances+=x["finalBalance"].as_i64().unwrap_or(0);
   }
   Ok(json!({"count":count,"totalSales":sales,"totalPurchases":purchases,
    "totalDeposits":deposits,"totalWithdrawals":withdrawals,"cashNet":cash_net,
    "initialBalances":initial_balances,"finalBalances":final_balances,
    "from":from,"to":to}))
  },
  _=>Err("UNKNOWN_CAIXA_COMMAND".into())
 }
}
