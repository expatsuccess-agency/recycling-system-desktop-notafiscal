mod store;
mod caixa;
mod scale;
mod pdf;
mod license;
mod auth;
mod finance;
mod fiscal;
mod a1;

use std::sync::Mutex;
use tauri::{Manager, State};
use rusqlite::Connection;
use serde_json::{json, Value};

struct LxoState(Mutex<Connection>);

#[tauri::command]
fn lxo_call(channel: String, args: Vec<Value>, state: State<'_, LxoState>, sessions: State<'_, auth::Sessions>, session: Option<String>, app: tauri::AppHandle) -> Result<Value, String> {
    let a = |i: usize| args.get(i).cloned().unwrap_or(Value::Null);
    if !matches!(channel.as_str(), "license:machine" | "license:status" | "license:install" | "license:use-external" | "license:use-internal") {
        license::assert_valid(&app)?;
    }
    if channel.starts_with("auth:") {
        let db=state.0.lock().map_err(|_|"LOCK_FAILED")?;
        return auth::dispatch(&db,&channel,&a(0),&sessions,session.as_deref());
    }
    if !channel.starts_with("license:") {
        let me=auth::current(session.as_deref(),&sessions)?;
        if me.must_change { return Err("ALTERAÇÃO_DE_SENHA_OBRIGATÓRIA".into()); }
        let employee_allowed=matches!(channel.as_str(),"articles"|"articles:all"|"checkout") || channel.starts_with("caixa:");
        if me.role!="admin" && !employee_allowed {return Err("ACESSO_NEGADO".into())}
        if me.role=="employee" && channel=="checkout" && a(0).get("kind").and_then(Value::as_str)!=Some("purchase") {return Err("ACESSO_NEGADO".into())}
    }
    // Employees receive only operational product fields, never acquisition costs.
    if matches!(channel.as_str(), "articles" | "articles:all") {
        let me=auth::current(session.as_deref(),&sessions)?;
        if me.role=="employee" {
            let db=state.0.lock().map_err(|_|"LOCK_FAILED")?;
            return store::employee_articles(&db);
        }
    }
    match channel.as_str() {
        "license:machine" => Ok(json!({"machineId": license::machine_id()?})),
        "license:status" => Ok(json!(license::status(&app))),
        "license:install" => Ok(json!(license::install(&app,a(0).as_str().ok_or("ARQUIVO_INVÁLIDO")?)?)),
        "license:use-external" => Ok(json!(license::use_external(&app,a(0).as_str().ok_or("CAMINHO_INVÁLIDO")?)?)),
        "license:use-internal" => Ok(json!(license::use_internal(&app)?)),
        c if c.starts_with("fiscal:") || c.starts_with("debug:") => {let db=state.0.lock().map_err(|_|"LOCK_FAILED")?; fiscal::dispatch(&db,c,&a(0))},
        c if c.starts_with("finance:") => {let mut db=state.0.lock().map_err(|_|"LOCK_FAILED")?;let me=auth::current(session.as_deref(),&sessions)?;finance::dispatch(&mut db,c,&a(0),&me.username)},
        c if c.starts_with("caixa:") => { let mut db=state.0.lock().map_err(|_| "LOCK_FAILED")?; caixa::dispatch(&mut db,c,&args,&auth::current(session.as_deref(),&sessions)?.username) },

        "scale:capture" => {
            let scale = {
                let db = state.0.lock().map_err(|_| "LOCK_FAILED")?;
                store::scale_by_id(&db, a(0).as_i64().ok_or("NO_SCALE_SELECTED")?)?
            };
            scale::capture(&scale)
        }
        "printers:list" => {
            // Listing only: no printing is triggered by choosing an item.
            #[cfg(target_os = "windows")]
            let output = std::process::Command::new("powershell.exe")
                .args(["-NoProfile", "-Command", "Get-Printer | Select-Object -ExpandProperty Name"])
                .output();
            #[cfg(not(target_os = "windows"))]
            let output = std::process::Command::new("lpstat").arg("-e").output();
            let list: Vec<String> = match output {
                Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                    .lines().map(str::trim).filter(|s| !s.is_empty())
                    .map(ToOwned::to_owned).collect(),
                _ => Vec::new(),
            };
            Ok(json!(list))
        }
        "weight" => Err("NO_STABLE_RECENT_WEIGHT: use scale:capture or manual entry".into()),
        "pdf:open" => {
            let kind = a(0).as_str().unwrap_or("").to_owned();
            let id = a(1).as_i64().ok_or("INVALID_PDF")?;
            let data = {
                let db = state.0.lock().map_err(|_| "LOCK_FAILED")?;
                store::pdf_data(&db, &kind, id)?
            };
            pdf::make_and_open(&app, &kind, id, &data)
        }
        _ => {
            let mut db = state.0.lock().map_err(|_| "LOCK_FAILED")?;
            store::dispatch(&mut db, &channel, &args, &auth::current(session.as_deref(),&sessions)?.username, &app)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("lxo.sqlite");
            // Explicit optional first-run import. Never overwrite an existing Tauri database.
            if !db_path.exists() {
                if let Ok(old) = std::env::var("LXO_IMPORT_SQLITE") {
                    let src = std::path::Path::new(&old);
                    if src.is_file() { std::fs::copy(src, &db_path)?; }
                }
            }
            let db = Connection::open(db_path)?;
            store::initialize(&db).map_err(std::io::Error::other)?;
            auth::initialize(&db).map_err(std::io::Error::other)?;
            finance::initialize(&db).map_err(std::io::Error::other)?;
            fiscal::initialize(&db).map_err(std::io::Error::other)?;
            app.manage(LxoState(Mutex::new(db)));
            app.manage(auth::Sessions(Mutex::new(std::collections::HashMap::new())));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![lxo_call])
        .run(tauri::generate_context!())
        .expect("Failed to start LXO Tauri");
}
