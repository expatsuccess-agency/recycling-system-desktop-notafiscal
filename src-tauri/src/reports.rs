//! Relatórios OPERACIONAIS, não equivalem a XML NF-e autorizado ou DANFE.
use rusqlite::{params,Connection};
use serde_json::{json,Value};
use tauri::Manager;
fn er(e:impl std::fmt::Display)->String{e.to_string()}
fn money(c:i64)->String{format!("R$ {:.2}",c as f64/100.0)}
fn get_date(v:&Value,key:&str)->Result<String,String>{
 let s=v.get(key).and_then(Value::as_str).ok_or("DATA_OBRIGATORIA")?;
 chrono::NaiveDate::parse_from_str(s,"%Y-%m-%d").map_err(|_|"DATA_INVALIDA".to_string())?;Ok(s.into())
}
fn query_lines(db:&Connection,sql:&str,from:&str,to:&str)->Result<Vec<String>,String>{
 let mut st=db.prepare(sql).map_err(er)?;
 let rows=st.query_map(params![from,to],|r|{
  let mut pieces=Vec::new();for i in 0..r.as_ref().column_count(){let x:rusqlite::types::Value=r.get(i)?;
   pieces.push(match x {rusqlite::types::Value::Null=>"-".into(),rusqlite::types::Value::Integer(x)=>x.to_string(),rusqlite::types::Value::Real(x)=>format!("{x:.2}"),rusqlite::types::Value::Text(x)=>x,rusqlite::types::Value::Blob(_)=>"[bin]".into()});}
  Ok(pieces.join(" | "))}).map_err(er)?;
 let mut result=Vec::new();for r in rows{result.push(r.map_err(er)?)}Ok(result)
}
fn section(lines:&mut Vec<String>,db:&Connection,from:&str,to:&str,label:&str,sql:&str)->Result<(),String>{
 lines.push(label.to_string());
 let values=query_lines(db,sql,from,to)?;
 if values.is_empty(){lines.push("Nenhum registro no periodo".into())}else{lines.extend(values)}
 lines.push(String::new());Ok(())
}
fn build_lines(db:&Connection,kind:&str,from:&str,to:&str)->Result<Vec<String>,String>{
 if from>to {return Err("PERIODO_INVALIDO".into())}
 let mut lines=vec![format!("RECYCLING SYSTEM INDUSTRIAL V8.3 - {}",kind.to_uppercase()),
   format!("Periodo: {from} a {to}"),"RELATORIO OPERACIONAL - NAO SUBSTITUI XML NF-e / DANFE".into(),"".into()];

 match kind {
  "caixa"|"consolidado"=>{
   section(&mut lines,db,from,to,"CAIXAS: ID | STATUS | ABERTURA | FECHAMENTO | SALDO INICIAL (CENTAVOS)",
    "SELECT id,status,substr(opened_at,1,19),coalesce(substr(closed_at,1,19),'-'),initial_balance_cents FROM caixa_sessions WHERE substr(opened_at,1,10) BETWEEN ?1 AND ?2 ORDER BY id")?;
  },_=>()}
 if ["caixa","consolidado"].contains(&kind){
   section(&mut lines,db,from,to,"CAIXA - OPERACOES: CAIXA | TIPO | QTD | TOTAL CENTAVOS",
    "SELECT coalesce(cast(caixa_id as text),'-'),kind,count(*),sum(total_cents) FROM sales WHERE substr(created_at,1,10) BETWEEN ?1 AND ?2 AND caixa_id IS NOT NULL GROUP BY caixa_id,kind ORDER BY caixa_id,kind")?;
   section(&mut lines,db,from,to,"CAIXA - PAGAMENTOS: MEIO | TOTAL CENTAVOS",
    "SELECT method,sum(amount_cents) FROM payments WHERE substr(created_at,1,10) BETWEEN ?1 AND ?2 GROUP BY method ORDER BY method")?;
   section(&mut lines,db,from,to,"MOVIMENTOS: CAIXA | TIPO | CENTAVOS | DATA",
    "SELECT caixa_id,kind,amount_cents,substr(created_at,1,19) FROM caixa_movements WHERE substr(created_at,1,10) BETWEEN ?1 AND ?2 ORDER BY id")?;
 }
 if ["compras","vendas","consolidado"].contains(&kind){
  let (title,filter)=if kind=="compras"{("COMPRAS","purchase")}else if kind=="vendas"{("VENDAS","sale")}else{("OPERACOES DE COMPRA E VENDA","both")};
  lines.push(format!("{title}: IDENTIFICADOR | DATA | TIPO | REFERENCIA | CENTAVOS"));
  let sql=format!("SELECT id,substr(created_at,1,19),kind,coalesce(invoice_number,'-'),total_cents FROM sales WHERE substr(created_at,1,10) BETWEEN ?1 AND ?2 {} ORDER BY id",if filter=="both"{""}else if filter=="purchase"{"AND kind='purchase'"}else{"AND kind='sale'"});
  let v=query_lines(db,&sql,from,to)?;
  if v.is_empty(){lines.push("Nenhuma operacao".into())}else{lines.extend(v)}
  let (total,count):(i64,i64)=db.query_row(&format!("SELECT coalesce(sum(total_cents),0),count(*) FROM sales WHERE substr(created_at,1,10) BETWEEN ?1 AND ?2 {}",if filter=="both"{""}else if filter=="purchase"{"AND kind='purchase'"}else{"AND kind='sale'"}),params![from,to],|r|Ok((r.get(0)?,r.get(1)?))).map_err(er)?;
  lines.push(format!("TOTAL: {} | {} operacoes",money(total),count));lines.push("".into());
 }
 if ["fiscal","consolidado"].contains(&kind){
  section(&mut lines,db,from,to,"DOCUMENTOS FISCAIS: ID | DATA | DIRECAO | STATUS | MODELO | SERIE | NUMERO | PROTOCOLO", 
   "SELECT id,substr(created_at,1,19),direction,status,model,series,number,coalesce(nullif(sefaz_protocol,''),'-') FROM fiscal_documents WHERE substr(created_at,1,10) BETWEEN ?1 AND ?2 ORDER BY id")?;
  section(&mut lines,db,from,to,"LOTES: ID | DATA | ORIGEM | STATUS ADMINISTRATIVO | APROVACAO", 
   "SELECT id,for_date,origin,status,coalesce(approved_at,'-') FROM fiscal_batches WHERE for_date BETWEEN ?1 AND ?2 ORDER BY id")?;
  lines.push("Sem emissao SEFAZ: nenhum lote neste modulo corresponde a autorizacao fiscal.".into());
 }
 if !["caixa","compras","vendas","fiscal","consolidado"].contains(&kind){return Err("TIPO_RELATORIO_INVALIDO".into())}
 Ok(lines)
}
pub fn dispatch(db:&Connection,app:&tauri::AppHandle,channel:&str,v:&Value)->Result<Value,String>{
 if channel!="reports:generate"{return Err("CANAL_RELATORIO_INVALIDO".into())}
 let kind=v.get("kind").and_then(Value::as_str).ok_or("TIPO_RELATORIO_INVALIDO")?;
 let from=get_date(v,"from")?;let to=get_date(v,"to")?;
 let lines=build_lines(db,kind,&from,&to)?;
 save(app,kind,&from,&to,&lines)
}
fn save(app:&tauri::AppHandle,kind:&str,from:&str,to:&str,lines:&[String])->Result<Value,String>{
 let folder=app.path().document_dir().map_err(er)?.join("Recycling System").join("Relatorios");
 std::fs::create_dir_all(&folder).map_err(er)?;
 let path=folder.join(format!("Relatorio-{}-{}-{}.pdf",kind,from,to));
 std::fs::write(&path,crate::pdf::make_pdf(lines)).map_err(er)?;
 Ok(json!({"path":path.to_string_lossy(),"pages":(lines.len()+44)/45,"fiscal_official":false}))
}
pub fn automatic_close(db:&Connection,app:&tauri::AppHandle,caixa_id:i64)->Result<Value,String>{
 let (start,end):(String,String)=db.query_row("SELECT substr(opened_at,1,10),substr(coalesce(closed_at,opened_at),1,10) FROM caixa_sessions WHERE id=?1",[caixa_id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(er)?;
 let mut lines=build_lines(db,"consolidado",&start,&end)?;
 lines.insert(1,format!("Caixa fechado: #{}",caixa_id));
 // A geração PDF não é requisito para fechar caixa; erros são informados separadamente.
 save(app,&format!("fechamento-caixa-{caixa_id}"),&start,&end,&lines)
}
