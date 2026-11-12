use tauri::Emitter;
use crate::caixa;
use rusqlite::{params, Connection, OptionalExtension, types::ValueRef};
use serde_json::{json, Value};
use chrono::Utc;

fn err(e: impl std::fmt::Display) -> String { e.to_string() }
fn a(args: &[Value], i: usize) -> Value { args.get(i).cloned().unwrap_or(Value::Null) }
fn s(v: &Value) -> String { v.as_str().unwrap_or("").trim().to_string() }
fn i(v: &Value) -> i64 { v.as_i64().unwrap_or(0) }
fn field<'a>(v: &'a Value, k: &str) -> &'a Value { v.get(k).unwrap_or(&Value::Null) }
fn fs(v: &Value, k: &str) -> String { s(field(v,k)) }
fn fi(v: &Value, k: &str) -> i64 { i(field(v,k)) }
fn stamp() -> String { Utc::now().to_rfc3339() }

fn rows(db: &Connection, sql: &str, params: &[&dyn rusqlite::ToSql]) -> Result<Value,String> {
 let mut stmt = db.prepare(sql).map_err(err)?;
 let names: Vec<String> = stmt.column_names().iter().map(|x|x.to_string()).collect();
 let rs = stmt.query_map(params, |row| {
   let mut out = serde_json::Map::new();
   for (idx, name) in names.iter().enumerate() {
     let value = match row.get_ref(idx)? {
       ValueRef::Null=>Value::Null, ValueRef::Integer(v)=>json!(v),
       ValueRef::Real(v)=>json!(v), ValueRef::Text(v)=>json!(String::from_utf8_lossy(v).to_string()),
       ValueRef::Blob(_)=>Value::Null,
     };
     out.insert(name.clone(),value);
   }
   Ok(Value::Object(out))
 }).map_err(err)?;
 let mut result=Vec::new();
 for row in rs { result.push(row.map_err(err)?); }
 Ok(Value::Array(result))
}
fn one(db:&Connection, sql:&str, params:&[&dyn rusqlite::ToSql]) -> Result<Value,String> {
 let array=rows(db,sql,params)?;
 Ok(array.as_array().and_then(|x|x.first()).cloned().unwrap_or(Value::Null))
}
fn exec(db:&Connection, sql:&str, params:&[&dyn rusqlite::ToSql]) -> Result<usize,String> {
 db.execute(sql, params).map_err(err)
}
fn has_col(db:&Connection,table:&str,col:&str)->Result<bool,String>{
 let mut st=db.prepare(&format!("PRAGMA table_info({})",table)).map_err(err)?;
 let cols=st.query_map([], |r|r.get::<_,String>(1)).map_err(err)?;
 for name in cols {if name.map_err(err)?==col {return Ok(true)}}Ok(false)
}
fn add_col(db:&Connection,t:&str,c:&str,decl:&str)->Result<(),String>{
 if !has_col(db,t,c)? {db.execute_batch(&format!("ALTER TABLE {} ADD COLUMN {} {}",t,c,decl)).map_err(err)?;}Ok(())
}
pub fn initialize(db:&Connection)->Result<(),String>{
 db.pragma_update(None,"journal_mode","WAL").map_err(err)?;
 db.pragma_update(None,"foreign_keys","ON").map_err(err)?;
 db.execute_batch("CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY,value TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS articles (id INTEGER PRIMARY KEY,code_article TEXT UNIQUE NOT NULL,designation TEXT NOT NULL,price_per_kg INTEGER NOT NULL,unit TEXT NOT NULL DEFAULT 'kg',vat_rate REAL NOT NULL DEFAULT 0,active INTEGER NOT NULL DEFAULT 1);
 CREATE TABLE IF NOT EXISTS sales (id INTEGER PRIMARY KEY,created_at TEXT NOT NULL,kind TEXT NOT NULL DEFAULT 'sale',total_cents INTEGER NOT NULL,status TEXT NOT NULL DEFAULT 'validated',invoice_number TEXT UNIQUE);
 CREATE TABLE IF NOT EXISTS sale_lines (id INTEGER PRIMARY KEY,sale_id INTEGER NOT NULL REFERENCES sales(id),article_id INTEGER NOT NULL REFERENCES articles(id),weight_grams INTEGER NOT NULL,price_per_kg INTEGER NOT NULL,total_cents INTEGER NOT NULL);
 CREATE TABLE IF NOT EXISTS payments (id INTEGER PRIMARY KEY,sale_id INTEGER NOT NULL REFERENCES sales(id),created_at TEXT NOT NULL,amount_cents INTEGER NOT NULL,method TEXT NOT NULL,receipt_number TEXT UNIQUE NOT NULL);
 CREATE TABLE IF NOT EXISTS scales (id INTEGER PRIMARY KEY,name TEXT NOT NULL,host TEXT NOT NULL,port INTEGER NOT NULL,is_default INTEGER NOT NULL DEFAULT 0);
 CREATE TABLE IF NOT EXISTS clients (id INTEGER PRIMARY KEY, gender TEXT NOT NULL DEFAULT '', first_name TEXT NOT NULL,last_name TEXT NOT NULL,photo TEXT NOT NULL DEFAULT '',document TEXT NOT NULL UNIQUE,created_at TEXT NOT NULL);
 CREATE TABLE IF NOT EXISTS categories (id INTEGER PRIMARY KEY,name TEXT UNIQUE NOT NULL);
 CREATE TABLE IF NOT EXISTS audit (id INTEGER PRIMARY KEY,created_at TEXT NOT NULL,action TEXT NOT NULL,resource TEXT,resource_id INTEGER);").map_err(err)?;
 for (t,c,d) in [
 ("sales","client_id","INTEGER REFERENCES clients(id)"),
 ("sales","buyer_company","TEXT"),
 ("articles","category_id","INTEGER REFERENCES categories(id)"),
 ("articles","icon","TEXT NOT NULL DEFAULT 'package'"),
 ("articles","price_editable","INTEGER NOT NULL DEFAULT 0"),
 ("articles","cost_per_kg","INTEGER"),
 ("articles","sale_price_per_kg","INTEGER"),
 ("sale_lines","cost_per_kg","INTEGER"),
 ("scales","brand","TEXT NOT NULL DEFAULT ''"),
 ("scales","model","TEXT NOT NULL DEFAULT ''"),
 ("scales","transport","TEXT NOT NULL DEFAULT 'tcp'"),
 ("scales","serial_path","TEXT NOT NULL DEFAULT ''"),
 ("scales","baud_rate","INTEGER NOT NULL DEFAULT 9600"),
 ("scales","photo","TEXT NOT NULL DEFAULT ''") ] { add_col(db,t,c,d)?; }
 caixa::init(db)?;
 exec(db,"INSERT OR IGNORE INTO categories(name) VALUES ('Geral')",&[])?;
 let count:i64=db.query_row("SELECT COUNT(*) FROM articles",[],|r|r.get(0)).map_err(err)?;
 if count==0 {for (code,name,price) in [("FERRO","Ferro",800),("ALUM","Alumínio",1250),("PAPEL","Papelão",150)] {
 exec(db,"INSERT INTO articles(code_article,designation,price_per_kg,vat_rate) VALUES (?1,?2,?3,0)",&[&code,&name,&price])?;
 }}
 Ok(())
}
pub fn scale_by_id(db:&Connection,id:i64)->Result<Value,String>{
 let r=one(db,"SELECT * FROM scales WHERE id=?1",&[&id])?;
 if r.is_null(){Err("NO_SCALE_SELECTED".into())}else{Ok(r)}
}
pub fn pdf_data(db:&Connection,kind:&str,id:i64)->Result<Value,String>{
 match kind {
 "client" => {let r=one(db,"SELECT * FROM clients WHERE id=?1",&[&id])?;if r.is_null(){Err("NOT_FOUND".into())}else{Ok(r)}},
 "sale" => sale_detail(db,id),
 _=>Err("INVALID_PDF".into())
 }
}
fn sale_detail(db:&Connection,id:i64)->Result<Value,String>{
 let mut sale=one(db,"SELECT s.*, c.first_name||' '||c.last_name AS client_name,c.document AS client_document FROM sales s LEFT JOIN clients c ON c.id=s.client_id WHERE s.id=?1",&[&id])?;
 if sale.is_null(){return Err("NOT_FOUND".into())}
 let lines=rows(db,"SELECT l.*,a.designation,c.name AS category FROM sale_lines l JOIN articles a ON a.id=l.article_id LEFT JOIN categories c ON c.id=a.category_id WHERE l.sale_id=?1",&[&id])?;
 let payments=rows(db,"SELECT * FROM payments WHERE sale_id=?1",&[&id])?;
 sale["lines"]=lines; sale["payments"]=payments;Ok(sale)
}
fn saved_id(db:&Connection)->Value{ json!({"id":db.last_insert_rowid()}) }
fn delete(db:&Connection,table:&str,id:i64)->Result<Value,String>{
 let q=format!("DELETE FROM {} WHERE id=?1",table);
 Ok(json!(exec(db,&q,&[&id])?))
}
pub fn dispatch(db:&mut Connection,channel:&str,args:&[Value],user:&str,app:&tauri::AppHandle)->Result<Value,String>{
 let v=a(args,0);
 match channel {
 "settings:get" => {let key=s(&v);if key=="admin_password_hash" {return Ok(json!(false))}
 if !["language","license_url","license_port"].contains(&key.as_str()){return Ok(Value::Null)}
 one(db,"SELECT value FROM settings WHERE key=?1",&[&key]).map(|x|if x.is_null(){Value::Null}else{x["value"].clone()})},
 "settings:set" => {let key=s(&v);let val=s(&a(args,1));if !["language","license_url","license_port"].contains(&key.as_str()) || val.len()>500{return Err("INVALID_SETTING".into())}
 if key=="language"&&! ["pt-BR","fr-FR","es-ES"].contains(&val.as_str()){return Err("INVALID_SETTING".into())}
 exec(db,"INSERT INTO settings(key,value) VALUES (?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",&[&key,&val])?;Ok(json!(true))},
 "articles"=>rows(db,"SELECT * FROM articles WHERE active=1 ORDER BY designation",&[]),
 "articles:all"=>rows(db,"SELECT a.*,c.name AS category_name FROM articles a LEFT JOIN categories c ON c.id=a.category_id ORDER BY a.designation",&[]),
 "clients:list"=>rows(db,"SELECT * FROM clients ORDER BY last_name,first_name",&[]),
 "categories:list"=>rows(db,"SELECT * FROM categories ORDER BY name",&[]),
 "scales:list"=>rows(db,"SELECT * FROM scales ORDER BY is_default DESC,name",&[]),
 "history"=> {let from=s(&v);let to=s(&a(args,1));let typ=s(&a(args,2));
 if from.len()!=10||to.len()!=10 {return Err("INVALID_DATE".into())}
 rows(db,"SELECT s.*,c.first_name||' '||c.last_name AS client_name,COALESCE((SELECT SUM(amount_cents) FROM payments p WHERE p.sale_id=s.id),0) AS paid_cents FROM sales s LEFT JOIN clients c ON c.id=s.client_id WHERE substr(s.created_at,1,10) BETWEEN ?1 AND ?2 AND (?3='all' OR ?3='invoice' OR (?3='payment' AND EXISTS (SELECT 1 FROM payments p WHERE p.sale_id=s.id))) ORDER BY s.created_at DESC LIMIT 500",&[&from,&to,&typ])},
 "clients:save"=>{let first=fs(&v,"first_name");let last=fs(&v,"last_name");let gender=fs(&v,"gender");let document=fs(&v,"document").chars().filter(|c|c.is_ascii_digit()).collect::<String>();let photo=fs(&v,"photo");
 if first.is_empty(){return Err("CLIENT_FIRST_NAME_REQUIRED".into())}if last.is_empty(){return Err("CLIENT_LAST_NAME_REQUIRED".into())}if !["","M","F","X"].contains(&gender.as_str()){return Err("CLIENT_INVALID_GENDER".into())}if ![11,14].contains(&document.len()){return Err("CPF_CNPJ_LENGTH".into())}if photo.len()>180000{return Err("PHOTO_TOO_LARGE".into())}
 let id=fi(&v,"id");if id>0 {exec(db,"UPDATE clients SET gender=?1,first_name=?2,last_name=?3,photo=?4,document=?5 WHERE id=?6",&[&gender,&first,&last,&photo,&document,&id])?;Ok(json!({"id":id}))}
 else{let stamp=stamp();exec(db,"INSERT INTO clients(gender,first_name,last_name,photo,document,created_at) VALUES (?1,?2,?3,?4,?5,?6)",&[&gender,&first,&last,&photo,&document,&stamp])?;Ok(saved_id(db))}},
 "clients:delete"=>{let id=i(&v);let n:i64=db.query_row("SELECT COUNT(*) FROM sales WHERE client_id=?1",[id],|r|r.get(0)).map_err(err)?;if n>0{return Err("CLIENT_HAS_OPERATIONS".into())}delete(db,"clients",id)},
 "categories:save"=>{let name=fs(&v,"name");if name.is_empty(){return Err("INVALID_CATEGORY".into())}let id=fi(&v,"id");
 if id>0{exec(db,"UPDATE categories SET name=?1 WHERE id=?2",&[&name,&id])?;Ok(json!({"id":id}))}
 else{exec(db,"INSERT INTO categories(name) VALUES(?1)",&[&name])?;Ok(saved_id(db))}},
 "categories:delete"=>{let id=i(&v);let n:i64=db.query_row("SELECT COUNT(*) FROM articles WHERE category_id=?1",[id],|r|r.get(0)).map_err(err)?;if n>0{return Err("CATEGORY_IN_USE".into())}delete(db,"categories",id)},
 "articles:save"=>{let name=fs(&v,"designation");let code=fs(&v,"code_article");let price=fi(&v,"price_per_kg");let icon=fs(&v,"icon");let cat=field(&v,"category_id").as_i64();let editable=if field(&v,"price_editable").as_bool().unwrap_or(false)||fi(&v,"price_editable")==1{1}else{0};let cost=field(&v,"cost_per_kg").as_i64();let sale_price=field(&v,"sale_price_per_kg").as_i64();if sale_price.map(|x|x<0||x>100_000_000).unwrap_or(false){return Err("PREÇO_DE_VENDA_INVÁLIDO".into())}if cost.map(|x|x<0||x>100_000_000).unwrap_or(false){return Err("CUSTO_INVÁLIDO".into())}let ncm=fs(&v,"ncm");let cest=fs(&v,"cest");let origem=fs(&v,"origem_icms");let cst=fs(&v,"cst_icms");let csosn=fs(&v,"csosn");let cfop_entrada=fs(&v,"cfop_entrada");let cfop_saida=fs(&v,"cfop_saida");
 if ncm.len()!=8||!ncm.bytes().all(|c|c.is_ascii_digit()){return Err("NCM_OBRIGATORIO_OITO_DIGITOS".into())}
 if !cest.is_empty()&&(cest.len()!=7||!cest.bytes().all(|c|c.is_ascii_digit())) {return Err("CEST_INVALIDO".into())}
 if !origem.is_empty()&&!(origem.len()==1&&origem.as_bytes()[0]>=b'0'&&origem.as_bytes()[0]<=b'8') {return Err("ORIGEM_ICMS_INVALIDA".into())}
 if !cst.is_empty()&&(cst.len()!=2||!cst.bytes().all(|c|c.is_ascii_digit())){return Err("CST_ICMS_INVALIDO".into())}
 if !csosn.is_empty()&&(csosn.len()!=3||!csosn.bytes().all(|c|c.is_ascii_digit())){return Err("CSOSN_INVALIDO".into())}
 if [&cfop_entrada,&cfop_saida].iter().any(|c|!c.is_empty()&&(c.len()!=4||!c.bytes().all(|x|x.is_ascii_digit()))){return Err("CFOP_INVALIDO".into())}
 let active=if fi(&v,"active")==0 && field(&v,"active")!=&Value::Null {0}else{1};
 if code.is_empty()||name.is_empty()||price<0{return Err("INVALID_ARTICLE".into())}let id=fi(&v,"id");
 if id>0{exec(db,"UPDATE articles SET code_article=?1,designation=?2,price_per_kg=?3,category_id=?4,icon=?5,active=?6,price_editable=?8,cost_per_kg=?9,sale_price_per_kg=?10,ncm=?11,cest=?12,origem_icms=?13,cst_icms=?14,csosn=?15,cfop_entrada=?16,cfop_saida=?17 WHERE id=?7",&[&code,&name,&price,&cat,&icon,&active,&id,&editable,&cost,&sale_price,&ncm,&cest,&origem,&cst,&csosn,&cfop_entrada,&cfop_saida])?;Ok(json!({"id":id}))}
 else{exec(db,"INSERT INTO articles(code_article,designation,price_per_kg,category_id,icon,price_editable,cost_per_kg,sale_price_per_kg,ncm,cest,origem_icms,cst_icms,csosn,cfop_entrada,cfop_saida) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15)",&[&code,&name,&price,&cat,&icon,&editable,&cost,&sale_price,&ncm,&cest,&origem,&cst,&csosn,&cfop_entrada,&cfop_saida])?;Ok(saved_id(db))}},
 "articles:delete"=>{let id=i(&v);let n:i64=db.query_row("SELECT COUNT(*) FROM sale_lines WHERE article_id=?1",[id],|r|r.get(0)).map_err(err)?;if n>0 {exec(db,"UPDATE articles SET active=0 WHERE id=?1",&[&id])?;Ok(json!("archived"))}else{delete(db,"articles",id)}},
 "scales:save"=>{let name=fs(&v,"name");let transport=fs(&v,"transport");if name.is_empty(){return Err("SCALE_NAME_REQUIRED".into())}if !["tcp","serial","bluetooth"].contains(&transport.as_str()){return Err("SCALE_INVALID_TRANSPORT".into())}
 let host=fs(&v,"host");let port=fi(&v,"port");let brand=fs(&v,"brand");let model=fs(&v,"model");let serial=fs(&v,"serial_path");let baud=if fi(&v,"baud_rate")>0{fi(&v,"baud_rate")}else{9600};let photo=fs(&v,"photo");let def=if fi(&v,"is_default")!=0 {1}else{0};let id=fi(&v,"id");
 if photo.len()>180000{return Err("PHOTO_TOO_LARGE".into())}
 let tx=db.transaction().map_err(err)?;
 if def==1 {exec(&tx,"UPDATE scales SET is_default=0",&[])?;}
 if id>0{exec(&tx,"UPDATE scales SET name=?1,host=?2,port=?3,brand=?4,model=?5,transport=?6,serial_path=?7,baud_rate=?8,photo=?9,is_default=?10 WHERE id=?11",&[&name,&host,&port,&brand,&model,&transport,&serial,&baud,&photo,&def,&id])?;tx.commit().map_err(err)?;Ok(json!({"id":id}))}
 else{exec(&tx,"INSERT INTO scales(name,host,port,brand,model,transport,serial_path,baud_rate,photo,is_default) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10)",&[&name,&host,&port,&brand,&model,&transport,&serial,&baud,&photo,&def])?;let new_id=tx.last_insert_rowid();tx.commit().map_err(err)?;Ok(json!({"id":new_id}))}},
 "scales:delete"=>delete(db,"scales",i(&v)),
 "maintenance:reset-financial"=>{
  if fs(&v,"confirmation")!="ZERAR FINANCEIRO" {return Err("CONFIRMAÇÃO_OBRIGATÓRIA".into())}
  let _=app.emit("maintenance:progress",json!({"level":"info","message":"Verificando o caminho do banco de dados…"}));
  let source=db.path().ok_or("BANCO_SEM_CAMINHO")?;
  let path=std::path::Path::new(source);
  let backup=path.with_file_name(format!("lxo-pre-reset-financeiro-{}.sqlite",Utc::now().format("%Y%m%d-%H%M%S-%6f")));
  let backup_name=backup.to_str().ok_or("CAMINHO_INVALIDO")?;
  let _=app.emit("maintenance:progress",json!({"level":"info","message":"Contando as movimentações financeiras…"}));
  // Contagem antes da manutenção para conferência no log do frontend.
  let tables=["payments","sale_lines","sales","caixa_logs","caixa_movements","caixa_sessions","expenses"];
  let mut removed=serde_json::Map::new();
  for table in tables {
   let query=format!("SELECT COUNT(*) FROM {}",table);
   let count:i64=db.query_row(&query,[],|r|r.get(0)).map_err(|e|format!("ERRO_CONTAGEM_{}: {}",table,e))?;
   removed.insert(table.into(),json!(count));
  }
  // Snapshot consistente também em modo WAL. Abortamos ANTES de alterar qualquer dado se falhar.
  let _=app.emit("maintenance:progress",json!({"level":"info","message":"Criando cópia de segurança consistente do SQLite…"}));
  db.execute("VACUUM INTO ?1",[backup_name]).map_err(|e|format!("ERRO_BACKUP: {}",e))?;
  let _=app.emit("maintenance:progress",json!({"level":"ok","message":format!("Cópia de segurança criada: {}",backup_name)}));
  let _=app.emit("maintenance:progress",json!({"level":"info","message":"Iniciando a transação de limpeza financeira…"}));
  let tx=db.transaction().map_err(|e|format!("ERRO_TRANSACAO: {}",e))?;
  tx.execute_batch("DELETE FROM payments; DELETE FROM sale_lines; DELETE FROM sales; DELETE FROM caixa_logs; DELETE FROM caixa_movements; DELETE FROM caixa_sessions; DELETE FROM expenses;").map_err(|e|format!("ERRO_LIMPEZA: {}",e))?;
  let _=app.emit("maintenance:progress",json!({"level":"info","message":"Confirmando a limpeza da base…"}));
  tx.commit().map_err(|e|format!("ERRO_CONFIRMACAO: {}",e))?;
  let _=app.emit("maintenance:progress",json!({"level":"ok","message":"Dados financeiros removidos. Cadastros preservados."}));
  Ok(json!({"ok":true,"backup":backup_name,"removed":removed}))
 },
 "sales:history"=>{
  let from=fs(&v,"from");let to=fs(&v,"to");
  if chrono::NaiveDate::parse_from_str(&from,"%Y-%m-%d").is_err()||chrono::NaiveDate::parse_from_str(&to,"%Y-%m-%d").is_err()||from>to {return Err("INVALID_DATE".into())}
  rows(db,"SELECT s.id,s.invoice_number,s.created_at,s.total_cents,s.buyer_company,COALESCE(SUM(l.weight_grams),0) weight_grams,COALESCE(GROUP_CONCAT(DISTINCT a.designation),'') materials,COALESCE(SUM(CASE WHEN l.cost_per_kg IS NOT NULL THEN (l.weight_grams*l.cost_per_kg+500)/1000 ELSE 0 END),0) cost_cents,COALESCE(SUM(CASE WHEN l.cost_per_kg IS NULL THEN 1 ELSE 0 END),0) missing_cost_lines,(SELECT p.created_at FROM sales p WHERE p.kind='sale' AND (p.created_at<s.created_at OR (p.created_at=s.created_at AND p.id<s.id)) ORDER BY p.created_at DESC,p.id DESC LIMIT 1) previous_at FROM sales s JOIN sale_lines l ON l.sale_id=s.id JOIN articles a ON a.id=l.article_id WHERE s.kind='sale' AND substr(s.created_at,1,10) BETWEEN ?1 AND ?2 GROUP BY s.id ORDER BY s.created_at DESC,s.id DESC LIMIT 500",&[&from,&to])
 },
 "sale:detail"=>sale_detail(db,i(&v)),
 "checkout"=>{
  let payload=&v;
  let kind=fs(payload,"kind");
  if !["sale","purchase"].contains(&kind.as_str()){return Err("INVALID_OPERATION_KIND".into())}
  let lines=payload.get("lines").and_then(Value::as_array).ok_or("INVALID_SALE")?;
  if lines.is_empty(){return Err("INVALID_SALE".into())}
  let method=fs(payload,"method");if !["cash","pix","card","transfer"].contains(&method.as_str()){return Err("INVALID_PAYMENT_METHOD".into())}
  let client=field(payload,"clientId").as_i64();
  let buyer_company=fs(payload,"buyerCompany");if buyer_company.len()>150{return Err("EMPRESA_INVALIDA".into())}
  let caixa_id=caixa::active_id(db)?;
  let tx=db.transaction().map_err(err)?;
  if let Some(cid)=client{if one(&tx,"SELECT id FROM clients WHERE id=?1",&[&cid])?.is_null(){return Err("INVALID_CLIENT".into())}}
  let mut items=Vec::new();let mut total=0_i64;
  for line in lines {
   let article_id=fi(line,"articleId");let grams=fi(line,"weightGrams");if grams<=0||grams>100_000_000{return Err("INVALID_WEIGHT".into())}
   let p=one(&tx,"SELECT price_per_kg,price_editable,cost_per_kg FROM articles WHERE id=?1 AND active=1",&[&article_id])?;
   if p.is_null(){return Err("INVALID_ARTICLE".into())}
   let catalogue_price=i(field(&p,"price_per_kg"));
   let price=if kind=="sale" {line.get("pricePerKg").and_then(Value::as_i64).ok_or("INVALID_SALE_PRICE")?}else{
      if let Some(custom)=line.get("pricePerKg").and_then(Value::as_i64){if custom!=catalogue_price && p["price_editable"].as_i64()!=Some(1){return Err("PREÇO_NÃO_EDITÁVEL".into())}custom}else{catalogue_price}
   };
   if price<=0||price>100_000_000{return Err("INVALID_PRICE".into())}
   let sum=grams.checked_mul(price).ok_or("AMOUNT_OVERFLOW")?.checked_add(500).ok_or("AMOUNT_OVERFLOW")?/1000;
   let cost=if kind=="sale" {let avg:Option<i64>=tx.query_row("SELECT (SUM(l.weight_grams*l.price_per_kg)/SUM(l.weight_grams)) FROM sale_lines l JOIN sales s ON s.id=l.sale_id WHERE s.kind='purchase' AND l.article_id=?1 AND s.created_at <= ?2",rusqlite::params![article_id, stamp()],|r|r.get(0)).map_err(err)?;avg.or_else(||p["cost_per_kg"].as_i64())}else{Some(price)};
   total=total.checked_add(sum).ok_or("AMOUNT_OVERFLOW")?;items.push((article_id,grams,price,sum,cost));
  }
  if total<=0{return Err("INVALID_TOTAL".into())}
  if kind=="purchase" && method=="cash" {
    let balance:i64=tx.query_row("SELECT c.initial_balance_cents + COALESCE((SELECT SUM(m.amount_cents * CASE WHEN m.kind='deposit' THEN 1 ELSE -1 END) FROM caixa_movements m WHERE m.caixa_id=c.id),0) + COALESCE((SELECT SUM(CASE WHEN s.kind='sale' THEN p.amount_cents WHEN s.kind='purchase' THEN -p.amount_cents ELSE 0 END) FROM payments p JOIN sales s ON s.id=p.sale_id WHERE p.caixa_id=c.id AND p.method='cash'),0) FROM caixa_sessions c WHERE c.id=?1",[caixa_id],|r|r.get(0)).map_err(err)?;
    if total>balance{return Err("SALDO_INSUFICIENTE_PARA_COMPRA".into())}
  }
  let created=stamp();exec(&tx,"INSERT INTO sales(created_at,kind,total_cents,client_id,caixa_id,buyer_company) VALUES (?1,?2,?3,?4,?5,?6)",&[&created,&kind,&total,&client,&caixa_id,&buyer_company])?;
  let id=tx.last_insert_rowid();let invoice=format!("PR-{:08}",id);
  exec(&tx,"UPDATE sales SET invoice_number=?1 WHERE id=?2",&[&invoice,&id])?;
  for (article,grams,price,sum,cost) in items {exec(&tx,"INSERT INTO sale_lines(sale_id,article_id,weight_grams,price_per_kg,total_cents,cost_per_kg) VALUES (?1,?2,?3,?4,?5,?6)",&[&id,&article,&grams,&price,&sum,&cost])?;}
  let receipt=format!("REC-{}-{}",id,Utc::now().timestamp_micros());
  let actor=user.to_string();
  exec(&tx,"INSERT INTO payments(sale_id,created_at,amount_cents,method,receipt_number,caixa_id) VALUES (?1,?2,?3,?4,?5,?6)",&[&id,&created,&total,&method,&receipt,&caixa_id])?;
  exec(&tx,"INSERT INTO caixa_logs(caixa_id,user,action,created_at) VALUES (?1,?2,?3,?4)",&[&caixa_id,&actor,&format!("{} {} via {}",if kind=="sale"{"Venda"}else{"Compra"},invoice,method),&created])?;
  tx.commit().map_err(err)?;
  Ok(json!({"id":id,"invoice":invoice,"totalCents":total,"caixaId":caixa_id,"method":method,"kind":kind}))
 },
 "sale"=>{let lines=v.as_array().ok_or("INVALID_SALE")?;if lines.is_empty(){return Err("INVALID_SALE".into())}
 let kind=s(&a(args,1));if !["sale","purchase"].contains(&kind.as_str()){return Err("INVALID_SALE".into())}
 let client=a(args,2).as_i64();let tx=db.transaction().map_err(err)?;
 if let Some(client_id)=client {let found=one(&tx,"SELECT id FROM clients WHERE id=?1",&[&client_id])?;if found.is_null(){return Err("INVALID_CLIENT".into())}}
 let mut normalized:Vec<(i64,i64,i64,i64)>=Vec::new();let mut total=0_i64;
 for line in lines {let article_id=fi(line,"articleId");let grams=fi(line,"weightGrams");if grams<=0{return Err("INVALID_LINE".into())}
 let product=one(&tx,"SELECT price_per_kg,price_editable,cost_per_kg FROM articles WHERE id=?1 AND active=1",&[&article_id])?;if product.is_null(){return Err("INVALID_LINE".into())}
 let price=i(field(&product,"price_per_kg"));let sum=((grams as f64)*(price as f64)/1000.0).round() as i64;
 total=total.checked_add(sum).ok_or("AMOUNT_OVERFLOW")?;normalized.push((article_id,grams,price,sum));}
 let created=stamp();exec(&tx,"INSERT INTO sales(created_at,kind,total_cents,client_id) VALUES(?1,?2,?3,?4)",&[&created,&kind,&total,&client])?;
 let id=tx.last_insert_rowid();let invoice=format!("LXO-{:08}",id);
 exec(&tx,"UPDATE sales SET invoice_number=?1 WHERE id=?2",&[&invoice,&id])?;
 for (article,grams,price,sum) in normalized{exec(&tx,"INSERT INTO sale_lines(sale_id,article_id,weight_grams,price_per_kg,total_cents) VALUES (?1,?2,?3,?4,?5)",&[&id,&article,&grams,&price,&sum])?;}
 tx.commit().map_err(err)?;Ok(json!({"id":id,"invoice":invoice,"totalCents":total,"createdAt":created}))},
 "payment"=>{let sale_id=i(&v);let amount=i(&a(args,1));let method=s(&a(args,2));if amount<=0||!["cash","pix","card","transfer"].contains(&method.as_str()){return Err("INVALID_PAYMENT".into())}
 let tx=db.transaction().map_err(err)?;let sale=one(&tx,"SELECT total_cents FROM sales WHERE id=?1",&[&sale_id])?;if sale.is_null(){return Err("NOT_FOUND".into())}
 let paid:i64=tx.query_row("SELECT COALESCE(SUM(amount_cents),0) FROM payments WHERE sale_id=?1",[sale_id],|r|r.get(0)).map_err(err)?;
 if paid+amount>i(field(&sale,"total_cents")){return Err("OVERPAYMENT".into())}
 let stamp=stamp();let receipt=format!("REC-{}-{}",sale_id,Utc::now().timestamp_micros());
 exec(&tx,"INSERT INTO payments(sale_id,created_at,amount_cents,method,receipt_number) VALUES(?1,?2,?3,?4,?5)",&[&sale_id,&stamp,&amount,&method,&receipt])?;
 let id=tx.last_insert_rowid();tx.commit().map_err(err)?;Ok(json!({"id":id}))},
 _=>Err(format!("UNKNOWN_COMMAND: {channel}")),
 }
}

// A restricted read model for employees. No cost figures or administration metadata.
pub fn employee_articles(db:&Connection)->Result<Value,String>{
    rows(db,"SELECT id,code_article,designation,price_per_kg,icon,active,price_editable FROM articles WHERE active=1 ORDER BY designation",&[])
}
