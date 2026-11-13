import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
const ui=fs.readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const store=fs.readFileSync(new URL('../src-tauri/src/store.rs',import.meta.url),'utf8');
const lib=fs.readFileSync(new URL('../src-tauri/src/lib.rs',import.meta.url),'utf8');
const css=fs.readFileSync(new URL('../src/renderer/style.css',import.meta.url),'utf8');
test('pesagem permite busca por codigo e nome com foco por Enter',()=>{
 assert.match(ui,/preferCode\/>/);assert.match(ui,/code:p.code_article/);
 assert.match(ui,/onEnterSelected=\{\(\)=>weightFocus\.current\?\.focus\(\)\}/);
 assert.match(ui,/buyPriceFocus\.current\?\.focus\(\)/);
 assert.match(ui,/const addLine=\(\)=>/);assert.match(ui,/productFocus\.current\?\.focus\(\)/);
 assert.match(store,/SELECT id,code_article,designation,price_per_kg,icon,active,price_editable FROM articles/);
});
test('preco unitario a vista, editavel apenas com permissao',()=>{
 assert.match(ui,/Preço unitário \(R\$\/kg\)/);assert.match(ui,/readOnly=\{!selected\?\.price_editable\}/);
 assert.match(css,/\.weighPriceRow\{/);
});
test('caixa tem acesso ao detalhe de cada transacao',()=>{
 assert.match(ui,/onDetail=\{readCashDetail\}/);assert.match(ui,/sale:detail/);
 assert.match(ui,/Ver produtos, pesos e valores/);assert.match(ui,/Peso<\/th>/);
 assert.match(css,/\.caixaMainButton\{/);
});
test('zeramento financeiro protegido e transacional, preserva cadastros',()=>{
 assert.match(ui,/ZERAR FINANCEIRO/);assert.match(store,/"maintenance:reset-financial"/);
 assert.match(store,/VACUUM INTO \?1/);
 assert.match(store,/let tx=db\.transaction\(\)/);
 for(const table of ['payments','sale_lines','sales','caixa_logs','caixa_movements','caixa_sessions','expenses'])assert.match(store,new RegExp('DELETE FROM '+table));
 for(const table of ['articles','users','clients','scales','categories','settings'])assert.doesNotMatch(store,new RegExp('DELETE FROM '+table));
 assert.match(lib,/me\.role!="admin" && !employee_allowed/);
});
test('versao 7.0.0 sem regressao da licenca',()=>{
 assert.match(ui,/Versão 8\.2\.0/);assert.match(lib,/license::assert_valid/);
});
