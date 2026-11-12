import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const ui=readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const css=readFileSync(new URL('../src/renderer/style.css',import.meta.url),'utf8');
const backend=readFileSync(new URL('../src-tauri/src/lib.rs',import.meta.url),'utf8');
const store=readFileSync(new URL('../src-tauri/src/store.rs',import.meta.url),'utf8');
test('suggestions only when typing or ArrowDown, never on focus',()=>{
 assert.match(ui,/onFocus=\{\(\)=>\{setOpen\(false\)/);
 assert.match(ui,/setOpen\(Boolean\(e.target.value.trim\(\)\)\)/);
});
test('layout uses equal weight and price columns, white icon selector',()=>{
 assert.match(css,/weighPriceRow\{grid-template-columns:repeat\(2,minmax\(0,1fr\)\)/);
 assert.match(css,/materialIconSelect\{background:#fff!important/);
});
test('navigation and dedicated configuration sections',()=>{
 assert.match(ui,/label:'Controle',key:'F2'/);
 for(const tab of ["'products'","'users'","'control'","'maintenance'"])assert.ok(ui.includes(`configTab===${tab}`));
 assert.ok(ui.indexOf("label:'Gestão Financeira'")<ui.indexOf("label:'Configuração'"));
 assert.match(ui,/ev.key==='F10'&&tab==='vendas'/);
});
test('employee authorized for caixa in Rust and UI',()=>{
 assert.match(backend,/channel.starts_with\("caixa:"\)/);
 assert.match(ui,/\['weigh','caixa'\]\.includes\(n.id\)/);
});
test('finance reset keeps backup, transaction and logs',()=>{
 assert.match(ui,/Registro detalhado da operação/);
 assert.match(ui,/setResetLogs/);
 assert.match(store,/VACUUM INTO/);
 assert.match(store,/tx.commit\(\)/);
 for(const table of ['payments','sales','caixa_sessions','expenses'])assert.match(store,new RegExp(`DELETE FROM ${table}`));
 assert.doesNotMatch(store,/DELETE FROM (?:articles|app_users|categories|clients|scales)/);
});
test('error code is translated into human messages',()=>{
 assert.match(ui,/CREDENCIAIS_INVÁLIDAS':'Usuário ou senha incorretos/);
});
