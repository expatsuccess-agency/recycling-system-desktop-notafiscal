import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const view=readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const store=readFileSync(new URL('../src-tauri/src/store.rs',import.meta.url),'utf8');
const caixa=readFileSync(new URL('../src-tauri/src/caixa.rs',import.meta.url),'utf8');
test('navigation Pesagem / Vendas / Caixa in Brazilian Portuguese',()=>{
 assert.match(view,/label:'Pesagem'/); assert.match(view,/label:'Controle'/);
 assert.match(view,/label:'Caixa'/);assert.match(view,/Versão 8\.2\.0/);
});
test('Vendas form requests product, weight and price per kg',()=>{
 assert.match(view,/Produto para venda/);assert.match(view,/Peso \(kg\)/);
 assert.match(view,/Valor por kg \(R\$\)/);assert.match(view,/Valor total/);
});
test('checkout distinguishes purchase and sale and checks available cash',()=>{
 assert.match(store,/INVALID_OPERATION_KIND/);
 assert.match(store,/SALDO_INSUFICIENTE_PARA_COMPRA/);
 assert.match(store,/pricePerKg/);
 assert.match(caixa,/s.kind='purchase'/);
});
test('Caixa has separate purchase total and cash net',()=>{
 assert.match(view,/Total de Compras/);
 assert.match(caixa,/totalPurchases/);
});
test('frontend has no Electron bridge',()=>{
 assert.doesNotMatch(view,/window\.lxo|ipcRenderer|require\(['"]electron/);
});
