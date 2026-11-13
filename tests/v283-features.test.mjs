import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
const src=readFileSync(new URL('../src/renderer/index.tsx',import.meta.url),'utf8');
const css=readFileSync(new URL('../src/renderer/style.css',import.meta.url),'utf8');
test('brand, navigation and footer',()=>{
  assert.match(src,/Recycling System/);
  assert.match(src,/label:'Pesagem'/);
  assert.match(src,/Versão 8\.2\.0/);
  assert.match(src,/className="navFooter"/);
});
test('timeline beside the cash summary',()=>{
  assert.match(src,/className="caixaColumns"/);
  assert.match(css,/\.caixaColumns\{display:grid/);
  assert.match(src,/className="panel caixaTimelinePanel"/);
});
test('weight formatter uses optional fraction digits',()=>{
  assert.match(src,/const formatKg=/);
  assert.match(src,/minimumFractionDigits:0,maximumFractionDigits:3/);
  assert.doesNotMatch(src,/toFixed\(3\)/);
});
test('material icon picker uses large selectable icons',()=>{
  assert.doesNotMatch(src,/className="materialIconGrid"/);
  assert.match(css,/\.iconPickerTrigger\{width:118px/);
  assert.match(css,/\.iconPickerGrid\{/);
});
test('notifications disappear in 7 seconds and on navigation',()=>{
  assert.match(src,/setTimeout\(\(\)=>\{setMessage\(''\);setError\(''\)\},7000\)/);
  assert.match(src,/\[tab,caixaTab,configTab\]/);
});
