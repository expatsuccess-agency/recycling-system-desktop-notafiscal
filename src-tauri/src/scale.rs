use std::{io::{Read, ErrorKind}, net::{TcpStream,ToSocketAddrs},time::{Duration,SystemTime,UNIX_EPOCH}};
use serde_json::{json, Value};
fn text(v:&Value,k:&str)->String{v.get(k).and_then(Value::as_str).unwrap_or("").to_string()}
fn number(v:&Value,k:&str)->i64{v.get(k).and_then(Value::as_i64).unwrap_or(0)}
fn parse(line:&str)->Option<Value>{
 let raw=line.trim();
 let stable=raw.split(|c:char|c==','||c==';'||c.is_whitespace()).any(|x|x.eq_ignore_ascii_case("ST"));
 if !stable { return None; }
 let pieces=raw.split(|c:char| c==','||c==';'||c.is_whitespace());
 for item in pieces {
  let x=item.trim();let lower=x.to_ascii_lowercase();
  let (num,mult)=if lower.ends_with("kg"){(&x[..x.len()-2],1.)}else if lower.ends_with('g'){(&x[..x.len()-1],0.001)}else{continue};
  if let Ok(v)=num.replace(',',".").parse::<f64>() {if v>0.0&&v.is_finite(){
    let at=SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_millis();
    return Some(json!({"kg":v*mult,"stable":true,"raw":raw,"at":at}));
  }}
 }
 None
}
fn read_stable(mut source:impl Read)->Result<Value,String>{
 let mut buf=[0_u8;512];let mut text=String::new();
 let deadline=std::time::Instant::now()+Duration::from_millis(4500);
 while std::time::Instant::now()<deadline {
  match source.read(&mut buf) {
   Ok(0)=>break,
   Ok(n)=>{text.push_str(&String::from_utf8_lossy(&buf[..n]));
     for line in text.split(|c|c=='\r'||c=='\n') { if let Some(weight)=parse(line){return Ok(weight)} }
     if text.len()>4096 {text=text.chars().rev().take(2048).collect::<String>().chars().rev().collect();}
   },
   Err(e) if e.kind()==ErrorKind::TimedOut || e.kind()==ErrorKind::WouldBlock => {},
   Err(e)=>return Err(format!("SCALE_READ_ERROR: {e}")),
  }
 }
 Err("SCALE_TIMEOUT_OR_UNSTABLE".into())
}
pub fn capture(s:&Value)->Result<Value,String>{
 match text(s,"transport").as_str(){
 "tcp"=>{let host=text(s,"host");let port=number(s,"port");if host.is_empty()||!(1..=65535).contains(&port){return Err("INVALID_SCALE_ENDPOINT".into())}
  let address=format!("{}:{}",host,port);let mut addrs=address.to_socket_addrs().map_err(|e|format!("SCALE_ADDRESS: {e}"))?;
  let addr=addrs.next().ok_or("SCALE_ADDRESS_NOT_FOUND")?;
  let stream=TcpStream::connect_timeout(&addr,Duration::from_millis(3500)).map_err(|e|format!("SCALE_CONNECTION_ERROR: {e}"))?;
  stream.set_read_timeout(Some(Duration::from_millis(500))).map_err(|e|e.to_string())?;
  read_stable(stream)
 },
 "serial"|"bluetooth"=>{
  let path=text(s,"serial_path");if path.is_empty(){return Err("SERIAL_PORT_REQUIRED".into())}
  let baud=number(s,"baud_rate").max(1) as u32;
  let port=serialport::new(path,baud).timeout(Duration::from_millis(500)).open().map_err(|e|format!("SERIAL_CONNECTION_ERROR: {e}"))?;
  read_stable(port)
 },
 _=>Err("UNSUPPORTED_TRANSPORT".into())
 }
}
