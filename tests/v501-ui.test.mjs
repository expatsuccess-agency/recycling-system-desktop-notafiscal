import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const ui=readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const css=readFileSync(new URL('../src/renderer/style.css',import.meta.url),'utf8');
test('Gestão Financeira moved to left sidebar and displayed inside workspace',()=>{
 assert.doesNotMatch(ui, /className="headerFinance"/);
 const nav=ui.slice(ui.indexOf('<aside className="leftNav"'),ui.indexOf('navShortcuts'));
 assert.match(nav,/label:'Gestão Financeira',key:'F8'/);
 assert.doesNotMatch(nav,/label:'Clientes'/);
});
test('Produto Enter selects product and focuses Peso',()=>{
 assert.match(ui, /onEnterSelected=\{\(\)=>weightFocus\.current\?\.focus\(\)\}/);
 assert.match(ui, /onEnterSelected\?\.\(\)/);
 assert.match(ui, /ref=\{weightFocus\}/);
});
test('Dinheiro default and Enter confirms checkout',()=>{
 assert.match(ui,/useState\('cash'\)/);
 assert.match(ui,/setMethod\('cash'\);setPaymentOpen\(true\)/);
 assert.match(ui,/paymentCashFocus\.current\?\.focus\(\)/);
 assert.match(ui,/aria-pressed=\{method===m\.code\}/);
 assert.match(ui,/onClick=\{\(\)=>setMethod\(m\.code\)\}/);
 assert.match(ui,/aria-label="Escolher pagamento" onKeyDown=/);
 assert.match(ui,/if\(!busy\)void checkout\(\)/);
});
test('Product configuration grid is uniform and compact',()=>{
 assert.match(ui,/className="formGrid productFormGrid"/);
 assert.match(css,/\.productFormGrid\{display:grid;grid-template-columns:repeat\(3,minmax\(0,1fr\)\)/);
});
