import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const view=readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const styles=readFileSync(new URL('../src/renderer/style.css',import.meta.url),'utf8');
const caixa=readFileSync(new URL('../src-tauri/src/caixa.rs',import.meta.url),'utf8');
test('menu left with admin F8 and Caixa F6/F7; Clientes hidden',()=>{
  assert.match(view,/className="leftNav"/);
  for(const n of [1,2,3,5,6,7,8])assert.match(view,new RegExp(`F${n}`));
  assert.doesNotMatch(view,/label:'Clientes',key:'F4'/);
  assert.match(styles,/\.leftNav\{order:-1/);
});
test('typeahead product/client search uses prefix matching',()=>{
  assert.match(view,/function AutoSelect/);
  assert.match(view,/startsWith\(normalize\(query\)\)/);
  assert.match(view,/onKeyDown=.*ArrowDown/);
  assert.match(view,/label="Produto para venda"/);
  assert.doesNotMatch(view,/Empresa compradora/);
});
test('Caixa timeline derives confirmed transactions from stored sales and payments',()=>{
  assert.match(caixa,/FROM sales s WHERE s.caixa_id=\?1/);
  assert.match(caixa,/FROM payments p WHERE p.sale_id=s.id/);
  assert.match(view,/function CaixaTimeline/);
  assert.match(view,/Em andamento/);
  assert.match(view,/Pago/);
  assert.match(caixa,/val\["timeline"\]/);
});
