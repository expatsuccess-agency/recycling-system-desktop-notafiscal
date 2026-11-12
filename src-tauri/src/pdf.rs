use serde_json::Value;
use tauri::Manager;
use std::path::PathBuf;
fn txt(v:&Value,key:&str)->String {match v.get(key){Some(Value::String(s))=>s.clone(),Some(v) if !v.is_null()=>v.to_string(),_=>String::new()}}
// Converte o texto para a codificação WinAnsi do PDF.
// Helvetica / WinAnsi suporta os acentos portugueses sem substituir por '?'.
fn winansi_pdf_string(s: &str) -> Vec<u8> {
 let mut out=Vec::new();
 for c in s.chars() {
  let byte=match c {
   '\\'=>Some(b'\\'), '('=>Some(b'('), ')'=>Some(b')'),
   '\n'|'\r'|'\t'=>Some(b' '),
   c if c.is_ascii()=>Some(c as u8),
   c if ('\u{00A0}'..='\u{00FF}').contains(&c)=>Some(c as u32 as u8),
   '€'=>Some(0x80),'‘'=>Some(0x91),'’'=>Some(0x92),
   '“'=>Some(0x93),'”'=>Some(0x94),'•'=>Some(0x95),
   '–'=>Some(0x96),'—'=>Some(0x97),'…'=>Some(0x85),
   _=>None
  };
  if let Some(b)=byte {
   if b==b'\\'||b==b'('||b==b')' {out.push(b'\\');}
   out.push(b);
  }else {out.push(b' ');} // caractere fora do repertorio da fonte
 }
 out
}
fn make_pdf(lines:&[String])->Vec<u8>{
 let mut content=b"BT /F1 11 Tf 40 790 Td 15 TL ".to_vec();
 for l in lines.iter().take(48){
  content.push(b'(');content.extend(winansi_pdf_string(l));content.extend_from_slice(b") Tj T* ");
 }
 content.extend_from_slice(b"ET");
 let objects=vec![
  b"<< /Type /Catalog /Pages 2 0 R >>".to_vec(),
  b"<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_vec(),
  b"<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_vec(),
  b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec(),
  {let mut obj=format!("<< /Length {} >>\nstream\n",content.len()).into_bytes();
   obj.extend_from_slice(&content);obj.extend_from_slice(b"\nendstream");obj}
 ];
 let mut out=b"%PDF-1.4\n".to_vec();let mut offsets=vec![0];
 for (idx,obj) in objects.iter().enumerate(){offsets.push(out.len());
  out.extend_from_slice(format!("{} 0 obj\n",idx+1).as_bytes());
  out.extend_from_slice(obj);out.extend_from_slice(b"\nendobj\n");}
 let xref=out.len();out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n",offsets.len()).as_bytes());
 for off in offsets.iter().skip(1){out.extend_from_slice(format!("{:010} 00000 n \n",off).as_bytes());}
 out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",offsets.len(),xref).as_bytes());out
}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn accents_winansi(){
  assert_eq!(winansi_pdf_string("Alumínio – Papelão"), b"Alum\xednio \x96 Papel\xe3o");
  assert!(make_pdf(&["Alumínio – Papelão".into()]).windows(7).any(|x|x==b"Alum\xednio"));
 }
}
pub fn make_and_open(app:&tauri::AppHandle,kind:&str,id:i64,data:&Value)->Result<Value,String>{
 let mut lines=vec![format!("RECYCLING SYSTEM INDUSTRIAL - {} {}",kind.to_uppercase(),id),"Documento operacional (não é nota fiscal eletrônica)".into(),"".into()];
 if kind=="client" {lines.push(format!("Cliente: {} {}",txt(data,"first_name"),txt(data,"last_name")));lines.push(format!("CPF/CNPJ: {}",txt(data,"document")));}
 else if kind=="sale" {lines.push(format!("Documento: {}",txt(data,"invoice_number")));lines.push(format!("Data: {}",txt(data,"created_at")));lines.push(format!("Cliente: {}",txt(data,"client_name")));
  for line in data["lines"].as_array().into_iter().flatten(){lines.push(format!("{} - {} g - {} centavos",txt(line,"designation"),txt(line,"weight_grams"),txt(line,"total_cents")));}
  lines.push(format!("TOTAL: R$ {:.2}",data["total_cents"].as_i64().unwrap_or(0) as f64/100.));
 }else{return Err("INVALID_PDF".into())}
 let documents:PathBuf=app.path().document_dir().map_err(|e|e.to_string())?;
 let folder=documents.join("Recycling System");std::fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
 let dest=folder.join(format!("Recycling-{}-{}.pdf",kind,id));std::fs::write(&dest,make_pdf(&lines)).map_err(|e|e.to_string())?;
 opener::open(&dest).map_err(|e|format!("PDF_SAVED_OPEN_FAILED: {e}"))?;
 Ok(Value::String(dest.to_string_lossy().to_string()))
}
