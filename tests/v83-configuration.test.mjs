import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const read=(p)=>readFileSync(new URL('../'+p,import.meta.url),'utf8');
test('novo mapeamento Fn V8.3 e menus',()=>{
 const s=read('src/renderer/index.tsx');
 assert.ok(s.includes("F1:'weigh',F3:'vendas',F7:'finance',F9:'config'"));
 assert.ok(s.includes("ev.key==='F5'||ev.key==='F11'"));
 for(const label of ["label:'Controle',key:'F3'","label:'Caixa',key:'F5'","label:'Gestão Financeira',key:'F7'","label:'Configuração',key:'F9'",'Caixas Anteriores <kbd>F11</kbd>'])assert.ok(s.includes(label));
 assert.ok(!s.includes("F2:'vendas'"));assert.ok(!s.includes("F8:'finance'"));
 assert.ok(s.includes("ev.key==='F12'"));
});
test('lotes não emitem NF-e nem falsificam autorização',()=>{
 const s=read('src-tauri/src/fiscal_batch.rs');
 assert.ok(s.includes('ENVIO_LOTE_SEFAZ_NAO_IMPLEMENTADO_NAO_TRANSMITIDO'));
 assert.ok(s.includes('APROVACAO_EXPLICITA_OBRIGATORIA'));
 assert.ok(s.includes('"transmitted":false'));
 assert.ok(s.includes("origin='scheduled'"));
});
test('relatórios e fechamento de caixa',()=>{
 assert.ok(read('src-tauri/src/reports.rs').includes('automatic_close'));
 assert.ok(read('src-tauri/src/lib.rs').includes('reports::automatic_close'));
 assert.ok(read('src-tauri/src/pdf.rs').includes('PAGE_LINES:usize=45'));
 assert.ok(read('src/renderer/FiscalPortal.tsx').includes('<ReportsPanel api={api}/>'));
});
